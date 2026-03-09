use crate::assets::Assets;
use crate::constants::{FIXED_DT, LEVEL_PATH};
use crate::editor::{EditorState, load_level, save_level};
use crate::input::gather_player_command;
use crate::render::{Renderer, camera};
use crate::replay::{ReplayController, ReplaySummary, default_replay_path};
use crate::world::{LevelData, World, default_level};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneMode {
    Play,
    Editor,
}

pub struct Game {
    pub assets: Assets,
    pub world: World,
    pub level: LevelData,
    pub renderer: Renderer,
    pub editor: EditorState,
    pub mode: SceneMode,
    pub prev_play_camera_center: Vec2,
    pub play_camera_center: Vec2,
    pub accumulator: f32,
    pub status_text: String,
    pub replay: ReplayController,
    pub pending_run_summary: Option<String>,
}

impl Game {
    pub fn new(assets: Assets) -> Self {
        let level = load_level(LEVEL_PATH).unwrap_or_else(default_level);
        let world = World::from_level(&level);
        let play_camera_center = camera::initial_play_camera_center(&world);
        let editor = EditorState::new(&world);

        let mut replay = ReplayController::new();
        let mut status_text = "Destroy cages, pick up hostages, extract up top.".to_owned();
        if let Ok(Some(message)) = replay.configure_from_env() {
            status_text = message;
        }

        let mut game = Self {
            assets,
            world,
            level,
            renderer: Renderer::new(),
            editor,
            mode: SceneMode::Play,
            prev_play_camera_center: play_camera_center,
            play_camera_center,
            accumulator: 0.0,
            status_text,
            replay,
            pending_run_summary: None,
        };
        if game.replay.active_mode() == crate::replay::ReplayMode::Playback {
            game.rebuild_world();
        }
        game
    }

    pub fn frame(&mut self, frame_dt: f32) {
        if is_key_pressed(KeyCode::Tab) {
            self.toggle_mode();
        }

        match self.mode {
            SceneMode::Play => self.update_play(frame_dt),
            SceneMode::Editor => self.update_editor(frame_dt),
        }
    }

    pub fn draw(&mut self) {
        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);
        let interpolated_camera = self
            .prev_play_camera_center
            .lerp(self.play_camera_center, alpha);
        self.renderer.draw(
            &self.assets,
            &self.world,
            self.mode,
            interpolated_camera,
            self.editor.camera_center,
            self.editor.brush,
            &self.status_text,
            alpha,
            self.replay.status_label(),
            &self.replay.status_detail(),
        );
    }

    fn update_play(&mut self, frame_dt: f32) {
        let live_command = gather_player_command();

        if live_command.restart && self.replay.active_mode() == crate::replay::ReplayMode::Idle {
            self.rebuild_world();
            self.status_text = "Mission restarted.".to_owned();
            return;
        }

        self.handle_replay_hotkeys();
        self.accumulator += frame_dt;

        while self.accumulator >= FIXED_DT {
            let command = self.replay.step_command(live_command);
            self.prev_play_camera_center = self.play_camera_center;
            self.world.update(command, FIXED_DT);
            self.play_camera_center =
                camera::update_play_camera_center(self.play_camera_center, &self.world);
            self.accumulator -= FIXED_DT;

            if self.replay.playback_finished() {
                break;
            }
        }

        if let Some(summary) = self.replay.take_summary() {
            let message = self.replay_summary_text(summary);
            self.pending_run_summary = Some(message.clone());
            self.status_text = message;
            if self.replay.should_autorun_exit() {
                return;
            }
            self.replay.stop();
            return;
        }

        if self.world.mission.victory {
            self.status_text = "All hostages extracted. Press R to run it again.".to_owned();
        } else if self.world.mission.game_over {
            self.status_text = "Out of lives. Press R to restart.".to_owned();
        } else if self.world.rider_count() == self.world.player.rider_capacity {
            self.status_text = "Jeep full. Head for extraction.".to_owned();
        } else {
            self.status_text =
                "Destroy cages, load hostages, extract at the green zone.".to_owned();
        }
    }

    fn update_editor(&mut self, frame_dt: f32) {
        let action = self.editor.update(frame_dt, &mut self.level, &self.world);

        if action.rebuild_world {
            self.rebuild_world();
            self.status_text = "Level updated.".to_owned();
        }

        if action.save {
            match save_level(LEVEL_PATH, &self.level) {
                Ok(()) => self.status_text = format!("Saved {}.", LEVEL_PATH),
                Err(err) => self.status_text = format!("Save failed: {err}"),
            }
        }

        if action.load {
            if let Some(level) = load_level(LEVEL_PATH) {
                self.level = level;
                self.rebuild_world();
                self.status_text = format!("Loaded {}.", LEVEL_PATH);
            } else {
                self.status_text = format!("No level found at {}.", LEVEL_PATH);
            }
        }

        if action.playtest {
            self.rebuild_world();
            self.mode = SceneMode::Play;
            self.accumulator = 0.0;
            self.status_text = "Playtest started.".to_owned();
        }
    }

    fn rebuild_world(&mut self) {
        self.world = World::from_level(&self.level);
        self.play_camera_center = camera::initial_play_camera_center(&self.world);
        self.prev_play_camera_center = self.play_camera_center;
        self.editor.camera_center = self.world.player.pos;
        self.accumulator = 0.0;
    }

    fn toggle_mode(&mut self) {
        match self.mode {
            SceneMode::Play => {
                self.mode = SceneMode::Editor;
                self.editor.camera_center = self.world.player.pos;
                self.status_text = "Editor mode. F5 save, F9 load, Enter playtest.".to_owned();
            }
            SceneMode::Editor => {
                self.rebuild_world();
                self.mode = SceneMode::Play;
                self.accumulator = 0.0;
                self.status_text = "Back to play.".to_owned();
            }
        }
    }

    pub fn should_exit_after_frame(&self) -> bool {
        self.replay.should_autorun_exit()
    }

    pub fn take_run_summary(&mut self) -> Option<String> {
        self.pending_run_summary.take()
    }

    pub fn drain_capture_paths(&mut self) -> Vec<String> {
        self.replay.drain_due_captures()
    }

    fn handle_replay_hotkeys(&mut self) {
        if is_key_pressed(KeyCode::F6) {
            match self.replay.active_mode() {
                crate::replay::ReplayMode::Recording => {
                    match self.replay.stop_and_save(default_replay_path()) {
                        Ok(frames) => {
                            self.status_text = format!(
                                "Saved replay with {frames} frames to {}.",
                                default_replay_path()
                            )
                        }
                        Err(err) => self.status_text = format!("Replay save failed: {err}"),
                    }
                }
                crate::replay::ReplayMode::Idle => {
                    self.rebuild_world();
                    self.replay.start_recording();
                    self.status_text = "Recording replay.".to_owned();
                }
                crate::replay::ReplayMode::Playback => {}
            }
        }

        if is_key_pressed(KeyCode::F7) {
            self.rebuild_world();
            match self
                .replay
                .start_playback_from_path(default_replay_path(), false)
            {
                Ok(()) => self.status_text = format!("Playing back {}.", default_replay_path()),
                Err(err) => self.status_text = format!("Replay load failed: {err}"),
            }
        }

        if is_key_pressed(KeyCode::F8) {
            self.replay.stop();
            self.status_text = "Replay stopped.".to_owned();
        }

        if is_key_pressed(KeyCode::F10) {
            self.rebuild_world();
            self.replay.start_demo(false);
            self.status_text = "Running built-in demo.".to_owned();
        }
    }

    fn replay_summary_text(&self, summary: ReplaySummary) -> String {
        format!(
            "Replay finished: {}  frames:{}  rescued:{}  lives:{}  victory:{}  game_over:{}",
            summary.label,
            summary.frames_played,
            self.world.mission.rescued_total,
            self.world.mission.lives,
            self.world.mission.victory,
            self.world.mission.game_over
        )
    }
}

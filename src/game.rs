use crate::assets::Assets;
use crate::constants::{FIXED_DT, LEVEL_PATH};
use crate::editor::{EditorState, load_level, save_level};
use crate::input::gather_player_command;
use crate::render::{Renderer, camera};
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
}

impl Game {
    pub fn new(assets: Assets) -> Self {
        let level = load_level(LEVEL_PATH).unwrap_or_else(default_level);
        let world = World::from_level(&level);
        let play_camera_center = camera::initial_play_camera_center(&world);
        let editor = EditorState::new(&world);

        Self {
            assets,
            world,
            level,
            renderer: Renderer::new(),
            editor,
            mode: SceneMode::Play,
            prev_play_camera_center: play_camera_center,
            play_camera_center,
            accumulator: 0.0,
            status_text: "Destroy cages, pick up hostages, extract up top.".to_owned(),
        }
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
        );
    }

    fn update_play(&mut self, frame_dt: f32) {
        if gather_player_command().restart {
            self.rebuild_world();
            self.status_text = "Mission restarted.".to_owned();
            return;
        }

        let command = gather_player_command();
        self.accumulator += frame_dt;

        while self.accumulator >= FIXED_DT {
            self.prev_play_camera_center = self.play_camera_center;
            self.world.update(command, FIXED_DT);
            self.play_camera_center =
                camera::update_play_camera_center(self.play_camera_center, &self.world);
            self.accumulator -= FIXED_DT;
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
}

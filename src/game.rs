use crate::assets::Assets;
use crate::constants::FIXED_DT;
use crate::input::gather_player_command;
use crate::render::{Renderer, camera};
use crate::world::{MissionResult, World};
use macroquad::audio::{PlaySoundParams, Sound, play_sound, stop_sound};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMode {
    Splash,
    Loading,
    Playing,
}

pub struct Game {
    splash_screen: Texture2D,
    theme_music: Sound,
    pub assets: Option<Assets>,
    pub world: Option<World>,
    pub renderer: Renderer,
    pub prev_play_camera_center: Vec2,
    pub play_camera_center: Vec2,
    pub accumulator: f32,
    pub show_collision_boxes: bool,
    mission_was_complete: bool,
    mode: AppMode,
    pending_result_sound: Option<MissionResult>,
    active_result_sound: Option<Sound>,
}

impl Game {
    pub fn new(splash_screen: Texture2D, theme_music: Sound, skip_splash: bool) -> Self {
        Self {
            splash_screen,
            theme_music,
            assets: None,
            world: None,
            renderer: Renderer::new(),
            prev_play_camera_center: Vec2::ZERO,
            play_camera_center: Vec2::ZERO,
            accumulator: 0.0,
            show_collision_boxes: false,
            mission_was_complete: false,
            mode: if skip_splash {
                AppMode::Loading
            } else {
                AppMode::Splash
            },
            pending_result_sound: None,
            active_result_sound: None,
        }
    }

    pub fn needs_runtime_load(&self) -> bool {
        self.mode == AppMode::Loading && self.assets.is_none()
    }

    pub fn finish_loading(&mut self, assets: Assets) {
        let world = World::load();
        let play_camera_center = camera::initial_play_camera_center(&world);

        self.mission_was_complete = world.mission_is_complete();
        self.prev_play_camera_center = play_camera_center;
        self.play_camera_center = play_camera_center;
        self.accumulator = 0.0;
        self.assets = Some(assets);
        self.world = Some(world);
        self.mode = AppMode::Playing;
    }

    pub fn take_pending_result_sound(&mut self) -> Option<MissionResult> {
        self.pending_result_sound.take()
    }

    pub fn play_result_sound(&mut self, sound: Sound) {
        play_sound(
            &sound,
            PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
        self.active_result_sound = Some(sound);
    }

    pub fn frame(&mut self, frame_dt: f32) {
        match self.mode {
            AppMode::Splash => {
                if is_key_pressed(KeyCode::Space) {
                    self.mode = AppMode::Loading;
                    self.accumulator = 0.0;
                }
                return;
            }
            AppMode::Loading => return,
            AppMode::Playing => {}
        }

        if is_key_pressed(KeyCode::H) {
            self.show_collision_boxes = !self.show_collision_boxes;
        }

        let world = self
            .world
            .as_mut()
            .expect("game world should be loaded in playing mode");
        let mission_active = !world.mission_is_complete();

        if mission_active && is_key_pressed(KeyCode::R) {
            world.reset_player();
            self.play_camera_center = camera::initial_play_camera_center(world);
            self.prev_play_camera_center = self.play_camera_center;
            self.accumulator = 0.0;
        }

        let live_command = if mission_active {
            gather_player_command()
        } else {
            Default::default()
        };
        self.accumulator += frame_dt;

        while self.accumulator >= FIXED_DT {
            self.prev_play_camera_center = self.play_camera_center;
            world.update(live_command, FIXED_DT);
            self.play_camera_center =
                camera::update_play_camera_center(self.play_camera_center, world);
            self.accumulator -= FIXED_DT;
        }

        if !self.mission_was_complete && world.mission_is_complete() {
            self.mission_was_complete = true;
            stop_sound(&self.theme_music);
            self.pending_result_sound = world.mission_result();
        }
    }

    pub fn draw(&mut self) {
        match self.mode {
            AppMode::Splash => {
                self.renderer.draw_splash(&self.splash_screen, false);
                return;
            }
            AppMode::Loading => {
                self.renderer.draw_splash(&self.splash_screen, true);
                self.accumulator = 0.0;
                return;
            }
            AppMode::Playing => {}
        }

        let assets = self
            .assets
            .as_ref()
            .expect("game assets should be loaded in playing mode");
        let world = self
            .world
            .as_ref()
            .expect("game world should be loaded in playing mode");

        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);
        let interpolated_camera = self
            .prev_play_camera_center
            .lerp(self.play_camera_center, alpha);
        self.renderer.draw(
            assets,
            world,
            interpolated_camera,
            alpha,
            self.show_collision_boxes,
        );
    }
}

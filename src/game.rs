use crate::assets::Assets;
use crate::constants::FIXED_DT;
use crate::input::gather_player_command;
use crate::render::{Renderer, camera};
use crate::world::{MissionResult, World};
use macroquad::audio::{PlaySoundParams, play_sound, stop_sound};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMode {
    Splash,
    Playing,
}

pub struct Game {
    pub assets: Assets,
    pub world: World,
    pub renderer: Renderer,
    pub prev_play_camera_center: Vec2,
    pub play_camera_center: Vec2,
    pub accumulator: f32,
    pub show_collision_boxes: bool,
    mission_was_complete: bool,
    mode: AppMode,
}

impl Game {
    pub fn new(assets: Assets, skip_splash: bool) -> Self {
        let world = World::load();
        let mission_was_complete = world.mission_is_complete();
        let play_camera_center = camera::initial_play_camera_center(&world);
        Self {
            assets,
            world,
            renderer: Renderer::new(),
            prev_play_camera_center: play_camera_center,
            play_camera_center,
            accumulator: 0.0,
            show_collision_boxes: false,
            mission_was_complete,
            mode: if skip_splash {
                AppMode::Playing
            } else {
                AppMode::Splash
            },
        }
    }

    pub fn frame(&mut self, frame_dt: f32) {
        if self.mode == AppMode::Splash {
            if is_key_pressed(KeyCode::Space) {
                self.mode = AppMode::Playing;
                self.accumulator = 0.0;
            }
            return;
        }

        if is_key_pressed(KeyCode::H) {
            self.show_collision_boxes = !self.show_collision_boxes;
        }

        let mission_active = !self.world.mission_is_complete();

        if mission_active && is_key_pressed(KeyCode::R) {
            self.world.reset_player();
            self.play_camera_center = camera::initial_play_camera_center(&self.world);
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
            self.world.update(live_command, FIXED_DT);
            self.play_camera_center =
                camera::update_play_camera_center(self.play_camera_center, &self.world);
            self.accumulator -= FIXED_DT;
        }

        if !self.mission_was_complete && self.world.mission_is_complete() {
            self.mission_was_complete = true;
            stop_sound(self.assets.theme_music());
            let music = match self.world.mission_result() {
                Some(MissionResult::Success) => self.assets.success_music(),
                Some(MissionResult::Failure) => self.assets.failure_music(),
                None => unreachable!(),
            };
            play_sound(
                music,
                PlaySoundParams {
                    looped: false,
                    volume: 0.6,
                },
            );
        }
    }

    pub fn draw(&mut self) {
        if self.mode == AppMode::Splash {
            self.renderer.draw_splash(&self.assets);
            return;
        }

        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);
        let interpolated_camera = self
            .prev_play_camera_center
            .lerp(self.play_camera_center, alpha);
        self.renderer.draw(
            &self.assets,
            &self.world,
            interpolated_camera,
            alpha,
            self.show_collision_boxes,
        );
    }
}

use crate::assets::Assets;
use crate::constants::FIXED_DT;
use crate::input::gather_player_command;
use crate::render::{Renderer, camera};
use crate::world::World;
use macroquad::prelude::*;

pub struct Game {
    pub assets: Assets,
    pub world: World,
    pub renderer: Renderer,
    pub prev_play_camera_center: Vec2,
    pub play_camera_center: Vec2,
    pub accumulator: f32,
}

impl Game {
    pub fn new(assets: Assets) -> Self {
        let world = World::load();
        let play_camera_center = camera::initial_play_camera_center(&world);
        Self {
            assets,
            world,
            renderer: Renderer::new(),
            prev_play_camera_center: play_camera_center,
            play_camera_center,
            accumulator: 0.0,
        }
    }

    pub fn frame(&mut self, frame_dt: f32) {
        if is_key_pressed(KeyCode::R) {
            self.world.reset_player();
            self.play_camera_center = camera::initial_play_camera_center(&self.world);
            self.prev_play_camera_center = self.play_camera_center;
            self.accumulator = 0.0;
        }

        let live_command = gather_player_command();
        self.accumulator += frame_dt;

        while self.accumulator >= FIXED_DT {
            self.prev_play_camera_center = self.play_camera_center;
            self.world.update(live_command, FIXED_DT);
            self.play_camera_center =
                camera::update_play_camera_center(self.play_camera_center, &self.world);
            self.accumulator -= FIXED_DT;
        }
    }

    pub fn draw(&mut self) {
        let alpha = (self.accumulator / FIXED_DT).clamp(0.0, 1.0);
        let interpolated_camera = self
            .prev_play_camera_center
            .lerp(self.play_camera_center, alpha);
        self.renderer
            .draw(&self.assets, &self.world, interpolated_camera, alpha);
    }
}

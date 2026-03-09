pub mod camera;
pub mod hud;
pub mod sprites;

use crate::assets::Assets;
use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::game::SceneMode;
use crate::world::{TileKind, World};
use macroquad::prelude::*;

pub struct Renderer {
    target: RenderTarget,
}

impl Renderer {
    pub fn new() -> Self {
        let target = render_target(VIEW_WIDTH as u32, VIEW_HEIGHT as u32);
        target.texture.set_filter(FilterMode::Nearest);
        Self { target }
    }

    pub fn draw(
        &mut self,
        assets: &Assets,
        world: &World,
        mode: SceneMode,
        play_camera_center: Vec2,
        editor_camera_center: Vec2,
        brush: TileKind,
        status_text: &str,
        alpha: f32,
    ) {
        let camera_center = match mode {
            SceneMode::Play => camera::clamp_camera_center(play_camera_center, world),
            SceneMode::Editor => camera::clamp_camera_center(editor_camera_center, world),
        };
        let top_left = camera_center - vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5);

        set_camera(&Camera2D {
            render_target: Some(self.target.clone()),
            zoom: vec2(2.0 / VIEW_WIDTH, -2.0 / VIEW_HEIGHT),
            target: vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5),
            ..Default::default()
        });
        clear_background(BLACK);
        sprites::draw_world(
            assets,
            world,
            top_left,
            matches!(mode, SceneMode::Editor),
            alpha,
        );
        hud::draw(world, mode, brush, status_text);

        set_default_camera();
        clear_background(BLACK);

        let scale = (screen_width() / VIEW_WIDTH)
            .min(screen_height() / VIEW_HEIGHT)
            .max(0.1);
        let dest = vec2(VIEW_WIDTH * scale, VIEW_HEIGHT * scale);
        let origin = vec2(
            (screen_width() - dest.x) * 0.5,
            (screen_height() - dest.y) * 0.5,
        );

        draw_texture_ex(
            &self.target.texture,
            origin.x,
            origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

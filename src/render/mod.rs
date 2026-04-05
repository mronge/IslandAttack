pub mod camera;
pub mod hud;
pub mod sprites;

use crate::assets::Assets;
use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::world::World;
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
        play_camera_center: Vec2,
        alpha: f32,
        show_collision_boxes: bool,
    ) {
        let camera_center = camera::clamp_camera_center(play_camera_center, world);
        let top_left = camera_center - vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5);

        set_camera(&Camera2D {
            render_target: Some(self.target.clone()),
            zoom: vec2(2.0 / VIEW_WIDTH, -2.0 / VIEW_HEIGHT),
            target: vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5),
            ..Default::default()
        });
        clear_background(BLACK);
        sprites::draw_world(assets, world, top_left, alpha);
        if show_collision_boxes {
            sprites::draw_collision_boxes(world, top_left, alpha);
        }

        set_default_camera();
        clear_background(BLACK);

        let (origin, dest, scale) = presentation_layout();

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

        hud::draw(world, origin, dest, scale, is_key_down(KeyCode::Tab));
    }

    pub fn draw_splash(&mut self, assets: &Assets) {
        set_default_camera();
        clear_background(BLACK);

        let (origin, dest, scale) = presentation_layout();
        draw_texture_ex(
            assets.splash_screen(),
            origin.x,
            origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest),
                ..Default::default()
            },
        );

        let prompt = "Press space to begin";
        let font_size = (12.0 * scale).max(1.0);
        let measured = measure_text(prompt, None, font_size as u16, 1.0);
        let x = origin.x + dest.x * 0.5 - measured.width * 0.5;
        let y = origin.y + dest.y - 16.0 * scale;
        let pulse = ((get_time() as f32 * 2.4).sin() * 0.5 + 0.5).powf(2.2);
        if pulse < 0.14 {
            return;
        }
        draw_text(
            prompt,
            x + 1.0 * scale,
            y + 1.0 * scale,
            font_size,
            Color::new(0.0, 0.0, 0.0, pulse * 0.55),
        );
        draw_text(prompt, x, y, font_size, Color::new(1.0, 1.0, 1.0, pulse));
    }
}

fn presentation_layout() -> (Vec2, Vec2, f32) {
    let scale = (screen_width() / VIEW_WIDTH)
        .min(screen_height() / VIEW_HEIGHT)
        .floor()
        .max(1.0);
    let dest = vec2(VIEW_WIDTH * scale, VIEW_HEIGHT * scale);
    let origin = vec2(
        (screen_width() - dest.x) * 0.5,
        (screen_height() - dest.y) * 0.5,
    );
    (origin, dest, scale)
}

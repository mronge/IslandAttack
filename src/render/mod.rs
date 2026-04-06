pub mod camera;
pub mod hud;
pub mod sprites;

use crate::assets::Assets;
use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::world::World;
use macroquad::prelude::*;
use macroquad::texture::{RenderTargetParams, render_target_ex};

pub struct Renderer {
    target: Option<RenderTarget>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { target: None }
    }

    fn target(&mut self) -> &RenderTarget {
        self.target.get_or_insert_with(|| {
            // WORKAROUND: macroquad 0.4.14 bug — render_target_ex checks
            // `sample_count != 0` to decide whether to create MSAA resolve
            // framebuffers, but the default sample_count of 1 (no MSAA)
            // still triggers that path. On WebGL the resolve framebuffer
            // creation fails (glCheckFramebufferStatus returns 0), panicking
            // inside miniquad. Using 0 skips the resolve path entirely.
            // See: miniquad 0.4.8 src/graphics/gl.rs:1144
            let target = render_target_ex(
                VIEW_WIDTH as u32,
                VIEW_HEIGHT as u32,
                RenderTargetParams {
                    sample_count: 0,
                    ..Default::default()
                },
            );
            target.texture.set_filter(FilterMode::Nearest);
            target
        })
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
            render_target: Some(self.target().clone()),
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
            &self.target().texture,
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

    pub fn draw_splash(&mut self, splash_screen: &Texture2D, loading: bool) {
        set_default_camera();
        clear_background(BLACK);

        let (origin, dest, scale) = presentation_layout();
        draw_texture_ex(
            splash_screen,
            origin.x,
            origin.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest),
                ..Default::default()
            },
        );

        let prompt = if loading {
            "Loading..."
        } else {
            "Press space to begin"
        };
        let font_size = (12.0 * scale).max(1.0);
        let measured = measure_text(prompt, None, font_size as u16, 1.0);
        let x = origin.x + dest.x * 0.5 - measured.width * 0.5;
        let y = origin.y + dest.y - 16.0 * scale;
        let pulse = if loading {
            1.0
        } else {
            ((get_time() as f32 * 2.4).sin() * 0.5 + 0.5).powf(2.2)
        };
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

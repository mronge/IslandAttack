use crate::world::World;
use macroquad::prelude::*;

pub fn draw(world: &World, origin: Vec2, dest: Vec2, scale: f32) {
    draw_top_bar(world, origin, scale);

    draw_text(
        "WASD / ARROWS MOVE   R RESET",
        origin.x + 18.0 * scale,
        origin.y + dest.y - 18.0 * scale,
        24.0 * scale,
        WHITE,
    );
}

fn draw_top_bar(world: &World, origin: Vec2, scale: f32) {
    let panel_pos = origin + vec2(10.0, 10.0) * scale;
    let panel_size = vec2(144.0, 24.0) * scale;
    draw_rectangle(
        panel_pos.x,
        panel_pos.y,
        panel_size.x,
        panel_size.y,
        Color::new(0.02, 0.03, 0.05, 0.82),
    );
    draw_rectangle_lines(
        panel_pos.x,
        panel_pos.y,
        panel_size.x,
        panel_size.y,
        2.0 * scale,
        Color::new(1.0, 1.0, 1.0, 0.08),
    );

    let bar_pos = panel_pos + vec2(29.0, 6.0) * scale;
    let bar_size = vec2(103.0, 12.0) * scale;
    let heart_center = vec2(
        panel_pos.x + 16.0 * scale,
        bar_pos.y + bar_size.y * 0.5 - 0.5 * scale,
    );
    draw_heart(heart_center, 5.5 * scale, color_u8!(220, 62, 74, 255));

    draw_rectangle(
        bar_pos.x,
        bar_pos.y,
        bar_size.x,
        bar_size.y,
        color_u8!(48, 16, 20, 255),
    );

    let ratio = if world.player.max_hp > 0 {
        world.player.hp as f32 / world.player.max_hp as f32
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    if ratio > 0.0 {
        draw_rectangle(
            bar_pos.x + 2.0 * scale,
            bar_pos.y + 2.0 * scale,
            (bar_size.x - 4.0 * scale) * ratio,
            bar_size.y - 4.0 * scale,
            color_u8!(232, 84, 96, 255),
        );
    }

    draw_rectangle_lines(
        bar_pos.x,
        bar_pos.y,
        bar_size.x,
        bar_size.y,
        2.0 * scale,
        color_u8!(255, 220, 220, 255),
    );

    let label = format!("{}/{}", world.player.hp, world.player.max_hp);
    let font_size = (10.0 * scale).max(1.0);
    let measured = measure_text(&label, None, font_size as u16, 1.0);
    draw_text(
        &label,
        bar_pos.x + bar_size.x * 0.5 - measured.width * 0.5,
        bar_pos.y + bar_size.y * 0.5 - measured.height * 0.5 + measured.offset_y,
        font_size,
        WHITE,
    );
}

fn draw_heart(center: Vec2, size: f32, color: Color) {
    let lobe_radius = size * 0.55;
    draw_circle(
        center.x - size * 0.42,
        center.y - size * 0.18,
        lobe_radius,
        color,
    );
    draw_circle(
        center.x + size * 0.42,
        center.y - size * 0.18,
        lobe_radius,
        color,
    );
    draw_triangle(
        vec2(center.x - size * 1.02, center.y + size * 0.04),
        vec2(center.x + size * 1.02, center.y + size * 0.04),
        vec2(center.x, center.y + size * 1.34),
        color,
    );
}

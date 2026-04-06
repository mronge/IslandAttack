use crate::world::{MissionResult, World};
use macroquad::prelude::*;

pub fn draw(world: &World, origin: Vec2, dest: Vec2, scale: f32, show_controls: bool) {
    draw_top_bar(world, origin, scale);
    draw_mission_overlay(world, origin, dest, scale);

    if show_controls {
        draw_text(
            "WASD / ARROWS MOVE   H HITBOXES   R RESET",
            origin.x + 18.0 * scale,
            origin.y + dest.y - 18.0 * scale,
            24.0 * scale,
            WHITE,
        );
    }
}

fn draw_top_bar(world: &World, origin: Vec2, scale: f32) {
    let panel_pos = origin + vec2(10.0, 10.0) * scale;
    let panel_size = vec2(179.2, 24.0) * scale;
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

    let pow_label = format!("POW {}", world.rescued_pows);
    let pow_font_size = (11.0 * scale).max(1.0);
    let pow_pos = panel_pos + vec2(138.0, 15.5) * scale;
    draw_text(
        &pow_label,
        pow_pos.x + 1.0 * scale,
        pow_pos.y + 1.0 * scale,
        pow_font_size,
        Color::new(0.0, 0.0, 0.0, 0.45),
    );
    draw_text(&pow_label, pow_pos.x, pow_pos.y, pow_font_size, WHITE);
}

fn draw_mission_overlay(world: &World, origin: Vec2, dest: Vec2, scale: f32) {
    let Some(result) = world.mission_result() else {
        return;
    };

    let panel_size = if result == MissionResult::Failure {
        vec2(256.0, 106.0) * scale
    } else {
        vec2(256.0, 86.0) * scale
    };
    let panel_pos = origin + (dest - panel_size) * 0.5;
    draw_rectangle(
        panel_pos.x,
        panel_pos.y,
        panel_size.x,
        panel_size.y,
        Color::new(0.03, 0.04, 0.06, 0.9),
    );
    draw_rectangle_lines(
        panel_pos.x,
        panel_pos.y,
        panel_size.x,
        panel_size.y,
        2.0 * scale,
        Color::new(1.0, 1.0, 1.0, 0.12),
    );

    let title = match result {
        MissionResult::Success => "MISSION SUCCESS",
        MissionResult::Failure => "MISSION FAILURE",
    };
    let progress = format!("{}/{}", world.rescued_pows, world.total_pows);

    let title_font_size = (20.0 * scale).max(1.0);
    let title_metrics = measure_text(title, None, title_font_size as u16, 1.0);
    let title_pos = vec2(
        panel_pos.x + panel_size.x * 0.5 - title_metrics.width * 0.5,
        panel_pos.y + 33.0 * scale,
    );
    draw_text(
        title,
        title_pos.x + 1.0 * scale,
        title_pos.y + 1.0 * scale,
        title_font_size,
        Color::new(0.0, 0.0, 0.0, 0.45),
    );
    draw_text(title, title_pos.x, title_pos.y, title_font_size, WHITE);

    let progress_font_size = (15.0 * scale).max(1.0);
    let progress_metrics = measure_text(&progress, None, progress_font_size as u16, 1.0);
    let progress_pos = vec2(
        panel_pos.x + panel_size.x * 0.5 - progress_metrics.width * 0.5,
        panel_pos.y + 59.0 * scale,
    );
    draw_text(
        &progress,
        progress_pos.x + 1.0 * scale,
        progress_pos.y + 1.0 * scale,
        progress_font_size,
        Color::new(0.0, 0.0, 0.0, 0.45),
    );
    draw_text(
        &progress,
        progress_pos.x,
        progress_pos.y,
        progress_font_size,
        WHITE,
    );

    if result == MissionResult::Failure {
        draw_retry_prompt(panel_pos, panel_size, scale);
    }
}

fn draw_retry_prompt(panel_pos: Vec2, panel_size: Vec2, scale: f32) {
    let prompt = "Press R to retry";
    let font_size = (12.0 * scale).max(1.0);
    let measured = measure_text(prompt, None, font_size as u16, 1.0);
    let x = panel_pos.x + panel_size.x * 0.5 - measured.width * 0.5;
    let y = panel_pos.y + panel_size.y - 15.0 * scale;
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

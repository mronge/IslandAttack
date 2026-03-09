use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::game::SceneMode;
use crate::world::{TileKind, World};
use macroquad::prelude::*;

pub fn draw(world: &World, mode: SceneMode, brush: TileKind, status_text: &str) {
    let mode_label = match mode {
        SceneMode::Play => "PLAY",
        SceneMode::Editor => "EDITOR",
    };

    draw_rectangle(0.0, 0.0, VIEW_WIDTH, 92.0, Color::new(0.0, 0.0, 0.0, 0.72));
    draw_rectangle(
        0.0,
        VIEW_HEIGHT - 44.0,
        VIEW_WIDTH,
        44.0,
        Color::new(0.0, 0.0, 0.0, 0.72),
    );

    draw_text(
        &format!(
            "{}  LIVES:{}  HP:{}  RESCUED:{}/{}  RIDERS:{}",
            mode_label,
            world.mission.lives,
            world.player.hp,
            world.mission.rescued_total,
            world.mission.total_hostages,
            world.rider_count()
        ),
        24.0,
        38.0,
        38.0,
        WHITE,
    );

    draw_text(status_text, 24.0, 72.0, 28.0, color_u8!(220, 220, 180, 255));

    match mode {
        SceneMode::Play => {
            draw_text(
                "WASD/ARROWS MOVE  SPACE FIRE  TAB EDITOR  R RESET",
                24.0,
                VIEW_HEIGHT - 14.0,
                22.0,
                WHITE,
            );
        }
        SceneMode::Editor => {
            draw_text(
                &format!(
                    "ARROWS PAN  LMB PAINT  RMB GRASS  1-8 BRUSH  F5 SAVE  F9 LOAD  ENTER PLAYTEST  BRUSH:{:?}",
                    brush
                ),
                24.0,
                VIEW_HEIGHT - 14.0,
                22.0,
                WHITE,
            );
        }
    }

    if world.mission.victory {
        draw_centered_message("MISSION CLEAR", color_u8!(220, 255, 200, 255));
    } else if world.mission.game_over {
        draw_centered_message("GAME OVER", color_u8!(255, 170, 170, 255));
    }
}

fn draw_centered_message(text: &str, color: Color) {
    let size = 64.0;
    let measured = measure_text(text, None, size as u16, 1.0);
    draw_rectangle(
        VIEW_WIDTH * 0.5 - measured.width * 0.5 - 24.0,
        VIEW_HEIGHT * 0.5 - 56.0,
        measured.width + 48.0,
        82.0,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    draw_text(
        text,
        VIEW_WIDTH * 0.5 - measured.width * 0.5,
        VIEW_HEIGHT * 0.5,
        size,
        color,
    );
}

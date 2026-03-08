use crate::game::SceneMode;
use crate::world::{TileKind, World};
use macroquad::prelude::*;

pub fn draw(world: &World, mode: SceneMode, brush: TileKind, status_text: &str) {
    let mode_label = match mode {
        SceneMode::Play => "PLAY",
        SceneMode::Editor => "EDITOR",
    };

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
        8.0,
        12.0,
        16.0,
        WHITE,
    );

    draw_text(status_text, 8.0, 28.0, 14.0, color_u8!(220, 220, 180, 255));

    match mode {
        SceneMode::Play => {
            draw_text(
                "WASD/ARROWS MOVE  SPACE FIRE  TAB EDITOR  R RESET",
                8.0,
                172.0,
                12.0,
                WHITE,
            );
        }
        SceneMode::Editor => {
            draw_text(
                &format!(
                    "ARROWS PAN  LMB PAINT  RMB GRASS  1-8 BRUSH  F5 SAVE  F9 LOAD  ENTER PLAYTEST  BRUSH:{:?}",
                    brush
                ),
                8.0,
                172.0,
                12.0,
                WHITE,
            );
        }
    }

    if world.mission.victory {
        draw_text(
            "MISSION CLEAR",
            104.0,
            88.0,
            24.0,
            color_u8!(220, 255, 200, 255),
        );
    } else if world.mission.game_over {
        draw_text(
            "GAME OVER",
            112.0,
            88.0,
            24.0,
            color_u8!(255, 170, 170, 255),
        );
    }
}

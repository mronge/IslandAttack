use crate::entities::Direction;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PlayerCommand {
    pub move_dir: Option<Direction>,
}

pub fn gather_player_command() -> PlayerCommand {
    let move_dir = if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        Some(Direction::Up)
    } else if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        Some(Direction::Down)
    } else if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        Some(Direction::Left)
    } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        Some(Direction::Right)
    } else {
        None
    };

    PlayerCommand {
        move_dir,
    }
}

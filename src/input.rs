use crate::entities::Direction;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerCommand {
    pub move_dir: Option<Direction>,
    pub fire: bool,
    pub restart: bool,
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
        fire: is_key_down(KeyCode::Space),
        restart: is_key_pressed(KeyCode::R),
    }
}

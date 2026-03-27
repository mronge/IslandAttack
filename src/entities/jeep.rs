use crate::constants::{JEEP_COLLISION_SCALE, JEEP_SIZE, JEEP_SPEED, PLAYER_MAX_HP};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_vec(self) -> Vec2 {
        match self {
            Self::Up => vec2(0.0, -1.0),
            Self::Down => vec2(0.0, 1.0),
            Self::Left => vec2(-1.0, 0.0),
            Self::Right => vec2(1.0, 0.0),
        }
    }

    pub fn from_vec(delta: Vec2) -> Self {
        if delta.x.abs() > delta.y.abs() {
            if delta.x >= 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if delta.y >= 0.0 {
            Self::Down
        } else {
            Self::Up
        }
    }
}

#[derive(Clone, Debug)]
pub struct Jeep {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub dir: Direction,
    pub speed: f32,
    pub fire_cooldown: f32,
    pub hp: i32,
    pub max_hp: i32,
}

impl Jeep {
    pub fn new(pos: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel: Vec2::ZERO,
            dir: Direction::Up,
            speed: JEEP_SPEED,
            fire_cooldown: 0.0,
            hp: PLAYER_MAX_HP,
            max_hp: PLAYER_MAX_HP,
        }
    }

    pub fn render_size(&self) -> Vec2 {
        vec2(JEEP_SIZE, JEEP_SIZE)
    }

    pub fn size(&self) -> Vec2 {
        self.render_size() * JEEP_COLLISION_SCALE
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

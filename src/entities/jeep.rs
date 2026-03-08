use crate::constants::JEEP_SPEED;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

#[derive(Clone, Debug)]
pub struct Jeep {
    pub pos: Vec2,
    pub dir: Direction,
    pub speed: f32,
    pub fire_cooldown: f32,
    pub hp: i32,
    pub invuln_timer: f32,
    pub rider_capacity: usize,
}

impl Jeep {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            dir: Direction::Up,
            speed: JEEP_SPEED,
            fire_cooldown: 0.0,
            hp: 3,
            invuln_timer: 0.0,
            rider_capacity: 4,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(12.0, 12.0)
    }
}

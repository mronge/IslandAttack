use crate::constants::ENEMY_SPEED;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyState {
    Idle,
    Chase,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub hp: i32,
    pub speed: f32,
    pub state: EnemyState,
}

impl Enemy {
    pub fn new(pos: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            hp: 2,
            speed: ENEMY_SPEED,
            state: EnemyState::Idle,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(52.0, 52.0)
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

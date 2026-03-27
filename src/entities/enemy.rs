use crate::constants::ENEMY_SPEED;
use crate::entities::Direction;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Soldier,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub dir: Direction,
    pub kind: EnemyKind,
    pub hp: i32,
    pub speed: f32,
}

impl Enemy {
    pub fn new(pos: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            dir: Direction::Down,
            kind: EnemyKind::Soldier,
            hp: 2,
            speed: ENEMY_SPEED,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(16.0, 16.0)
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

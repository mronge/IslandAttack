use crate::constants::{ENEMY_SPEED, ROCKETEER_FIRE_COOLDOWN};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Commando,
    Rocketeer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyState {
    Idle,
    Chase,
    Attack,
    Retreat,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub kind: EnemyKind,
    pub hp: i32,
    pub speed: f32,
    pub state: EnemyState,
    pub fire_cooldown: f32,
}

impl Enemy {
    pub fn new(pos: Vec2, kind: EnemyKind) -> Self {
        Self {
            prev_pos: pos,
            pos,
            kind,
            hp: 2,
            speed: ENEMY_SPEED,
            state: EnemyState::Idle,
            fire_cooldown: if kind == EnemyKind::Rocketeer {
                ROCKETEER_FIRE_COOLDOWN * 0.45
            } else {
                0.0
            },
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(52.0, 52.0)
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

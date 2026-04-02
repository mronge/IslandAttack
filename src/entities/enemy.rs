use crate::constants::{ENEMY_FIRE_COOLDOWN, ENEMY_SPEED, ENEMY_WALK_FRAME_TIME};
use crate::entities::Direction;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Soldier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyAnimState {
    Idle,
    Walk,
    Shoot,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub dir: Direction,
    pub kind: EnemyKind,
    pub hp: i32,
    pub speed: f32,
    pub fire_cooldown: f32,
    pub shoot_timer: f32,
    pub animation_state: EnemyAnimState,
    pub animation_timer: f32,
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
            fire_cooldown: ENEMY_FIRE_COOLDOWN * 0.5,
            shoot_timer: 0.0,
            animation_state: EnemyAnimState::Idle,
            animation_timer: 0.0,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(16.0, 16.0)
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }

    pub fn set_animation_state(&mut self, state: EnemyAnimState) {
        if self.animation_state != state {
            self.animation_state = state;
            self.animation_timer = 0.0;
        }
    }

    pub fn tick_animation(&mut self, dt: f32) {
        self.animation_timer += dt;
    }

    pub fn walk_frame_index(&self) -> usize {
        (self.animation_timer / ENEMY_WALK_FRAME_TIME) as usize
    }
}

use crate::constants::{
    ENEMY_WALK_FRAME_TIME, POW_ESCAPE_DISTANCE, POW_RENDER_SIZE, POW_SIZE, POW_SPEED,
};
use crate::entities::{ActorAnimState, Direction};
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Pow {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub dir: Direction,
    pub speed: f32,
    pub animation_state: ActorAnimState,
    pub animation_timer: f32,
    pub escape_origin: Vec2,
    pub escape_dir: Vec2,
    pub boarded: bool,
}

impl Pow {
    pub fn new(pos: Vec2, escape_dir: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            dir: Direction::from_vec(escape_dir),
            speed: POW_SPEED,
            animation_state: ActorAnimState::Idle,
            animation_timer: 0.0,
            escape_origin: pos,
            escape_dir: escape_dir.normalize(),
            boarded: false,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(POW_SIZE, POW_SIZE)
    }

    pub fn render_size(&self) -> Vec2 {
        vec2(POW_RENDER_SIZE, POW_RENDER_SIZE)
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }

    pub fn has_cleared_barracks(&self) -> bool {
        self.pos.distance_squared(self.escape_origin) >= POW_ESCAPE_DISTANCE * POW_ESCAPE_DISTANCE
    }

    pub fn set_animation_state(&mut self, state: ActorAnimState) {
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

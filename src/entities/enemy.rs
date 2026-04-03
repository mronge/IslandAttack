use crate::constants::{
    ENEMY_BULLET_DAMAGE, ENEMY_BULLET_RADIUS, ENEMY_BULLET_SPEED, ENEMY_FIRE_COOLDOWN,
    ENEMY_FIRE_RANGE, ENEMY_SHOOT_DURATION, ENEMY_SPEED, ENEMY_WALK_FRAME_TIME,
    SOLDIER_RENDER_SIZE, TURRET_BULLET_DAMAGE, TURRET_BULLET_RADIUS, TURRET_BULLET_SPEED,
    TURRET_FIRE_COOLDOWN, TURRET_FIRE_RANGE, TURRET_HP, TURRET_RENDER_SIZE, TURRET_SIZE,
};
use crate::entities::Direction;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyState {
    Active,
    Destroyed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Soldier,
    Turret,
}

impl EnemyKind {
    pub fn max_per_tile(self) -> usize {
        match self {
            // Keep the cap with the enemy kind so later variants can tighten it
            // without scattering occupancy rules across movement code.
            Self::Soldier => 2,
            Self::Turret => 1,
        }
    }

    pub fn hp(self) -> i32 {
        match self {
            Self::Soldier => 2,
            Self::Turret => TURRET_HP,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_SPEED,
            Self::Turret => 0.0,
        }
    }

    pub fn size(self) -> Vec2 {
        match self {
            Self::Soldier => vec2(16.0, 16.0),
            Self::Turret => vec2(TURRET_SIZE, TURRET_SIZE),
        }
    }

    pub fn render_size(self) -> Vec2 {
        match self {
            Self::Soldier => vec2(SOLDIER_RENDER_SIZE, SOLDIER_RENDER_SIZE),
            Self::Turret => vec2(TURRET_RENDER_SIZE, TURRET_RENDER_SIZE),
        }
    }

    pub fn fire_cooldown(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_FIRE_COOLDOWN,
            Self::Turret => TURRET_FIRE_COOLDOWN,
        }
    }

    pub fn fire_range(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_FIRE_RANGE,
            Self::Turret => TURRET_FIRE_RANGE,
        }
    }

    pub fn shoot_duration(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_SHOOT_DURATION,
            Self::Turret => ENEMY_SHOOT_DURATION,
        }
    }

    pub fn bullet_speed(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_BULLET_SPEED,
            Self::Turret => TURRET_BULLET_SPEED,
        }
    }

    pub fn bullet_damage(self) -> i32 {
        match self {
            Self::Soldier => ENEMY_BULLET_DAMAGE,
            Self::Turret => TURRET_BULLET_DAMAGE,
        }
    }

    pub fn bullet_radius(self) -> f32 {
        match self {
            Self::Soldier => ENEMY_BULLET_RADIUS,
            Self::Turret => TURRET_BULLET_RADIUS,
        }
    }

    pub fn is_stationary(self) -> bool {
        matches!(self, Self::Turret)
    }
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
    pub state: EnemyState,
    pub hp: i32,
    pub speed: f32,
    pub fire_cooldown: f32,
    pub shoot_timer: f32,
    pub animation_state: EnemyAnimState,
    pub animation_timer: f32,
}

impl Enemy {
    pub fn new(pos: Vec2) -> Self {
        Self::new_with_kind(pos, EnemyKind::Soldier)
    }

    pub fn new_with_kind(pos: Vec2, kind: EnemyKind) -> Self {
        Self {
            prev_pos: pos,
            pos,
            dir: Direction::Down,
            kind,
            state: EnemyState::Active,
            hp: kind.hp(),
            speed: kind.speed(),
            fire_cooldown: kind.fire_cooldown() * 0.5,
            shoot_timer: 0.0,
            animation_state: EnemyAnimState::Idle,
            animation_timer: 0.0,
        }
    }

    pub fn size(&self) -> Vec2 {
        self.kind.size()
    }

    pub fn render_size(&self) -> Vec2 {
        self.kind.render_size()
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }

    pub fn is_destroyed(&self) -> bool {
        self.state == EnemyState::Destroyed
    }

    pub fn can_act(&self) -> bool {
        self.state == EnemyState::Active && self.hp > 0
    }

    pub fn destroy(&mut self) {
        self.state = EnemyState::Destroyed;
        self.hp = 0;
        self.shoot_timer = 0.0;
        self.fire_cooldown = 0.0;
        self.set_animation_state(EnemyAnimState::Idle);
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

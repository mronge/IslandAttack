use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulletKind {
    Normal,
    Rocket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulletOwner {
    Player,
    Enemy,
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
    pub radius: f32,
    pub damage: u8,
    pub owner: BulletOwner,
    pub kind: BulletKind,
}

impl Bullet {
    pub fn player(pos: Vec2, vel: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel,
            ttl: 1.5,
            radius: 2.5,
            damage: 1,
            owner: BulletOwner::Player,
            kind: BulletKind::Normal,
        }
    }

    pub fn enemy(pos: Vec2, vel: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel,
            ttl: 1.8,
            radius: 2.5,
            damage: 1,
            owner: BulletOwner::Enemy,
            kind: BulletKind::Normal,
        }
    }

    pub fn rocket(pos: Vec2, vel: Vec2, damage: u8) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel,
            ttl: 2.2,
            radius: 4.5,
            damage,
            owner: BulletOwner::Enemy,
            kind: BulletKind::Rocket,
        }
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

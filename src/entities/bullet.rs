use macroquad::prelude::*;

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
    pub damage: i32,
    pub owner: BulletOwner,
}

impl Bullet {
    pub fn new(pos: Vec2, vel: Vec2, owner: BulletOwner) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel,
            ttl: 1.5,
            radius: 2.5,
            damage: 1,
            owner,
        }
    }

    pub fn with_damage(mut self, damage: i32) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

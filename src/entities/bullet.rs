use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
    pub radius: f32,
    pub damage: u8,
}

impl Bullet {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
        Self {
            pos,
            vel,
            ttl: 1.2,
            radius: 2.0,
            damage: 1,
        }
    }
}

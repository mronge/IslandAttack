use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Bullet {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
    pub radius: f32,
    pub damage: u8,
}

impl Bullet {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            vel,
            ttl: 1.5,
            radius: 10.0,
            damage: 1,
        }
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Explosion {
    pub pos: Vec2,
    pub timer: f32,
}

impl Explosion {
    pub fn new(pos: Vec2) -> Self {
        Self { pos, timer: 0.35 }
    }
}

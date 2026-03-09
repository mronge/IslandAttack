use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct Turret {
    pub pos: Vec2,
    pub home_tile: IVec2,
    pub hp: i32,
    pub fire_cooldown: f32,
}

impl Turret {
    pub fn new(pos: Vec2, home_tile: IVec2) -> Self {
        Self {
            pos,
            home_tile,
            hp: 3,
            fire_cooldown: 0.2,
        }
    }
}

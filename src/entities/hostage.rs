use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostageState {
    Captive,
    RunningToJeep,
    Riding { slot: usize },
    Rescued,
}

#[derive(Clone, Debug)]
pub struct Hostage {
    pub prev_pos: Vec2,
    pub pos: Vec2,
    pub home_tile: IVec2,
    pub state: HostageState,
    pub speed: f32,
}

impl Hostage {
    pub fn new(pos: Vec2, home_tile: IVec2) -> Self {
        Self {
            prev_pos: pos,
            pos,
            home_tile,
            state: HostageState::Captive,
            speed: 80.0,
        }
    }

    pub fn render_pos(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

pub fn rider_offset(slot: usize) -> Vec2 {
    match slot {
        0 => vec2(-11.0, 11.0),
        1 => vec2(11.0, 11.0),
        2 => vec2(-11.0, 22.0),
        3 => vec2(11.0, 22.0),
        _ => vec2(0.0, 11.0 + (slot as f32 * 5.0)),
    }
}

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
    pub pos: Vec2,
    pub home_tile: IVec2,
    pub state: HostageState,
    pub speed: f32,
}

impl Hostage {
    pub fn new(pos: Vec2, home_tile: IVec2) -> Self {
        Self {
            pos,
            home_tile,
            state: HostageState::Captive,
            speed: 44.0,
        }
    }
}

pub fn rider_offset(slot: usize) -> Vec2 {
    match slot {
        0 => vec2(-6.0, 10.0),
        1 => vec2(6.0, 10.0),
        2 => vec2(-6.0, 18.0),
        3 => vec2(6.0, 18.0),
        _ => vec2(0.0, 10.0 + (slot as f32 * 4.0)),
    }
}

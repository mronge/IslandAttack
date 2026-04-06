use crate::constants::{BARRACKS_HP, BARRACKS_SIZE};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarracksState {
    Active,
    Destroyed,
}

#[derive(Clone, Debug)]
pub struct Barracks {
    pub pos: Vec2,
    pub state: BarracksState,
    pub hp: i32,
    pub released_pows: bool,
}

impl Barracks {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            state: BarracksState::Active,
            hp: BARRACKS_HP,
            released_pows: false,
        }
    }

    pub fn size(&self) -> Vec2 {
        vec2(BARRACKS_SIZE, BARRACKS_SIZE)
    }

    pub fn render_size(&self) -> Vec2 {
        self.size()
    }

    pub fn is_destroyed(&self) -> bool {
        self.state == BarracksState::Destroyed
    }

    pub fn can_take_damage(&self) -> bool {
        self.state == BarracksState::Active && self.hp > 0
    }

    pub fn destroy(&mut self) {
        self.state = BarracksState::Destroyed;
        self.hp = 0;
    }

    pub fn can_release_pows(&self) -> bool {
        self.is_destroyed() && !self.released_pows
    }

    pub fn mark_pows_released(&mut self) {
        self.released_pows = true;
    }
}

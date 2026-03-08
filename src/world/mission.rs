use crate::constants::EXTRA_LIFE_EVERY;

#[derive(Clone, Debug)]
pub struct MissionState {
    pub lives: u8,
    pub rescued_total: u32,
    pub next_extra_life_at: u32,
    pub total_hostages: u32,
    pub game_over: bool,
    pub victory: bool,
}

impl MissionState {
    pub fn new(total_hostages: u32) -> Self {
        Self {
            lives: 3,
            rescued_total: 0,
            next_extra_life_at: EXTRA_LIFE_EVERY,
            total_hostages,
            game_over: false,
            victory: false,
        }
    }
}

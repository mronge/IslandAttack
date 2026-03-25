use crate::entities::Jeep;
use crate::world::ImportedMap;
use macroquad::prelude::*;

pub struct World {
    pub map: ImportedMap,
    pub player: Jeep,
    pub player_spawn: Vec2,
}

impl World {
    pub fn load() -> Self {
        let map = ImportedMap::load();
        let probe = Jeep::new(Vec2::ZERO);
        let player_spawn = map.default_spawn_point_for(probe.size());
        Self {
            map,
            player: Jeep::new(player_spawn),
            player_spawn,
        }
    }

    pub fn reset_player(&mut self) {
        self.player = Jeep::new(self.player_spawn);
    }

    pub fn snapshot_positions(&mut self) {
        self.player.prev_pos = self.player.pos;
    }
}

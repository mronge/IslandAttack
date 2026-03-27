use crate::entities::{Bullet, Enemy, Jeep};
use crate::world::ImportedMap;
use macroquad::prelude::*;

pub struct World {
    pub map: ImportedMap,
    pub player: Jeep,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub player_spawn: Vec2,
}

impl World {
    pub fn load() -> Self {
        let map = ImportedMap::load();
        let probe = Jeep::new(Vec2::ZERO);
        let player_spawn = map.default_spawn_point_for(probe.size());
        let enemies = map
            .enemy_spawn_points(vec2(16.0, 16.0), 6, player_spawn, 5.0 * map.tile_size)
            .into_iter()
            .map(Enemy::new)
            .collect();
        Self {
            map,
            player: Jeep::new(player_spawn),
            enemies,
            bullets: Vec::new(),
            player_spawn,
        }
    }

    pub fn reset_player(&mut self) {
        self.player = Jeep::new(self.player_spawn);
    }

    pub fn snapshot_positions(&mut self) {
        self.player.prev_pos = self.player.pos;
        for enemy in &mut self.enemies {
            enemy.prev_pos = enemy.pos;
        }
        for bullet in &mut self.bullets {
            bullet.prev_pos = bullet.pos;
        }
    }
}

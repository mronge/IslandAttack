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
        let mut enemies = Vec::new();

        for spawn in map.enemy_spawns() {
            let center = map.tile_center(spawn.tile);
            let rect = Rect::new(
                center.x - spawn.kind.size().x * 0.5,
                center.y - spawn.kind.size().y * 0.5,
                spawn.kind.size().x,
                spawn.kind.size().y,
            );
            assert!(
                !map.collides_rect(rect),
                "enemy spawn at tile ({}, {}) collides with the map",
                spawn.tile.x,
                spawn.tile.y
            );

            for _ in 0..spawn.count {
                enemies.push(Enemy::new_with_kind(center, spawn.kind));
            }
        }

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

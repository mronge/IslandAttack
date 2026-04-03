use crate::constants::{INITIAL_SOLDIER_COUNT, INITIAL_TURRET_COUNT};
use crate::entities::{Bullet, Enemy, EnemyKind, Jeep};
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
        let enemy_spawn_points = map
            .enemy_spawn_points(
                vec2(16.0, 16.0),
                INITIAL_SOLDIER_COUNT + INITIAL_TURRET_COUNT,
                player_spawn,
                5.0 * map.tile_size,
            )
            .into_iter()
            .collect::<Vec<_>>();
        let mut enemies = Vec::with_capacity(enemy_spawn_points.len());

        // Reserve the farthest spawn points for fixed turrets so they pressure
        // the player from strong positions without crowding the opening area.
        for pos in enemy_spawn_points.iter().take(INITIAL_TURRET_COUNT) {
            enemies.push(Enemy::new_with_kind(*pos, EnemyKind::Turret));
        }
        for pos in enemy_spawn_points
            .iter()
            .skip(INITIAL_TURRET_COUNT)
            .take(INITIAL_SOLDIER_COUNT)
        {
            enemies.push(Enemy::new_with_kind(*pos, EnemyKind::Soldier));
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

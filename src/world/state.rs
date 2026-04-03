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
        let mut enemies = Vec::new();

        for spawn in map.enemy_spawns() {
            for pos in enemy_spawn_positions(&map, *spawn) {
                let rect = Rect::new(
                    pos.x - spawn.kind.size().x * 0.5,
                    pos.y - spawn.kind.size().y * 0.5,
                    spawn.kind.size().x,
                    spawn.kind.size().y,
                );
                assert!(
                    !map.collides_rect(rect),
                    "enemy spawn at tile ({}, {}) collides with the map",
                    spawn.tile.x,
                    spawn.tile.y
                );

                enemies.push(Enemy::new_with_kind(pos, spawn.kind));
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

fn enemy_spawn_positions(map: &ImportedMap, spawn: crate::world::map::EnemySpawn) -> Vec<Vec2> {
    let center = map.tile_center(spawn.tile);

    match (spawn.kind, spawn.count) {
        // Two soldiers on one tile need distinct centers or they render as one
        // stacked sprite and immediately behave like they are occupying the
        // exact same point.
        (EnemyKind::Soldier, 2) => {
            let half_step = (map.tile_size - spawn.kind.size().x) * 0.5;
            vec![
                center + vec2(-half_step, 0.0),
                center + vec2(half_step, 0.0),
            ]
        }
        _ => vec![center; spawn.count],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::map::EnemySpawn;

    #[test]
    fn two_soldier_spawn_points_are_separated_within_the_tile() {
        let map = ImportedMap::load();
        let spawn = EnemySpawn {
            tile: ivec2(0, 0),
            kind: EnemyKind::Soldier,
            count: 2,
        };

        let positions = enemy_spawn_positions(&map, spawn);

        assert_eq!(positions.len(), 2);
        assert!(positions[0].distance(positions[1]) >= spawn.kind.size().x);
    }
}

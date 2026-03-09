use crate::entities::{Bullet, Enemy, EnemyKind, Explosion, Hostage, Jeep, Turret};
use crate::world::{LevelData, MissionState, TileKind, TileMap};
use macroquad::prelude::*;

pub struct World {
    pub map: TileMap,
    pub player: Jeep,
    pub enemies: Vec<Enemy>,
    pub turrets: Vec<Turret>,
    pub bullets: Vec<Bullet>,
    pub hostages: Vec<Hostage>,
    pub explosions: Vec<Explosion>,
    pub mission: MissionState,
    pub player_spawn: Vec2,
}

impl World {
    pub fn from_level(level: &LevelData) -> Self {
        let map = TileMap::from_level_data(level);
        let mut player_spawn = vec2(
            map.dimensions_px().x * 0.5,
            map.dimensions_px().y - crate::constants::TILE_SIZE * 1.5,
        );
        let mut enemies = Vec::new();
        let mut turrets = Vec::new();
        let mut hostages = Vec::new();

        for y in 0..map.height {
            for x in 0..map.width {
                let tile = ivec2(x as i32, y as i32);
                let center = map.tile_center(tile);
                if bunker_origin_at(&map, tile) {
                    turrets.push(Turret::new(
                        center
                            + vec2(
                                crate::constants::TILE_SIZE * 0.5,
                                crate::constants::TILE_SIZE * 0.5,
                            ),
                        tile,
                    ));
                }
                match map.tile_kind(tile).unwrap_or(TileKind::Grass) {
                    TileKind::PlayerSpawn => player_spawn = center,
                    TileKind::EnemySpawn => {
                        let kind = if (x + y) % 2 == 0 {
                            EnemyKind::Commando
                        } else {
                            EnemyKind::Rocketeer
                        };
                        enemies.push(Enemy::new(center, kind));
                    }
                    TileKind::HostageCage => {
                        hostages.push(Hostage::new(center + vec2(-28.0, 12.0), tile));
                        hostages.push(Hostage::new(center + vec2(28.0, 12.0), tile));
                    }
                    _ => {}
                }
            }
        }

        Self {
            map,
            player: Jeep::new(player_spawn),
            enemies,
            turrets,
            bullets: Vec::new(),
            hostages,
            explosions: Vec::new(),
            mission: MissionState::new(0),
            player_spawn,
        }
        .with_total_hostages()
    }

    fn with_total_hostages(mut self) -> Self {
        self.mission = MissionState::new(self.hostages.len() as u32);
        self
    }

    pub fn rider_count(&self) -> usize {
        self.hostages
            .iter()
            .filter(|hostage| matches!(hostage.state, crate::entities::HostageState::Riding { .. }))
            .count()
    }

    pub fn snapshot_positions(&mut self) {
        self.player.prev_pos = self.player.pos;

        for enemy in &mut self.enemies {
            enemy.prev_pos = enemy.pos;
        }

        for hostage in &mut self.hostages {
            hostage.prev_pos = hostage.pos;
        }

        for bullet in &mut self.bullets {
            bullet.prev_pos = bullet.pos;
        }
    }
}

fn bunker_origin_at(map: &TileMap, tile_pos: IVec2) -> bool {
    let neighbors = [
        tile_pos,
        tile_pos + ivec2(1, 0),
        tile_pos + ivec2(0, 1),
        tile_pos + ivec2(1, 1),
    ];
    if neighbors
        .iter()
        .any(|pos| map.tile_kind(*pos) != Some(TileKind::Wall))
    {
        return false;
    }

    map.tile_kind(tile_pos + ivec2(-1, 0)) != Some(TileKind::Wall)
        && map.tile_kind(tile_pos + ivec2(0, -1)) != Some(TileKind::Wall)
}

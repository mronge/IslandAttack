use crate::entities::{Barracks, Bullet, Enemy, EnemyKind, Jeep, Pow};
use crate::world::{ImportedMap, rect_from_center};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissionResult {
    Success,
    Failure,
}

pub struct World {
    pub map: ImportedMap,
    pub player: Jeep,
    pub enemies: Vec<Enemy>,
    pub barracks: Vec<Barracks>,
    pub pows: Vec<Pow>,
    pub bullets: Vec<Bullet>,
    pub rescued_pows: usize,
    pub total_pows: usize,
    pub lost_pows: usize,
    pub player_spawn: Vec2,
    mission_result: Option<MissionResult>,
}

impl World {
    pub fn load() -> Self {
        let map = ImportedMap::load();
        let probe = Jeep::new(Vec2::ZERO);
        let player_spawn = map.default_spawn_point_for(probe.size());
        let mut enemies = Vec::new();
        let mut barracks = Vec::new();

        for spawn in map.barracks_spawns() {
            let center = barracks_center(&map, spawn.top_left);
            let rect = rect_from_center(center, vec2(64.0, 64.0));
            assert!(
                !map.collides_rect(rect),
                "barracks spawn at tile ({}, {}) collides with the map",
                spawn.top_left.x,
                spawn.top_left.y
            );
            barracks.push(Barracks::new(center));
        }

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

        let total_pows = barracks.len() * crate::constants::POWS_PER_BARRACKS;

        Self {
            map,
            player: Jeep::new(player_spawn),
            enemies,
            barracks,
            pows: Vec::new(),
            bullets: Vec::new(),
            rescued_pows: 0,
            total_pows,
            lost_pows: 0,
            player_spawn,
            mission_result: None,
        }
    }

    pub fn reset_player(&mut self) {
        if self.mission_is_complete() {
            return;
        }

        self.lose_rescued_pows();
        if !self.mission_is_complete() {
            self.reset_player_state();
        }
    }

    pub fn snapshot_positions(&mut self) {
        self.player.prev_pos = self.player.pos;
        for enemy in &mut self.enemies {
            enemy.prev_pos = enemy.pos;
        }
        for pow in &mut self.pows {
            pow.prev_pos = pow.pos;
        }
        for bullet in &mut self.bullets {
            bullet.prev_pos = bullet.pos;
        }
    }

    pub fn mission_result(&self) -> Option<MissionResult> {
        self.mission_result
    }

    pub fn mission_is_complete(&self) -> bool {
        self.mission_result.is_some()
    }

    pub(crate) fn finish_mission_at_goal(&mut self) {
        if self.mission_result.is_some() {
            return;
        }

        self.mission_result = Some(if self.rescued_pows == self.total_pows {
            MissionResult::Success
        } else {
            MissionResult::Failure
        });
    }

    pub(crate) fn handle_player_death(&mut self) {
        self.lose_rescued_pows();
        self.reset_player_state();
    }

    fn lose_rescued_pows(&mut self) {
        if self.rescued_pows == 0 {
            return;
        }

        // POWs already inside the jeep are the only ones lost on a reset or
        // death. Released POWs still in the world remain available to rescue.
        self.lost_pows += self.rescued_pows;
        self.rescued_pows = 0;
    }

    fn reset_player_state(&mut self) {
        self.player = Jeep::new(self.player_spawn);
    }
}

fn barracks_center(map: &ImportedMap, top_left: IVec2) -> Vec2 {
    map.tile_center(top_left) + vec2(map.tile_size * 0.5, map.tile_size * 0.5)
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

    #[test]
    fn player_reset_clears_rescued_count_but_keeps_world_state() {
        let mut world = World::load();
        world.rescued_pows = 3;
        let barracks_before = world.barracks.len();
        world.pows.push(Pow::new(vec2(32.0, 32.0), vec2(1.0, 0.0)));

        world.reset_player();

        assert_eq!(world.rescued_pows, 0);
        assert_eq!(world.lost_pows, 3);
        assert_eq!(world.barracks.len(), barracks_before);
        assert_eq!(world.pows.len(), 1);
    }

    #[test]
    fn mission_succeeds_at_goal_only_when_every_pow_is_rescued() {
        let mut world = World::load();
        world.rescued_pows = world.total_pows;

        world.finish_mission_at_goal();

        assert_eq!(world.mission_result(), Some(MissionResult::Success));
    }

    #[test]
    fn mission_fails_at_goal_when_any_pow_is_not_rescued() {
        let mut world = World::load();
        world.lost_pows = 2;
        world.rescued_pows = world.total_pows.saturating_sub(2);

        world.finish_mission_at_goal();

        assert_eq!(world.mission_result(), Some(MissionResult::Failure));
    }
}

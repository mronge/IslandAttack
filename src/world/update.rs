use crate::constants::{
    BULLET_SPEED, JEEP_ACCEL, JEEP_BRAKE, PLAYER_BULLET_DAMAGE, PLAYER_FIRE_COOLDOWN,
    POW_BOARD_RANGE, POW_SIZE, POWS_PER_BARRACKS, SOLDIER_ALERT_RANGE,
};
use crate::entities::{
    ActorAnimState, Barracks, Bullet, BulletOwner, Direction, Enemy, EnemyKind, Pow,
};
use crate::input::PlayerCommand;
use crate::world::{ImportedMap, World, rect_from_center};
use macroquad::prelude::*;
use std::collections::HashMap;

impl World {
    pub fn update(&mut self, command: PlayerCommand, dt: f32) {
        if self.mission_is_complete() {
            return;
        }

        self.snapshot_positions();
        self.player.fire_cooldown = (self.player.fire_cooldown - dt).max(0.0);
        self.update_player(command, dt);
        self.update_bullets(dt);
        self.update_enemies(dt);
        self.update_pows(dt);
        self.cleanup();
    }

    fn update_player(&mut self, command: PlayerCommand, dt: f32) {
        let desired_velocity = if let Some(dir) = command.move_dir {
            self.player.dir = dir;
            dir.as_vec() * self.player.speed
        } else {
            Vec2::ZERO
        };

        let accel = if command.move_dir.is_some() {
            JEEP_ACCEL
        } else {
            JEEP_BRAKE
        };
        self.player.vel = move_towards_vec(self.player.vel, desired_velocity, accel * dt);
        if command.move_dir.is_none() && self.player.vel.length_squared() < 1.0 {
            self.player.vel = Vec2::ZERO;
        }

        let step = self.player.vel * dt;
        if step.x.abs() > 0.0 {
            let attempt = self.player.pos + vec2(step.x, 0.0);
            if can_move_player_to(
                &self.map,
                &self.barracks,
                &mut self.enemies,
                attempt,
                self.player.size(),
            ) {
                self.player.pos.x = attempt.x;
            } else {
                self.player.vel.x = 0.0;
            }
        }

        if step.y.abs() > 0.0 {
            let attempt = self.player.pos + vec2(0.0, step.y);
            if can_move_player_to(
                &self.map,
                &self.barracks,
                &mut self.enemies,
                attempt,
                self.player.size(),
            ) {
                self.player.pos.y = attempt.y;
            } else {
                self.player.vel.y = 0.0;
            }
        }

        if command.fire && self.player.fire_cooldown <= 0.0 {
            let muzzle = self.player.pos + self.player.dir.as_vec() * (self.player.size().x * 0.62);
            self.bullets.push(
                Bullet::new(
                    muzzle,
                    self.player.dir.as_vec() * BULLET_SPEED,
                    BulletOwner::Player,
                )
                .with_damage(PLAYER_BULLET_DAMAGE),
            );
            self.player.fire_cooldown = PLAYER_FIRE_COOLDOWN;
        }
    }

    fn update_bullets(&mut self, dt: f32) {
        let mut survivors = Vec::with_capacity(self.bullets.len());
        let mut player_hits = 0;
        let mut released_pows = Vec::new();

        for mut bullet in std::mem::take(&mut self.bullets) {
            bullet.prev_pos = bullet.pos;
            bullet.pos += bullet.vel * dt;
            bullet.ttl -= dt;

            let bullet_rect = Rect::new(
                bullet.pos.x - bullet.radius,
                bullet.pos.y - bullet.radius,
                bullet.radius * 2.0,
                bullet.radius * 2.0,
            );
            let mut hit = self.map.collides_rect(bullet_rect);

            if !hit {
                if let Some(index) = self
                    .barracks
                    .iter()
                    .position(|barracks| rects_overlap(bullet_rect, barracks_rect(barracks)))
                {
                    if bullet.owner == BulletOwner::Player {
                        let barracks_snapshot = self.barracks.clone();
                        let barracks = &mut self.barracks[index];
                        if barracks.can_take_damage() {
                            barracks.hp -= bullet.damage;
                            if barracks.hp <= 0 {
                                barracks.destroy();
                                released_pows.extend(release_pows_from_barracks(
                                    &self.map,
                                    &barracks_snapshot,
                                    &self.enemies,
                                    &self.pows,
                                    barracks,
                                    self.player.pos,
                                    self.player.size(),
                                ));
                            }
                        }
                    }
                    hit = true;
                }
            }

            if !hit {
                match bullet.owner {
                    BulletOwner::Player => {
                        for enemy in &mut self.enemies {
                            if enemy.can_act()
                                && enemy.pos.distance(bullet.pos)
                                    <= bullet.radius + enemy.size().x * 0.5
                            {
                                enemy.hp -= bullet.damage;
                                if enemy.hp <= 0 && enemy.kind == EnemyKind::Turret {
                                    enemy.destroy();
                                }
                                hit = true;
                                break;
                            }
                        }
                    }
                    BulletOwner::Enemy => {
                        if self.player.pos.distance(bullet.pos)
                            <= bullet.radius + self.player.size().x * 0.5
                        {
                            player_hits += bullet.damage;
                            hit = true;
                        }
                    }
                }
            }

            if !hit && bullet.ttl > 0.0 {
                survivors.push(bullet);
            }
        }

        self.pows.extend(released_pows);
        if player_hits > 0 {
            self.player.hp = (self.player.hp - player_hits).max(0);
            if self.player.hp <= 0 {
                survivors.clear();
                self.handle_player_death();
            }
        }

        self.bullets = survivors;
    }

    fn update_enemies(&mut self, dt: f32) {
        let mut spawned_bullets = Vec::new();
        // Track live occupancy by enemy kind and tile so each mover can decide
        // whether stepping into a neighboring tile would exceed that kind's cap.
        let mut tile_occupancy = HashMap::new();

        for enemy in self.enemies.iter().filter(|enemy| enemy.can_act()) {
            if let Some(tile) = enemy_tile_key(&self.map, enemy.kind, enemy.pos) {
                *tile_occupancy.entry(tile).or_insert(0) += 1;
            }
        }

        for enemy in &mut self.enemies {
            if !enemy.can_act() {
                continue;
            }

            enemy.fire_cooldown = (enemy.fire_cooldown - dt).max(0.0);
            enemy.shoot_timer = (enemy.shoot_timer - dt).max(0.0);

            let to_player = self.player.pos - enemy.pos;
            let distance_sq = to_player.length_squared();
            if distance_sq <= 1.0 {
                enemy.set_animation_state(ActorAnimState::Idle);
                continue;
            }

            let distance = distance_sq.sqrt();
            let step_dir = to_player / distance;
            enemy.dir = Direction::from_vec(step_dir);
            let can_see_player_for_pursuit = can_enemy_pursue_player(
                &self.map,
                &self.barracks,
                enemy.kind,
                enemy.pos,
                self.player.pos,
                distance,
            );
            let can_shoot_from_here = distance <= enemy.kind.fire_range()
                && has_world_line_of_sight(&self.map, &self.barracks, enemy.pos, self.player.pos);

            if enemy.shoot_timer > 0.0 {
                enemy.set_animation_state(ActorAnimState::Shoot);
                continue;
            }

            if can_shoot_from_here {
                enemy.set_animation_state(ActorAnimState::Idle);
                if enemy.fire_cooldown <= 0.0 {
                    enemy.set_animation_state(ActorAnimState::Shoot);
                    enemy.shoot_timer = enemy.kind.shoot_duration();

                    let muzzle = enemy.pos + step_dir * (enemy.size().x * 0.75);
                    spawned_bullets.push(
                        Bullet::new(
                            muzzle,
                            step_dir * enemy.kind.bullet_speed(),
                            BulletOwner::Enemy,
                        )
                        .with_damage(enemy.kind.bullet_damage())
                        .with_radius(enemy.kind.bullet_radius()),
                    );
                    enemy.fire_cooldown = enemy.kind.fire_cooldown();
                }
                continue;
            }

            if enemy.kind.is_stationary() {
                enemy.set_animation_state(ActorAnimState::Idle);
                continue;
            }

            if !can_see_player_for_pursuit {
                enemy.set_animation_state(ActorAnimState::Idle);
                continue;
            }

            let start_pos = enemy.pos;
            let attempt_x = enemy.pos + vec2(step_dir.x * enemy.speed * dt, 0.0);
            let rect_x = rect_from_center(attempt_x, enemy.size());
            if can_move_enemy_to(
                &self.map,
                &self.barracks,
                enemy,
                attempt_x,
                rect_x,
                &tile_occupancy,
            ) {
                update_enemy_tile_occupancy(&self.map, &mut tile_occupancy, enemy, attempt_x);
                enemy.pos.x = attempt_x.x;
            }

            let attempt_y = enemy.pos + vec2(0.0, step_dir.y * enemy.speed * dt);
            let rect_y = rect_from_center(attempt_y, enemy.size());
            if can_move_enemy_to(
                &self.map,
                &self.barracks,
                enemy,
                attempt_y,
                rect_y,
                &tile_occupancy,
            ) {
                update_enemy_tile_occupancy(&self.map, &mut tile_occupancy, enemy, attempt_y);
                enemy.pos.y = attempt_y.y;
            }

            if enemy.pos.distance_squared(start_pos) > 0.01 {
                enemy.set_animation_state(ActorAnimState::Walk);
                enemy.tick_animation(dt);
            } else {
                enemy.set_animation_state(ActorAnimState::Idle);
            }
        }

        self.bullets.extend(spawned_bullets);
    }

    fn update_pows(&mut self, dt: f32) {
        let mut boarded_this_tick = 0usize;

        for pow in &mut self.pows {
            if pow.boarded {
                continue;
            }

            let to_player = self.player.pos - pow.pos;
            let player_distance_sq = to_player.length_squared();
            let should_board = player_distance_sq <= POW_BOARD_RANGE * POW_BOARD_RANGE;

            let desired_dir = if should_board && player_distance_sq > 1.0 {
                to_player / player_distance_sq.sqrt()
            } else if !pow.has_cleared_barracks() {
                pow.escape_dir
            } else {
                Vec2::ZERO
            };

            if desired_dir == Vec2::ZERO {
                pow.set_animation_state(ActorAnimState::Idle);
                continue;
            }

            pow.dir = Direction::from_vec(desired_dir);
            let start_pos = pow.pos;
            let attempt_x = pow.pos + vec2(desired_dir.x * pow.speed * dt, 0.0);
            if can_move_pow_to(
                &self.map,
                &self.barracks,
                attempt_x,
                rect_from_center(attempt_x, pow.size()),
            ) {
                pow.pos.x = attempt_x.x;
            }

            let attempt_y = pow.pos + vec2(0.0, desired_dir.y * pow.speed * dt);
            if can_move_pow_to(
                &self.map,
                &self.barracks,
                attempt_y,
                rect_from_center(attempt_y, pow.size()),
            ) {
                pow.pos.y = attempt_y.y;
            }

            if should_board
                && pow.pos.distance(self.player.pos)
                    <= pow.size().x * 0.5 + self.player.size().x * 0.5 + 4.0
            {
                pow.boarded = true;
                self.rescued_pows += 1;
                boarded_this_tick += 1;
                continue;
            }

            if pow.pos.distance_squared(start_pos) > 0.01 {
                pow.set_animation_state(ActorAnimState::Walk);
                pow.tick_animation(dt);
            } else {
                pow.set_animation_state(ActorAnimState::Idle);
            }
        }

        if boarded_this_tick > 0 {
            self.resolve_mission_if_complete();
        }
    }

    fn cleanup(&mut self) {
        self.enemies
            .retain(|enemy| enemy.kind == EnemyKind::Turret || enemy.hp > 0);
        self.pows.retain(|pow| !pow.boarded);
    }
}

fn move_towards_vec(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
    let delta = target - current;
    let distance = delta.length();
    if distance <= max_delta || distance == 0.0 {
        target
    } else {
        current + delta / distance * max_delta
    }
}

fn can_enemy_pursue_player(
    map: &ImportedMap,
    barracks: &[Barracks],
    kind: EnemyKind,
    enemy_pos: Vec2,
    player_pos: Vec2,
    distance: f32,
) -> bool {
    match kind {
        // Soldiers stay dormant until the jeep is close enough to plausibly be
        // on their half-screen "view" and they have line of sight.
        EnemyKind::Soldier => {
            distance <= SOLDIER_ALERT_RANGE
                && has_world_line_of_sight(map, barracks, enemy_pos, player_pos)
        }
        EnemyKind::Turret => true,
    }
}

fn can_move_player_to(
    map: &ImportedMap,
    barracks: &[Barracks],
    enemies: &mut [Enemy],
    attempt_pos: Vec2,
    player_size: Vec2,
) -> bool {
    let rect = rect_from_center(attempt_pos, player_size);
    if map.collides_rect(rect) || collides_with_barracks(rect, barracks) {
        return false;
    }

    // Jeep contact resolves directly against enemies: soldiers are crushed on
    // impact, while turrets always stay solid, even after being destroyed.
    !resolve_player_enemy_collision(rect, enemies)
}

fn resolve_player_enemy_collision(player_rect: Rect, enemies: &mut [Enemy]) -> bool {
    let mut blocked = false;

    for enemy in enemies {
        let enemy_rect = rect_from_center(enemy.pos, enemy.size());
        if !rects_overlap(player_rect, enemy_rect) {
            continue;
        }

        match enemy.kind {
            EnemyKind::Soldier => enemy.hp = 0,
            EnemyKind::Turret => blocked = true,
        }
    }

    blocked
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

type EnemyTileKey = (EnemyKind, i32, i32);

fn enemy_tile_key(map: &ImportedMap, kind: EnemyKind, pos: Vec2) -> Option<EnemyTileKey> {
    let tile_x = (pos.x / map.tile_size).floor() as i32;
    let tile_y = (pos.y / map.tile_size).floor() as i32;
    let tile = ivec2(tile_x, tile_y);
    map.in_bounds(tile).then_some((kind, tile_x, tile_y))
}

fn can_move_enemy_to(
    map: &ImportedMap,
    barracks: &[Barracks],
    enemy: &Enemy,
    attempt_pos: Vec2,
    attempt_rect: Rect,
    tile_occupancy: &HashMap<EnemyTileKey, usize>,
) -> bool {
    if map.collides_rect(attempt_rect) || collides_with_barracks(attempt_rect, barracks) {
        return false;
    }

    let current_tile = enemy_tile_key(map, enemy.kind, enemy.pos);
    let Some(attempt_tile) = enemy_tile_key(map, enemy.kind, attempt_pos) else {
        return false;
    };

    // Sliding around within the current tile is always allowed. The cap only
    // blocks entering a different tile that is already full for this kind.
    if Some(attempt_tile) == current_tile {
        return true;
    }

    tile_occupancy.get(&attempt_tile).copied().unwrap_or(0) < enemy.kind.max_per_tile()
}

fn can_move_pow_to(
    map: &ImportedMap,
    barracks: &[Barracks],
    attempt_pos: Vec2,
    attempt_rect: Rect,
) -> bool {
    enemy_tile_key(map, EnemyKind::Soldier, attempt_pos).is_some()
        && !map.collides_rect(attempt_rect)
        && !collides_with_barracks(attempt_rect, barracks)
}

fn update_enemy_tile_occupancy(
    map: &ImportedMap,
    tile_occupancy: &mut HashMap<EnemyTileKey, usize>,
    enemy: &Enemy,
    attempt_pos: Vec2,
) {
    let current_tile = enemy_tile_key(map, enemy.kind, enemy.pos);
    let attempt_tile = enemy_tile_key(map, enemy.kind, attempt_pos);

    if current_tile == attempt_tile {
        return;
    }

    // Mirror the accepted move in the occupancy map immediately so later
    // enemies in this frame see the updated counts.
    if let Some(current_tile) = current_tile {
        decrement_tile_occupancy(tile_occupancy, current_tile);
    }

    if let Some(attempt_tile) = attempt_tile {
        *tile_occupancy.entry(attempt_tile).or_insert(0) += 1;
    }
}

fn decrement_tile_occupancy(tile_occupancy: &mut HashMap<EnemyTileKey, usize>, tile: EnemyTileKey) {
    let Some(count) = tile_occupancy.get_mut(&tile) else {
        return;
    };

    if *count <= 1 {
        tile_occupancy.remove(&tile);
    } else {
        *count -= 1;
    }
}

fn barracks_rect(barracks: &Barracks) -> Rect {
    rect_from_center(barracks.pos, barracks.size())
}

fn collides_with_barracks(rect: Rect, barracks: &[Barracks]) -> bool {
    barracks
        .iter()
        .any(|barracks| rects_overlap(rect, barracks_rect(barracks)))
}

fn has_world_line_of_sight(map: &ImportedMap, barracks: &[Barracks], from: Vec2, to: Vec2) -> bool {
    let delta = to - from;
    let distance = delta.length();
    if distance <= 1.0 {
        return true;
    }

    let step = delta / distance;
    let step_size = 4.0;
    let steps = (distance / step_size).ceil() as i32;

    for i in 1..steps {
        let sample = from + step * (i as f32 * step_size);
        if map.collides_point(sample) || point_in_any_barracks(sample, barracks) {
            return false;
        }
    }

    !map.collides_point(to) && !point_in_any_barracks(to, barracks)
}

fn point_in_any_barracks(point: Vec2, barracks: &[Barracks]) -> bool {
    barracks
        .iter()
        .any(|barracks| point_in_rect(point, barracks_rect(barracks)))
}

fn point_in_rect(point: Vec2, rect: Rect) -> bool {
    point.x >= rect.x && point.x < rect.x + rect.w && point.y >= rect.y && point.y < rect.y + rect.h
}

fn release_pows_from_barracks(
    map: &ImportedMap,
    barracks_list: &[Barracks],
    enemies: &[Enemy],
    existing_pows: &[Pow],
    barracks: &mut Barracks,
    player_pos: Vec2,
    player_size: Vec2,
) -> Vec<Pow> {
    if !barracks.can_release_pows() {
        return Vec::new();
    }

    let escape_dirs = [
        vec2(0.0, -1.0),
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
    ];
    let mut pows = Vec::with_capacity(POWS_PER_BARRACKS);
    let mut reserved_rects = Vec::with_capacity(POWS_PER_BARRACKS);
    for preferred_dir in 0..POWS_PER_BARRACKS {
        let mut spawned = false;

        // Prefer spreading POWs across the four sides first, but allow a
        // second POW from an open side when another lane is blocked by walls
        // or a neighboring barracks footprint.
        for offset in 0..escape_dirs.len() {
            let escape_dir = escape_dirs[(preferred_dir + offset) % escape_dirs.len()];
            if let Some(spawn_pos) = find_pow_spawn_pos(
                map,
                barracks_list,
                enemies,
                existing_pows,
                &reserved_rects,
                barracks,
                escape_dir,
                player_pos,
                player_size,
            ) {
                reserved_rects.push(rect_from_center(spawn_pos, vec2(POW_SIZE, POW_SIZE)));
                pows.push(Pow::new(spawn_pos, escape_dir));
                spawned = true;
                break;
            }
        }

        if !spawned {
            break;
        }
    }
    barracks.mark_pows_released();
    pows
}

fn find_pow_spawn_pos(
    map: &ImportedMap,
    barracks_list: &[Barracks],
    enemies: &[Enemy],
    existing_pows: &[Pow],
    reserved_rects: &[Rect],
    source_barracks: &Barracks,
    escape_dir: Vec2,
    player_pos: Vec2,
    player_size: Vec2,
) -> Option<Vec2> {
    let pow_size = vec2(POW_SIZE, POW_SIZE);
    let start_distance = source_barracks.size().x * 0.5 + pow_size.x * 0.75;
    let lateral_dir = vec2(-escape_dir.y, escape_dir.x);

    for step in 0..8 {
        let distance = start_distance + step as f32 * pow_size.x;
        for lateral_steps in [0.0, -1.0, 1.0, -2.0, 2.0] {
            let candidate = source_barracks.pos
                + escape_dir * distance
                + lateral_dir * (lateral_steps * pow_size.x);
            let rect = rect_from_center(candidate, pow_size);

            if map.collides_rect(rect)
                || collides_with_barracks_except(rect, barracks_list, source_barracks.pos)
                || collides_with_enemy_rects(rect, enemies)
                || collides_with_pow_rects(rect, existing_pows)
                || rects_overlap(rect, rect_from_center(player_pos, player_size))
                || reserved_rects
                    .iter()
                    .any(|reserved| rects_overlap(rect, *reserved))
            {
                continue;
            }

            return Some(candidate);
        }
    }

    None
}

fn collides_with_barracks_except(rect: Rect, barracks: &[Barracks], ignore_center: Vec2) -> bool {
    barracks.iter().any(|barracks| {
        barracks.pos.distance_squared(ignore_center) > 0.01
            && rects_overlap(rect, barracks_rect(barracks))
    })
}

fn collides_with_enemy_rects(rect: Rect, enemies: &[Enemy]) -> bool {
    enemies
        .iter()
        .any(|enemy| rects_overlap(rect, rect_from_center(enemy.pos, enemy.size())))
}

fn collides_with_pow_rects(rect: Rect, pows: &[Pow]) -> bool {
    pows.iter()
        .any(|pow| rects_overlap(rect, rect_from_center(pow.pos, pow.size())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walkable_tiles(map: &ImportedMap, size: Vec2, count: usize) -> Vec<IVec2> {
        let mut tiles = Vec::new();

        for y in 0..map.height as i32 {
            for x in 0..map.width as i32 {
                let tile = ivec2(x, y);
                let center = map.tile_center(tile);
                let rect = rect_from_center(center, size);
                if !map.collides_rect(rect) {
                    tiles.push(tile);
                    if tiles.len() == count {
                        return tiles;
                    }
                }
            }
        }

        panic!("expected at least {count} walkable tiles in the map");
    }

    fn barracks_tile_with_four_release_slots(map: &ImportedMap) -> IVec2 {
        for y in 0..map.height as i32 {
            for x in 0..map.width as i32 {
                let tile = ivec2(x, y);
                let center = map.tile_center(tile);
                if map.collides_rect(rect_from_center(center, vec2(64.0, 64.0))) {
                    continue;
                }

                let mut barracks = Barracks::new(center);
                barracks.destroy();
                let released = release_pows_from_barracks(
                    map,
                    &[],
                    &[],
                    &[],
                    &mut barracks,
                    vec2(-999.0, -999.0),
                    vec2(32.0, 32.0),
                );
                if released.len() == POWS_PER_BARRACKS {
                    return tile;
                }
            }
        }

        panic!("expected at least one barracks tile with four open release slots");
    }

    #[test]
    fn soldier_limit_is_two_per_tile() {
        assert_eq!(EnemyKind::Soldier.max_per_tile(), 2);
    }

    #[test]
    fn turret_rules_are_stricter_and_stronger() {
        assert_eq!(EnemyKind::Turret.max_per_tile(), 1);
        assert!(EnemyKind::Turret.is_stationary());
        assert!(EnemyKind::Turret.bullet_damage() > EnemyKind::Soldier.bullet_damage());
    }

    #[test]
    fn destroyed_turret_stays_but_cannot_act() {
        let mut turret = Enemy::new_with_kind(vec2(32.0, 32.0), EnemyKind::Turret);
        turret.destroy();

        assert!(turret.is_destroyed());
        assert!(!turret.can_act());
    }

    #[test]
    fn soldier_pursuit_requires_alert_range() {
        let map = ImportedMap::load();
        let enemy_pos = vec2(64.0, 64.0);
        let player_pos = enemy_pos + vec2(SOLDIER_ALERT_RANGE + 1.0, 0.0);

        assert!(!can_enemy_pursue_player(
            &map,
            &[],
            EnemyKind::Soldier,
            enemy_pos,
            player_pos,
            SOLDIER_ALERT_RANGE + 1.0,
        ));
        assert!(can_enemy_pursue_player(
            &map,
            &[],
            EnemyKind::Turret,
            enemy_pos,
            player_pos,
            SOLDIER_ALERT_RANGE + 1.0,
        ));
    }

    #[test]
    fn moving_into_full_tile_is_blocked() {
        let map = ImportedMap::load();
        let enemy = Enemy::new(Vec2::ZERO);
        let tiles = walkable_tiles(&map, enemy.size(), 2);
        let target_tile = tiles[0];
        let start_tile = tiles[1];
        let center = map.tile_center(target_tile);
        let mut tile_occupancy = HashMap::new();
        tile_occupancy.insert((EnemyKind::Soldier, target_tile.x, target_tile.y), 2);

        let enemy = Enemy::new(map.tile_center(start_tile));
        let rect = rect_from_center(center, enemy.size());

        assert!(!can_move_enemy_to(
            &map,
            &[],
            &enemy,
            center,
            rect,
            &tile_occupancy,
        ));
    }

    #[test]
    fn moving_within_same_tile_ignores_cap() {
        let map = ImportedMap::load();
        let enemy = Enemy::new(Vec2::ZERO);
        let tile = walkable_tiles(&map, enemy.size(), 1)[0];
        let center = map.tile_center(tile);
        let mut tile_occupancy = HashMap::new();
        tile_occupancy.insert((EnemyKind::Soldier, tile.x, tile.y), 2);

        let enemy = Enemy::new(center);
        let attempt = center + vec2(1.0, 0.0);
        let rect = rect_from_center(attempt, enemy.size());

        assert!(can_move_enemy_to(
            &map,
            &[],
            &enemy,
            attempt,
            rect,
            &tile_occupancy,
        ));
    }

    #[test]
    fn destroyed_barracks_release_four_pows_once() {
        let map = ImportedMap::load();
        let tile = barracks_tile_with_four_release_slots(&map);
        let mut barracks = Barracks::new(map.tile_center(tile));
        barracks.destroy();

        let pows = release_pows_from_barracks(
            &map,
            &[],
            &[],
            &[],
            &mut barracks,
            vec2(-999.0, -999.0),
            vec2(32.0, 32.0),
        );

        assert_eq!(pows.len(), POWS_PER_BARRACKS);
        assert!(barracks.released_pows);
        assert!(
            release_pows_from_barracks(
                &map,
                &[],
                &[],
                &[],
                &mut barracks,
                vec2(-999.0, -999.0),
                vec2(32.0, 32.0),
            )
            .is_empty()
        );
    }

    #[test]
    fn adjacent_barracks_still_release_four_pows() {
        let map = ImportedMap::load();
        let barracks_list: Vec<_> = map
            .barracks_spawns()
            .iter()
            .map(|spawn| {
                let center = map.tile_center(spawn.top_left)
                    + vec2(map.tile_size * 0.5, map.tile_size * 0.5);
                Barracks::new(center)
            })
            .collect();
        let left_barracks_index = barracks_list
            .iter()
            .enumerate()
            .min_by_key(|(_, barracks)| barracks.pos.x as i32)
            .map(|(index, _)| index)
            .expect("expected at least one barracks spawn");
        let mut source_barracks = barracks_list[left_barracks_index].clone();
        source_barracks.destroy();

        let pows = release_pows_from_barracks(
            &map,
            &barracks_list,
            &[],
            &[],
            &mut source_barracks,
            vec2(-999.0, -999.0),
            vec2(32.0, 32.0),
        );

        assert_eq!(pows.len(), POWS_PER_BARRACKS);
    }

    #[test]
    fn released_pows_do_not_spawn_inside_enemies() {
        let map = ImportedMap::load();
        let tile = walkable_tiles(&map, vec2(64.0, 64.0), 1)[0];
        let center = map.tile_center(tile);
        let mut barracks = Barracks::new(center);
        barracks.destroy();
        let enemy = Enemy::new(center + vec2(0.0, -(32.0 + POW_SIZE * 0.75)));

        let pows = release_pows_from_barracks(
            &map,
            &[],
            &[enemy.clone()],
            &[],
            &mut barracks,
            vec2(-999.0, -999.0),
            vec2(32.0, 32.0),
        );

        assert!(pows.iter().all(|pow| {
            !rects_overlap(
                rect_from_center(pow.pos, pow.size()),
                rect_from_center(enemy.pos, enemy.size()),
            )
        }));
    }

    #[test]
    fn jeep_collision_with_soldier_kills_it_without_blocking() {
        let mut enemies = vec![Enemy::new(vec2(0.0, 0.0))];
        let player_rect = rect_from_center(vec2(0.0, 0.0), vec2(32.0, 32.0));

        assert!(!resolve_player_enemy_collision(player_rect, &mut enemies));
        assert_eq!(enemies[0].kind, EnemyKind::Soldier);
        assert_eq!(enemies[0].hp, 0);
        assert!(!enemies[0].can_act());
    }

    #[test]
    fn turret_collision_blocks_even_when_destroyed() {
        let player_rect = rect_from_center(vec2(0.0, 0.0), vec2(32.0, 32.0));
        let mut active_turret = vec![Enemy::new_with_kind(vec2(0.0, 0.0), EnemyKind::Turret)];

        assert!(resolve_player_enemy_collision(
            player_rect,
            &mut active_turret
        ));
        assert_eq!(active_turret[0].hp, EnemyKind::Turret.hp());

        let mut destroyed_turret = vec![Enemy::new_with_kind(vec2(0.0, 0.0), EnemyKind::Turret)];
        destroyed_turret[0].destroy();

        assert!(resolve_player_enemy_collision(
            player_rect,
            &mut destroyed_turret,
        ));
        assert!(destroyed_turret[0].is_destroyed());
    }
}

use crate::constants::{
    BULLET_SPEED, ENEMY_BULLET_SPEED, ENEMY_FIRE_COOLDOWN, ENEMY_FIRE_RANGE, ENEMY_SHOOT_DURATION,
    JEEP_ACCEL, JEEP_BRAKE, PLAYER_FIRE_COOLDOWN,
};
use crate::entities::{Bullet, BulletOwner, Direction, Enemy, EnemyAnimState, EnemyKind};
use crate::input::PlayerCommand;
use crate::world::{ImportedMap, World, rect_from_center};
use macroquad::prelude::*;
use std::collections::HashMap;

impl World {
    pub fn update(&mut self, command: PlayerCommand, dt: f32) {
        self.snapshot_positions();
        self.player.fire_cooldown = (self.player.fire_cooldown - dt).max(0.0);
        self.update_player(command, dt);
        self.update_bullets(dt);
        self.update_enemies(dt);
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
            let rect = rect_from_center(attempt, self.player.size());
            if !self.map.collides_rect(rect) {
                self.player.pos.x = attempt.x;
            } else {
                self.player.vel.x = 0.0;
            }
        }

        if step.y.abs() > 0.0 {
            let attempt = self.player.pos + vec2(0.0, step.y);
            let rect = rect_from_center(attempt, self.player.size());
            if !self.map.collides_rect(rect) {
                self.player.pos.y = attempt.y;
            } else {
                self.player.vel.y = 0.0;
            }
        }

        if command.fire && self.player.fire_cooldown <= 0.0 {
            let muzzle = self.player.pos + self.player.dir.as_vec() * (self.player.size().x * 0.62);
            self.bullets.push(Bullet::new(
                muzzle,
                self.player.dir.as_vec() * BULLET_SPEED,
                BulletOwner::Player,
            ));
            self.player.fire_cooldown = PLAYER_FIRE_COOLDOWN;
        }
    }

    fn update_bullets(&mut self, dt: f32) {
        let mut survivors = Vec::with_capacity(self.bullets.len());
        let mut player_hits = 0;

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
                match bullet.owner {
                    BulletOwner::Player => {
                        for enemy in &mut self.enemies {
                            if enemy.hp > 0 && enemy.pos.distance(bullet.pos) <= bullet.radius + 9.0
                            {
                                enemy.hp -= 1;
                                hit = true;
                                break;
                            }
                        }
                    }
                    BulletOwner::Enemy => {
                        if self.player.pos.distance(bullet.pos)
                            <= bullet.radius + self.player.size().x * 0.5
                        {
                            player_hits += 1;
                            hit = true;
                        }
                    }
                }
            }

            if !hit && bullet.ttl > 0.0 {
                survivors.push(bullet);
            }
        }

        if player_hits > 0 {
            self.player.hp = (self.player.hp - player_hits).max(0);
            if self.player.hp <= 0 {
                survivors.clear();
                self.reset_player();
            }
        }

        self.bullets = survivors;
    }

    fn update_enemies(&mut self, dt: f32) {
        let mut spawned_bullets = Vec::new();
        // Track live occupancy by enemy kind and tile so each mover can decide
        // whether stepping into a neighboring tile would exceed that kind's cap.
        let mut tile_occupancy = HashMap::new();

        for enemy in self.enemies.iter().filter(|enemy| enemy.hp > 0) {
            if let Some(tile) = enemy_tile_key(&self.map, enemy.kind, enemy.pos) {
                *tile_occupancy.entry(tile).or_insert(0) += 1;
            }
        }

        for enemy in &mut self.enemies {
            if enemy.hp <= 0 {
                continue;
            }

            enemy.fire_cooldown = (enemy.fire_cooldown - dt).max(0.0);
            enemy.shoot_timer = (enemy.shoot_timer - dt).max(0.0);

            let to_player = self.player.pos - enemy.pos;
            let distance_sq = to_player.length_squared();
            if distance_sq <= 1.0 {
                enemy.set_animation_state(EnemyAnimState::Idle);
                continue;
            }

            let distance = distance_sq.sqrt();
            let step_dir = to_player / distance;
            enemy.dir = Direction::from_vec(step_dir);
            let can_shoot_from_here = distance <= ENEMY_FIRE_RANGE
                && self.map.has_line_of_sight(enemy.pos, self.player.pos);

            if enemy.shoot_timer > 0.0 {
                enemy.set_animation_state(EnemyAnimState::Shoot);
                continue;
            }

            if can_shoot_from_here {
                enemy.set_animation_state(EnemyAnimState::Idle);
                if enemy.fire_cooldown <= 0.0 {
                    enemy.set_animation_state(EnemyAnimState::Shoot);
                    enemy.shoot_timer = ENEMY_SHOOT_DURATION;

                    let muzzle = enemy.pos + step_dir * 12.0;
                    spawned_bullets.push(Bullet::new(
                        muzzle,
                        step_dir * ENEMY_BULLET_SPEED,
                        BulletOwner::Enemy,
                    ));
                    enemy.fire_cooldown = ENEMY_FIRE_COOLDOWN;
                }
                continue;
            }

            let start_pos = enemy.pos;
            let attempt_x = enemy.pos + vec2(step_dir.x * enemy.speed * dt, 0.0);
            let rect_x = rect_from_center(attempt_x, enemy.size());
            if can_move_enemy_to(&self.map, enemy, attempt_x, rect_x, &tile_occupancy) {
                update_enemy_tile_occupancy(&self.map, &mut tile_occupancy, enemy, attempt_x);
                enemy.pos.x = attempt_x.x;
            }

            let attempt_y = enemy.pos + vec2(0.0, step_dir.y * enemy.speed * dt);
            let rect_y = rect_from_center(attempt_y, enemy.size());
            if can_move_enemy_to(&self.map, enemy, attempt_y, rect_y, &tile_occupancy) {
                update_enemy_tile_occupancy(&self.map, &mut tile_occupancy, enemy, attempt_y);
                enemy.pos.y = attempt_y.y;
            }

            if enemy.pos.distance_squared(start_pos) > 0.01 {
                enemy.set_animation_state(EnemyAnimState::Walk);
                enemy.tick_animation(dt);
            } else {
                enemy.set_animation_state(EnemyAnimState::Idle);
            }
        }

        self.bullets.extend(spawned_bullets);
    }

    fn cleanup(&mut self) {
        self.enemies.retain(|enemy| enemy.hp > 0);
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

type EnemyTileKey = (EnemyKind, i32, i32);

fn enemy_tile_key(map: &ImportedMap, kind: EnemyKind, pos: Vec2) -> Option<EnemyTileKey> {
    let tile_x = (pos.x / map.tile_size).floor() as i32;
    let tile_y = (pos.y / map.tile_size).floor() as i32;
    let tile = ivec2(tile_x, tile_y);
    map.in_bounds(tile).then_some((kind, tile_x, tile_y))
}

fn can_move_enemy_to(
    map: &ImportedMap,
    enemy: &Enemy,
    attempt_pos: Vec2,
    attempt_rect: Rect,
    tile_occupancy: &HashMap<EnemyTileKey, usize>,
) -> bool {
    if map.collides_rect(attempt_rect) {
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

    #[test]
    fn soldier_limit_is_two_per_tile() {
        assert_eq!(EnemyKind::Soldier.max_per_tile(), 2);
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
            &enemy,
            attempt,
            rect,
            &tile_occupancy,
        ));
    }
}

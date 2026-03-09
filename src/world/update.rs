use crate::constants::{BULLET_SPEED, JEEP_ACCEL, JEEP_BRAKE, TILE_SIZE};
use crate::entities::{Bullet, Direction, EnemyState, Explosion, HostageState, rider_offset};
use crate::input::PlayerCommand;
use crate::world::{TileKind, World, rect_from_center};
use macroquad::prelude::*;

impl World {
    pub fn update(&mut self, command: PlayerCommand, dt: f32) {
        if self.mission.game_over || self.mission.victory {
            return;
        }

        self.snapshot_positions();
        self.player.fire_cooldown = (self.player.fire_cooldown - dt).max(0.0);
        self.player.invuln_timer = (self.player.invuln_timer - dt).max(0.0);

        self.update_player(command, dt);
        self.update_bullets(dt);
        self.release_freed_hostages();
        self.update_enemies(dt);
        self.update_hostages(dt);
        self.update_explosions(dt);
        self.handle_extraction();
        self.cleanup();
        self.update_mission_state();
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
            let muzzle = self.player.pos + self.player.dir.as_vec() * (TILE_SIZE * 0.8);
            self.bullets
                .push(Bullet::new(muzzle, self.player.dir.as_vec() * BULLET_SPEED));
            self.player.fire_cooldown = 0.12;
        }
    }

    fn update_bullets(&mut self, dt: f32) {
        let mut survivors = Vec::with_capacity(self.bullets.len());

        for mut bullet in std::mem::take(&mut self.bullets) {
            bullet.prev_pos = bullet.pos;
            bullet.pos += bullet.vel * dt;
            bullet.ttl -= dt;

            let mut hit = false;

            for enemy in &mut self.enemies {
                if enemy.hp > 0
                    && enemy.pos.distance(bullet.pos) <= bullet.radius + TILE_SIZE * 0.25
                {
                    enemy.hp -= i32::from(bullet.damage);
                    hit = true;
                    self.explosions.push(Explosion::new(bullet.pos));
                    break;
                }
            }

            if !hit && self.map.damage_at_world(bullet.pos, bullet.damage) {
                hit = true;
                self.explosions.push(Explosion::new(bullet.pos));
            }

            if !hit && bullet.ttl > 0.0 {
                survivors.push(bullet);
            }
        }

        self.bullets = survivors;
    }

    fn release_freed_hostages(&mut self) {
        for hostage in &mut self.hostages {
            if hostage.state == HostageState::Captive
                && self
                    .map
                    .tile_kind(hostage.home_tile)
                    .is_some_and(|kind| kind != TileKind::HostageCage)
            {
                hostage.state = HostageState::RunningToJeep;
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        for enemy in &mut self.enemies {
            if enemy.hp <= 0 {
                continue;
            }

            let to_player = self.player.pos - enemy.pos;
            let distance = to_player.length();

            if distance < TILE_SIZE * 5.0 {
                enemy.state = EnemyState::Chase;
                let step = dominant_axis_step(to_player);
                let attempt = enemy.pos + step * enemy.speed * dt;
                let rect = rect_from_center(attempt, enemy.size());
                if !self.map.collides_rect(rect) {
                    enemy.pos = attempt;
                }
            } else {
                enemy.state = EnemyState::Idle;
            }
        }

        for enemy in &self.enemies {
            if enemy.hp > 0
                && enemy.pos.distance(self.player.pos) <= TILE_SIZE * 0.75
                && self.player.invuln_timer <= 0.0
            {
                self.damage_player();
                break;
            }
        }
    }

    fn update_hostages(&mut self, dt: f32) {
        for idx in 0..self.hostages.len() {
            let state = self.hostages[idx].state;
            match state {
                HostageState::Captive | HostageState::Rescued => {}
                HostageState::RunningToJeep => {
                    let rider_count = self.rider_count();
                    let hostage = &mut self.hostages[idx];
                    let to_player = self.player.pos - hostage.pos;

                    if rider_count < self.player.rider_capacity && to_player.length() < TILE_SIZE {
                        hostage.state = HostageState::Riding { slot: rider_count };
                    } else if to_player.length() > 0.1 {
                        hostage.pos += to_player.normalize() * hostage.speed * dt;
                    }
                }
                HostageState::Riding { slot } => {
                    self.hostages[idx].pos = self.player.pos + rider_offset(slot);
                }
            }
        }

        self.normalize_rider_slots();
    }

    fn update_explosions(&mut self, dt: f32) {
        for explosion in &mut self.explosions {
            explosion.timer -= dt;
        }
    }

    fn handle_extraction(&mut self) {
        let Some(tile) = self.map.world_to_tile(self.player.pos) else {
            return;
        };
        if self.map.tile_kind(tile) != Some(TileKind::Extraction) {
            return;
        }

        let mut rescued_now = 0;
        for hostage in &mut self.hostages {
            if matches!(hostage.state, HostageState::Riding { .. }) {
                hostage.state = HostageState::Rescued;
                rescued_now += 1;
            }
        }

        if rescued_now > 0 {
            self.mission.rescued_total += rescued_now;
            while self.mission.rescued_total >= self.mission.next_extra_life_at {
                self.mission.lives = self.mission.lives.saturating_add(1);
                self.mission.next_extra_life_at += crate::constants::EXTRA_LIFE_EVERY;
            }
        }
    }

    fn cleanup(&mut self) {
        self.enemies.retain(|enemy| enemy.hp > 0);
        self.explosions.retain(|explosion| explosion.timer > 0.0);
    }

    fn update_mission_state(&mut self) {
        let pending = self.hostages.iter().any(|hostage| {
            matches!(
                hostage.state,
                HostageState::Captive | HostageState::RunningToJeep | HostageState::Riding { .. }
            )
        });

        if self.mission.total_hostages > 0 && !pending {
            self.mission.victory = true;
        }
    }

    fn damage_player(&mut self) {
        self.player.hp -= 1;
        self.player.invuln_timer = 1.0;
        self.explosions.push(Explosion::new(self.player.pos));

        if self.player.hp > 0 {
            return;
        }

        if self.mission.lives > 0 {
            self.mission.lives -= 1;
        }

        if self.mission.lives == 0 {
            self.mission.game_over = true;
            return;
        }

        for hostage in &mut self.hostages {
            if let HostageState::Riding { slot } = hostage.state {
                hostage.state = HostageState::RunningToJeep;
                hostage.pos = self.player.pos + rider_offset(slot);
            }
        }

        self.player.pos = self.player_spawn;
        self.player.hp = 3;
        self.player.invuln_timer = 2.0;
    }

    fn normalize_rider_slots(&mut self) {
        let mut slot = 0;
        for hostage in &mut self.hostages {
            if matches!(hostage.state, HostageState::Riding { .. }) {
                hostage.state = HostageState::Riding { slot };
                slot += 1;
            }
        }
    }
}

fn dominant_axis_step(delta: Vec2) -> Vec2 {
    if delta.x.abs() > delta.y.abs() {
        vec2(delta.x.signum(), 0.0)
    } else {
        vec2(0.0, delta.y.signum())
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

#[allow(dead_code)]
fn _direction_from_step(step: Vec2) -> Direction {
    if step.x > 0.0 {
        Direction::Right
    } else if step.x < 0.0 {
        Direction::Left
    } else if step.y > 0.0 {
        Direction::Down
    } else {
        Direction::Up
    }
}

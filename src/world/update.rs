use crate::constants::{
    BULLET_SPEED, ENEMY_BULLET_SPEED, ENEMY_FIRE_COOLDOWN, ENEMY_FIRE_RANGE, ENEMY_SHOOT_DURATION,
    JEEP_ACCEL, JEEP_BRAKE, PLAYER_FIRE_COOLDOWN,
};
use crate::entities::{Bullet, BulletOwner, Direction, EnemyAnimState};
use crate::input::PlayerCommand;
use crate::world::{World, rect_from_center};
use macroquad::prelude::*;

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
            if !self.map.collides_rect(rect_x) {
                enemy.pos.x = attempt_x.x;
            }

            let attempt_y = enemy.pos + vec2(0.0, step_dir.y * enemy.speed * dt);
            let rect_y = rect_from_center(attempt_y, enemy.size());
            if !self.map.collides_rect(rect_y) {
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

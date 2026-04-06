use crate::assets::{Assets, Facing4SpriteId, Facing8SpriteId, SpriteAsset, StaticSpriteId};
use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::entities::{BulletOwner, Enemy, EnemyKind, Facing4, Facing8, Pow};
use crate::world::{World, rect_from_center};
use macroquad::prelude::*;

pub fn draw_world(assets: &Assets, world: &World, top_left: Vec2, alpha: f32) {
    draw_imported_map(assets, world, top_left);
    draw_barracks(assets, world, top_left);
    draw_enemies(assets, world, top_left, alpha);
    draw_pows(assets, world, top_left, alpha);
    draw_bullets(world, top_left, alpha);

    let jeep_pos = snap_screen_pos(world_to_screen(world.player.render_pos(alpha), top_left));
    let jeep_size = world.player.render_size();
    draw_ellipse(
        jeep_pos.x,
        jeep_pos.y + jeep_size.y * 0.22,
        jeep_size.x * 0.26,
        jeep_size.y * 0.085,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );
    draw_sprite_centered_sized(
        assets.facing4_sprite(
            Facing4SpriteId::Jeep,
            Facing4::from_direction(world.player.dir),
        ),
        jeep_pos,
        jeep_size,
        WHITE,
    );
}

pub fn draw_collision_boxes(world: &World, top_left: Vec2, alpha: f32) {
    draw_collision_tiles(world, top_left);

    draw_collision_rect(
        rect_from_center(world.player.render_pos(alpha), world.player.size()),
        top_left,
        color_u8!(80, 180, 255, 70),
        color_u8!(140, 220, 255, 210),
    );

    for enemy in &world.enemies {
        draw_collision_rect(
            rect_from_center(enemy.render_pos(alpha), enemy.size()),
            top_left,
            color_u8!(255, 120, 120, 70),
            color_u8!(255, 180, 180, 220),
        );
    }

    for barracks in &world.barracks {
        draw_collision_rect(
            rect_from_center(barracks.pos, barracks.size()),
            top_left,
            color_u8!(255, 210, 120, 70),
            color_u8!(255, 230, 160, 220),
        );
    }

    for pow in &world.pows {
        draw_collision_rect(
            rect_from_center(pow.render_pos(alpha), pow.size()),
            top_left,
            color_u8!(140, 220, 255, 70),
            color_u8!(190, 240, 255, 220),
        );
    }

    for bullet in &world.bullets {
        let pos = bullet.render_pos(alpha);
        draw_collision_rect(
            Rect::new(
                pos.x - bullet.radius,
                pos.y - bullet.radius,
                bullet.radius * 2.0,
                bullet.radius * 2.0,
            ),
            top_left,
            Color::new(1.0, 1.0, 1.0, 0.16),
            Color::new(1.0, 1.0, 1.0, 0.55),
        );
    }
}

fn draw_collision_tiles(world: &World, top_left: Vec2) {
    let visible_world = Rect::new(top_left.x, top_left.y, VIEW_WIDTH, VIEW_HEIGHT);
    for span in world.map.collision_spans_in_rect(visible_world) {
        let screen_pos = world_to_screen(vec2(span.x, span.y), top_left);
        draw_rectangle(
            screen_pos.x,
            screen_pos.y,
            span.w,
            span.h,
            color_u8!(120, 255, 140, 70),
        );
    }
}

fn draw_enemies(assets: &Assets, world: &World, top_left: Vec2, alpha: f32) {
    let player_pos = world.player.render_pos(alpha);
    for enemy in &world.enemies {
        let pos = snap_screen_pos(world_to_screen(enemy.render_pos(alpha), top_left));
        let size = enemy.render_size();
        draw_ellipse(
            pos.x,
            pos.y + size.y * 0.24,
            size.x * 0.2,
            size.y * 0.07,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.14),
        );
        draw_enemy(
            assets,
            enemy,
            // Turrets use the live vector to the player for 8-way aiming. The
            // soldier path ignores this and still uses its 4-way animation set.
            player_pos - enemy.render_pos(alpha),
            pos,
            size,
        );
    }
}

fn draw_barracks(assets: &Assets, world: &World, top_left: Vec2) {
    for barracks in &world.barracks {
        let pos = snap_screen_pos(world_to_screen(barracks.pos, top_left));
        let size = barracks.render_size();
        draw_ellipse(
            pos.x,
            pos.y + size.y * 0.28,
            size.x * 0.28,
            size.y * 0.1,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.18),
        );
        let sprite = if barracks.is_destroyed() {
            assets.static_sprite(
                StaticSpriteId::BarracksDestroyed,
                static_sprite_seed(barracks.pos, 1),
            )
        } else {
            assets.static_sprite(
                StaticSpriteId::Barracks,
                static_sprite_seed(barracks.pos, 0),
            )
        };
        draw_sprite_centered_sized(sprite, pos, size, WHITE);
        if !barracks.is_destroyed() {
            draw_barracks_help_text(pos, barracks.pos);
        }
    }
}

fn draw_pows(assets: &Assets, world: &World, top_left: Vec2, alpha: f32) {
    for pow in &world.pows {
        let pos = snap_screen_pos(world_to_screen(pow.render_pos(alpha), top_left));
        let size = pow.render_size();
        draw_ellipse(
            pos.x,
            pos.y + size.y * 0.24,
            size.x * 0.18,
            size.y * 0.065,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.12),
        );
        draw_pow(assets, pow, pos, size);
    }
}

fn draw_bullets(world: &World, top_left: Vec2, alpha: f32) {
    for bullet in &world.bullets {
        let pos = snap_screen_pos(world_to_screen(bullet.render_pos(alpha), top_left));
        let visual_radius = bullet.radius * 0.5;
        let fill = match bullet.owner {
            BulletOwner::Player => WHITE,
            BulletOwner::Enemy => color_u8!(255, 120, 90, 255),
        };
        let outline = match bullet.owner {
            BulletOwner::Player => BLACK,
            BulletOwner::Enemy => color_u8!(90, 20, 10, 255),
        };
        draw_circle(pos.x, pos.y, visual_radius + 1.4, outline);
        draw_circle(pos.x, pos.y, visual_radius + 0.4, fill);
    }
}

fn draw_imported_map(assets: &Assets, world: &World, top_left: Vec2) {
    let (min_x, max_x, min_y, max_y) = visible_tile_bounds(world, top_left, 1);
    let tile_size = world.map.tile_size;
    let atlas_cols = ((assets.atlas().width() / tile_size).round() as u16).max(1);

    for layer in world.map.layers.iter().rev() {
        for tile in &layer.tiles {
            if tile.pos.x < min_x
                || tile.pos.x >= max_x
                || tile.pos.y < min_y
                || tile.pos.y >= max_y
            {
                continue;
            }

            let screen_pos =
                world.map.tile_center(tile.pos) - top_left - vec2(tile_size * 0.5, tile_size * 0.5);
            let atlas_id = tile.atlas_id;
            let src_col = atlas_id % atlas_cols;
            let src_row = atlas_id / atlas_cols;
            draw_texture_ex(
                assets.atlas(),
                screen_pos.x.floor(),
                screen_pos.y.floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(tile_size, tile_size)),
                    source: Some(Rect::new(
                        src_col as f32 * tile_size,
                        src_row as f32 * tile_size,
                        tile_size,
                        tile_size,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}

fn draw_sprite_top_left(sprite: &SpriteAsset, top_left: Vec2, tint: Color) {
    draw_texture_ex(
        &sprite.texture,
        top_left.x,
        top_left.y,
        tint,
        DrawTextureParams {
            source: sprite.source,
            dest_size: Some(sprite.draw_size),
            ..Default::default()
        },
    );
}

fn draw_sprite_centered(sprite: &SpriteAsset, center: Vec2, tint: Color) {
    let top_left = center - sprite.anchor;
    draw_sprite_top_left(sprite, top_left, tint);
}

fn draw_sprite_centered_sized(sprite: &SpriteAsset, center: Vec2, size: Vec2, tint: Color) {
    let scale = vec2(size.x / sprite.draw_size.x, size.y / sprite.draw_size.y);
    let scaled_anchor = vec2(sprite.anchor.x * scale.x, sprite.anchor.y * scale.y);
    let top_left = center - scaled_anchor;
    draw_texture_ex(
        &sprite.texture,
        top_left.x,
        top_left.y,
        tint,
        DrawTextureParams {
            source: sprite.source,
            dest_size: Some(size),
            ..Default::default()
        },
    );
}

fn world_to_screen(world_pos: Vec2, top_left: Vec2) -> Vec2 {
    vec2(world_pos.x - top_left.x, world_pos.y - top_left.y)
}

fn snap_screen_pos(screen_pos: Vec2) -> Vec2 {
    vec2(screen_pos.x.floor(), screen_pos.y.floor())
}

fn draw_collision_rect(rect: Rect, top_left: Vec2, fill: Color, outline: Color) {
    let screen_pos = world_to_screen(vec2(rect.x, rect.y), top_left);
    draw_rectangle(screen_pos.x, screen_pos.y, rect.w, rect.h, fill);
    draw_rectangle_lines(screen_pos.x, screen_pos.y, rect.w, rect.h, 1.0, outline);
}

fn visible_tile_bounds(world: &World, top_left: Vec2, padding_tiles: i32) -> (i32, i32, i32, i32) {
    let tile_size = world.map.tile_size;
    let min_x =
        ((top_left.x / tile_size).floor() as i32 - padding_tiles).clamp(0, world.map.width as i32);
    let max_x = (((top_left.x + VIEW_WIDTH) / tile_size).ceil() as i32 + padding_tiles + 1)
        .clamp(0, world.map.width as i32);
    let min_y =
        ((top_left.y / tile_size).floor() as i32 - padding_tiles).clamp(0, world.map.height as i32);
    let max_y = (((top_left.y + VIEW_HEIGHT) / tile_size).ceil() as i32 + padding_tiles + 1)
        .clamp(0, world.map.height as i32);
    (min_x, max_x, min_y, max_y)
}

fn draw_enemy(assets: &Assets, enemy: &Enemy, aim_delta: Vec2, pos: Vec2, size: Vec2) {
    match enemy.kind {
        EnemyKind::Soldier => draw_sprite_centered_sized(
            assets.animated_facing4_sprite(
                Facing4SpriteId::Soldier,
                Facing4::from_direction(enemy.dir),
                enemy.animation_state,
                enemy.walk_frame_index(),
            ),
            pos,
            size,
            WHITE,
        ),
        EnemyKind::Turret => {
            let sprite = if enemy.is_destroyed() {
                assets.static_sprite(
                    StaticSpriteId::TurretDestroyed,
                    static_sprite_seed(enemy.pos, 2),
                )
            } else {
                assets.facing8_sprite(Facing8SpriteId::Turret, Facing8::from_vec(aim_delta))
            };
            draw_sprite_centered_sized(sprite, pos, size, WHITE);
        }
    }
}

fn draw_pow(assets: &Assets, pow: &Pow, pos: Vec2, size: Vec2) {
    draw_sprite_centered_sized(
        assets.animated_facing4_sprite(
            Facing4SpriteId::Pow,
            Facing4::from_direction(pow.dir),
            pow.animation_state,
            pow.walk_frame_index(),
        ),
        pos,
        size,
        WHITE,
    );
}

fn draw_barracks_help_text(screen_pos: Vec2, world_pos: Vec2) {
    let pulse =
        (((get_time() as f32) * 2.4) + world_pos.x * 0.02 + world_pos.y * 0.015).sin() * 0.5 + 0.5;
    let alpha = pulse.powf(2.2);
    if alpha < 0.14 {
        return;
    }

    let text = "HELP";
    let font_size = 16;
    let metrics = measure_text(text, None, font_size, 1.0);
    let label_center = screen_pos - vec2(0.0, 4.0);
    let text_pos = vec2(
        label_center.x - metrics.width * 0.5,
        label_center.y + metrics.height * 0.35,
    );

    draw_text_ex(
        text,
        text_pos.x + 1.0,
        text_pos.y + 1.0,
        TextParams {
            font_size,
            color: Color::new(0.0, 0.0, 0.0, alpha * 0.55),
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        text_pos.x,
        text_pos.y,
        TextParams {
            font_size,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        },
    );
}

fn static_sprite_seed(pos: Vec2, salt: u64) -> u64 {
    let x = pos.x.round().to_bits() as u64;
    let y = pos.y.round().to_bits() as u64;
    x.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ y.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ salt
}

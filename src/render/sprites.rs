use crate::assets::Assets;
use crate::constants::{TILE_SIZE, VIEW_HEIGHT, VIEW_WIDTH};
use crate::entities::{Direction, HostageState};
use crate::world::{TileKind, World};
use macroquad::prelude::*;

pub fn viewport_scale() -> (f32, Vec2) {
    let raw_scale = (screen_width() / VIEW_WIDTH).min(screen_height() / VIEW_HEIGHT);
    let scale = if raw_scale < 1.0 {
        raw_scale
    } else {
        raw_scale.floor().max(1.0)
    };
    let size = vec2(VIEW_WIDTH * scale, VIEW_HEIGHT * scale);
    let origin = vec2(
        (screen_width() - size.x) * 0.5,
        (screen_height() - size.y) * 0.5,
    );
    (scale, origin)
}

pub fn draw_world(assets: &Assets, world: &World, top_left: Vec2, editor_mode: bool) {
    draw_tiles(assets, world, top_left, editor_mode);
    draw_entities(assets, world, top_left);
}

fn draw_tiles(assets: &Assets, world: &World, top_left: Vec2, editor_mode: bool) {
    for y in 0..world.map.height {
        for x in 0..world.map.width {
            let tile_pos = ivec2(x as i32, y as i32);
            let tile = &world.map.tiles[y * world.map.width + x];
            let screen_pos = world.map.tile_center(tile_pos) - top_left - vec2(8.0, 8.0);

            draw_metasprite(
                assets,
                tile_sprite_name(tile.kind, editor_mode),
                vec2(screen_pos.x.floor(), screen_pos.y.floor()),
                WHITE,
            );

            if matches!(tile.kind, TileKind::Wall | TileKind::HostageCage) {
                draw_rectangle_lines(
                    screen_pos.x.floor(),
                    screen_pos.y.floor(),
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    color_u8!(44, 38, 32, 255),
                );
            }

            if tile.kind == TileKind::Extraction {
                draw_rectangle_lines(
                    screen_pos.x.floor() + 2.0,
                    screen_pos.y.floor() + 2.0,
                    TILE_SIZE - 4.0,
                    TILE_SIZE - 4.0,
                    2.0,
                    color_u8!(180, 255, 180, 255),
                );
            }

            if editor_mode {
                draw_rectangle_lines(
                    screen_pos.x.floor(),
                    screen_pos.y.floor(),
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.10),
                );
            }
        }
    }
}

fn draw_entities(assets: &Assets, world: &World, top_left: Vec2) {
    for hostage in &world.hostages {
        if matches!(hostage.state, HostageState::Rescued) {
            continue;
        }
        let pos = world_to_screen(hostage.pos, top_left);
        draw_metasprite_centered(assets, "hostage", pos, WHITE);
    }

    for enemy in &world.enemies {
        let pos = world_to_screen(enemy.pos, top_left);
        draw_metasprite_centered(assets, "enemy_soldier", pos, WHITE);
    }

    for bullet in &world.bullets {
        let pos = bullet.pos - top_left;
        draw_rectangle(
            (pos.x - 1.0).floor(),
            (pos.y - 1.0).floor(),
            3.0,
            3.0,
            WHITE,
        );
    }

    for explosion in &world.explosions {
        let pos = world_to_screen(explosion.pos, top_left);
        draw_metasprite_centered(assets, "explosion", pos, WHITE);
    }

    let pos = world_to_screen(world.player.pos, top_left);
    let jeep_tint = if world.player.invuln_timer > 0.0 {
        Color::new(1.0, 1.0, 1.0, 0.7)
    } else {
        WHITE
    };
    let shadow_pos = pos + vec2(1.0, 1.0);
    draw_metasprite_centered(
        assets,
        jeep_sprite_name(world.player.dir),
        shadow_pos,
        Color::new(0.0, 0.0, 0.0, 0.45),
    );
    draw_metasprite_centered(assets, jeep_sprite_name(world.player.dir), pos, jeep_tint);
}

fn draw_metasprite(assets: &Assets, sprite_name: &str, pos: Vec2, tint: Color) {
    let meta = assets.metasprite(sprite_name);
    for part in &meta.parts {
        let region = assets.subtile_region(&part.tile);
        draw_texture_ex(
            assets.texture(),
            (pos.x + part.x).floor(),
            (pos.y + part.y).floor(),
            tint,
            DrawTextureParams {
                source: Some(region),
                dest_size: Some(vec2(region.w, region.h)),
                ..Default::default()
            },
        );
    }
}

fn draw_metasprite_centered(assets: &Assets, sprite_name: &str, center: Vec2, tint: Color) {
    let meta = assets.metasprite(sprite_name);
    let top_left = vec2(center.x - meta.w * 0.5, center.y - meta.h * 0.5);
    draw_metasprite(assets, sprite_name, top_left, tint);
}

fn world_to_screen(world_pos: Vec2, top_left: Vec2) -> Vec2 {
    vec2((world_pos.x - top_left.x).floor(), (world_pos.y - top_left.y).floor())
}

fn tile_sprite_name(kind: TileKind, editor_mode: bool) -> &'static str {
    match kind {
        TileKind::Grass => "grass_tile",
        TileKind::Road => "road_tile",
        TileKind::Water => "water_tile",
        TileKind::Wall => "wall_tile",
        TileKind::Rubble => "road_tile",
        TileKind::HostageCage => "cage_tile",
        TileKind::Extraction => "extraction_tile",
        TileKind::EnemySpawn if editor_mode => "wall_tile",
        TileKind::EnemySpawn => "grass_tile",
        TileKind::PlayerSpawn if editor_mode => "road_tile",
        TileKind::PlayerSpawn => "grass_tile",
    }
}

fn jeep_sprite_name(dir: Direction) -> &'static str {
    match dir {
        Direction::Up => "jeep_up",
        Direction::Down => "jeep_down",
        Direction::Left => "jeep_left",
        Direction::Right => "jeep_right",
    }
}

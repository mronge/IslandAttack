use crate::assets::Assets;
use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::entities::Direction;
use crate::world::World;
use macroquad::prelude::*;

pub fn draw_world(assets: &Assets, world: &World, top_left: Vec2, alpha: f32) {
    draw_imported_map(assets, world, top_left);

    let jeep_pos = world_to_screen(world.player.render_pos(alpha), top_left);
    let jeep_size = vec2(world.map.tile_size, world.map.tile_size);
    draw_ellipse(
        jeep_pos.x,
        jeep_pos.y + jeep_size.y * 0.22,
        jeep_size.x * 0.26,
        jeep_size.y * 0.085,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );
    draw_sprite_centered_sized(assets, jeep_sprite_name(world.player.dir), jeep_pos, jeep_size, WHITE);
}

fn draw_imported_map(assets: &Assets, world: &World, top_left: Vec2) {
    let (min_x, max_x, min_y, max_y) = visible_tile_bounds(world, top_left, 1);
    let tile_size = world.map.tile_size;
    let atlas_cols = ((assets.atlas().width() / tile_size).round() as u16).max(1);

    for layer in world.map.layers.iter().rev() {
        for tile in &layer.tiles {
            if tile.pos.x < min_x || tile.pos.x >= max_x || tile.pos.y < min_y || tile.pos.y >= max_y
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

fn draw_sprite_top_left(assets: &Assets, sprite_name: &str, top_left: Vec2, tint: Color) {
    let sprite = assets.sprite(sprite_name);
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

fn draw_sprite_centered(assets: &Assets, sprite_name: &str, center: Vec2, tint: Color) {
    let sprite = assets.sprite(sprite_name);
    let top_left = center - sprite.anchor;
    draw_sprite_top_left(assets, sprite_name, top_left, tint);
}

fn draw_sprite_centered_sized(
    assets: &Assets,
    sprite_name: &str,
    center: Vec2,
    size: Vec2,
    tint: Color,
) {
    let sprite = assets.sprite(sprite_name);
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

fn jeep_sprite_name(dir: Direction) -> &'static str {
    match dir {
        Direction::Up => "jeep_up",
        Direction::Down => "jeep_down",
        Direction::Left => "jeep_left",
        Direction::Right => "jeep_right",
    }
}

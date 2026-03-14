use crate::assets::Assets;
use crate::constants::{TILE_SIZE, VIEW_HEIGHT, VIEW_WIDTH};
use crate::entities::{BulletKind, BulletOwner, Direction, EnemyKind, HostageState};
use crate::world::{TileKind, World};
use macroquad::prelude::*;

pub fn viewport_scale() -> (f32, Vec2) {
    let scale = (screen_width() / VIEW_WIDTH)
        .min(screen_height() / VIEW_HEIGHT)
        .max(0.1);
    let size = vec2(VIEW_WIDTH * scale, VIEW_HEIGHT * scale);
    let origin = vec2(
        (screen_width() - size.x) * 0.5,
        (screen_height() - size.y) * 0.5,
    );
    (scale, origin)
}

pub fn draw_world(assets: &Assets, world: &World, top_left: Vec2, editor_mode: bool, alpha: f32) {
    draw_ground_tiles(assets, world, top_left, editor_mode);
    draw_prop_layer(assets, world, top_left, editor_mode);
    draw_entities(assets, world, top_left, alpha);
}

fn draw_ground_tiles(assets: &Assets, world: &World, top_left: Vec2, editor_mode: bool) {
    let (min_x, max_x, min_y, max_y) = visible_tile_bounds(world, top_left, 0);
    for y in min_y..max_y {
        for x in min_x..max_x {
            let tile_pos = ivec2(x, y);
            let tile = &world.map.tiles[y as usize * world.map.width + x as usize];
            let screen_pos =
                world.map.tile_center(tile_pos) - top_left - vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);
            draw_sprite_top_left_snapped(assets, grass_sprite_name(tile_pos), screen_pos, WHITE);

            if let Some(sprite_name) = terrain_overlay_name(world, tile_pos, tile.kind) {
                draw_sprite_top_left_snapped(assets, &sprite_name, screen_pos, WHITE);
            }

            if editor_mode {
                draw_rectangle_lines(
                    screen_pos.x,
                    screen_pos.y,
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.12),
                );
            }
        }
    }
}

fn draw_prop_layer(assets: &Assets, world: &World, top_left: Vec2, editor_mode: bool) {
    let (min_x, max_x, min_y, max_y) = visible_tile_bounds(world, top_left, 2);
    for y in min_y..max_y {
        for x in min_x..max_x {
            let tile_pos = ivec2(x, y);
            let tile = &world.map.tiles[y as usize * world.map.width + x as usize];
            let tile_center = world.map.tile_center(tile_pos);
            let tile_top_left = tile_center - top_left - vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);

            if should_draw_palm(world, tile_pos, tile.kind) {
                let palm_base = vec2(tile_center.x, tile_center.y + TILE_SIZE * 0.15);
                draw_sprite_anchored(
                    assets,
                    "palm_tree",
                    world_to_screen(palm_base, top_left),
                    WHITE,
                );
            }

            match tile.kind {
                TileKind::Wall => {
                    draw_sprite_top_left_snapped(
                        assets,
                        wall_sprite_name(tile_pos),
                        tile_top_left,
                        WHITE,
                    );
                    if is_bunker_origin(world, tile_pos) {
                        let bunker_center = tile_center + vec2(TILE_SIZE * 0.5, TILE_SIZE * 0.5);
                        draw_sprite_centered(
                            assets,
                            "bunker_turret",
                            world_to_screen(bunker_center, top_left),
                            WHITE,
                        );
                    }
                }
                TileKind::HostageCage => {
                    draw_sprite_top_left_snapped(assets, "cage_tile", tile_top_left, WHITE);
                }
                TileKind::Extraction => {
                    draw_sprite_top_left_snapped(assets, "extraction_tile", tile_top_left, WHITE);
                }
                TileKind::EnemySpawn if editor_mode => {
                    draw_editor_marker(
                        tile_top_left,
                        Color::new(0.86, 0.24, 0.24, 0.45),
                        Color::new(1.0, 0.83, 0.6, 0.9),
                    );
                }
                TileKind::PlayerSpawn if editor_mode => {
                    draw_editor_marker(
                        tile_top_left,
                        Color::new(0.18, 0.72, 0.3, 0.42),
                        Color::new(0.9, 1.0, 0.82, 0.9),
                    );
                }
                _ => {}
            }
        }
    }
}

fn draw_entities(assets: &Assets, world: &World, top_left: Vec2, alpha: f32) {
    for hostage in &world.hostages {
        if matches!(hostage.state, HostageState::Rescued) {
            continue;
        }
        draw_sprite_centered(
            assets,
            "hostage",
            world_to_screen(hostage.render_pos(alpha), top_left),
            WHITE,
        );
    }

    for enemy in &world.enemies {
        let pos = world_to_screen(enemy.render_pos(alpha), top_left);
        let to_player = world.player.render_pos(alpha) - enemy.render_pos(alpha);
        let rotation = enemy_rotation(to_player);
        draw_sprite_centered_rotated(assets, enemy_sprite_name(enemy.kind), pos, rotation, WHITE);
    }

    for bullet in &world.bullets {
        let pos = world_to_screen(bullet.render_pos(alpha), top_left);
        match bullet.kind {
            BulletKind::Normal => {
                let (outer, inner) = match bullet.owner {
                    BulletOwner::Player => {
                        (color_u8!(245, 245, 230, 255), color_u8!(255, 190, 90, 255))
                    }
                    BulletOwner::Enemy => {
                        (color_u8!(255, 210, 210, 255), color_u8!(255, 80, 60, 255))
                    }
                };
                draw_circle(pos.x, pos.y, 2.5, outer);
                draw_circle(pos.x, pos.y, 1.0, inner);
            }
            BulletKind::Rocket => {
                let dir = if bullet.vel.length_squared() > 0.0 {
                    bullet.vel.normalize()
                } else {
                    vec2(1.0, 0.0)
                };
                let tail = pos - dir * 4.5;
                draw_line(
                    tail.x,
                    tail.y,
                    pos.x,
                    pos.y,
                    2.0,
                    color_u8!(120, 48, 32, 220),
                );
                draw_circle(pos.x, pos.y, 3.0, color_u8!(255, 228, 190, 255));
                draw_circle(pos.x, pos.y, 1.5, color_u8!(255, 106, 48, 255));
            }
        }
    }

    for explosion in &world.explosions {
        draw_sprite_centered(
            assets,
            "explosion",
            world_to_screen(explosion.pos, top_left),
            WHITE,
        );
    }

    let jeep_pos = world_to_screen(world.player.render_pos(alpha), top_left);
    let jeep_sprite = assets.sprite(jeep_sprite_name(world.player.dir));
    draw_ellipse(
        jeep_pos.x,
        jeep_pos.y + jeep_sprite.draw_size.y * 0.26,
        jeep_sprite.draw_size.x * 0.26,
        jeep_sprite.draw_size.y * 0.085,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );
    draw_sprite_centered(
        assets,
        jeep_sprite_name(world.player.dir),
        jeep_pos,
        if world.player.invuln_timer > 0.0 {
            Color::new(1.0, 1.0, 1.0, 0.7)
        } else {
            WHITE
        },
    );
}

fn draw_editor_marker(top_left: Vec2, fill: Color, stroke: Color) {
    draw_rectangle(top_left.x, top_left.y, TILE_SIZE, TILE_SIZE, fill);
    draw_rectangle_lines(top_left.x, top_left.y, TILE_SIZE, TILE_SIZE, 4.0, stroke);
}

fn draw_sprite_top_left(assets: &Assets, sprite_name: &str, top_left: Vec2, tint: Color) {
    let sprite = assets.sprite(sprite_name);
    draw_texture_ex(
        &sprite.texture,
        top_left.x,
        top_left.y,
        tint,
        DrawTextureParams {
            dest_size: Some(sprite.draw_size),
            ..Default::default()
        },
    );
}

fn draw_sprite_top_left_snapped(assets: &Assets, sprite_name: &str, top_left: Vec2, tint: Color) {
    draw_sprite_top_left(
        assets,
        sprite_name,
        vec2(top_left.x.floor(), top_left.y.floor()),
        tint,
    );
}

fn draw_sprite_anchored(assets: &Assets, sprite_name: &str, anchor_world: Vec2, tint: Color) {
    let sprite = assets.sprite(sprite_name);
    let top_left = anchor_world - sprite.anchor;
    draw_sprite_top_left(assets, sprite_name, top_left, tint);
}

fn draw_sprite_centered(assets: &Assets, sprite_name: &str, center: Vec2, tint: Color) {
    let sprite = assets.sprite(sprite_name);
    let top_left = center - sprite.anchor;
    draw_sprite_top_left(assets, sprite_name, top_left, tint);
}

fn draw_sprite_centered_rotated(
    assets: &Assets,
    sprite_name: &str,
    center: Vec2,
    rotation: f32,
    tint: Color,
) {
    let sprite = assets.sprite(sprite_name);
    let top_left = center - sprite.anchor;
    draw_texture_ex(
        &sprite.texture,
        top_left.x,
        top_left.y,
        tint,
        DrawTextureParams {
            dest_size: Some(sprite.draw_size),
            rotation,
            pivot: Some(center),
            ..Default::default()
        },
    );
}

fn world_to_screen(world_pos: Vec2, top_left: Vec2) -> Vec2 {
    vec2(world_pos.x - top_left.x, world_pos.y - top_left.y)
}

fn visible_tile_bounds(world: &World, top_left: Vec2, padding_tiles: i32) -> (i32, i32, i32, i32) {
    let min_x =
        ((top_left.x / TILE_SIZE).floor() as i32 - padding_tiles).clamp(0, world.map.width as i32);
    let max_x = (((top_left.x + VIEW_WIDTH) / TILE_SIZE).ceil() as i32 + padding_tiles + 1)
        .clamp(0, world.map.width as i32);
    let min_y =
        ((top_left.y / TILE_SIZE).floor() as i32 - padding_tiles).clamp(0, world.map.height as i32);
    let max_y = (((top_left.y + VIEW_HEIGHT) / TILE_SIZE).ceil() as i32 + padding_tiles + 1)
        .clamp(0, world.map.height as i32);
    (min_x, max_x, min_y, max_y)
}

fn grass_sprite_name(tile_pos: IVec2) -> &'static str {
    const GROUND: [&str; 6] = [
        "ground_0", "ground_1", "ground_2", "ground_3", "ground_4", "ground_5",
    ];
    let variant = terrain_hash(tile_pos).rem_euclid(GROUND.len() as i32) as usize;
    GROUND[variant]
}

fn terrain_overlay_name(world: &World, tile_pos: IVec2, kind: TileKind) -> Option<String> {
    match kind {
        TileKind::Road | TileKind::Rubble => Some(format!(
            "road_overlay_{}_{}",
            macro_terrain_hash(tile_pos, 4).rem_euclid(4),
            connection_mask(world, tile_pos, kind)
        )),
        TileKind::Water => Some(format!(
            "water_overlay_{}_{}",
            macro_terrain_hash(tile_pos, 4).rem_euclid(4),
            connection_mask(world, tile_pos, kind)
        )),
        _ => None,
    }
}

fn wall_sprite_name(tile_pos: IVec2) -> &'static str {
    let variant = ((tile_pos.x + tile_pos.y).rem_euclid(2)) as usize;
    if variant == 0 { "wall_0" } else { "wall_1" }
}

fn is_bunker_origin(world: &World, tile_pos: IVec2) -> bool {
    let neighbors = [
        tile_pos,
        tile_pos + ivec2(1, 0),
        tile_pos + ivec2(0, 1),
        tile_pos + ivec2(1, 1),
    ];
    if neighbors
        .iter()
        .any(|pos| world.map.tile_kind(*pos) != Some(TileKind::Wall))
    {
        return false;
    }

    world.map.tile_kind(tile_pos + ivec2(-1, 0)) != Some(TileKind::Wall)
        && world.map.tile_kind(tile_pos + ivec2(0, -1)) != Some(TileKind::Wall)
}

fn should_draw_palm(world: &World, tile_pos: IVec2, kind: TileKind) -> bool {
    if kind != TileKind::Grass {
        return false;
    }

    let hash = (tile_pos.x * 17 + tile_pos.y * 31).rem_euclid(19);
    if hash != 0 {
        return false;
    }

    for dy in -1..=1 {
        for dx in -1..=1 {
            let neighbor = tile_pos + ivec2(dx, dy);
            if world
                .map
                .tile_kind(neighbor)
                .is_some_and(|other| other != TileKind::Grass)
            {
                return false;
            }
        }
    }

    true
}

fn jeep_sprite_name(dir: Direction) -> &'static str {
    match dir {
        Direction::Up => "jeep_up",
        Direction::Down => "jeep_down",
        Direction::Left => "jeep_left",
        Direction::Right => "jeep_right",
    }
}

fn enemy_sprite_name(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Commando => "enemy_commando",
        EnemyKind::Rocketeer => "enemy_rocketeer",
    }
}

fn enemy_rotation(to_player: Vec2) -> f32 {
    if to_player.length_squared() <= 0.001 {
        0.0
    } else {
        to_player.y.atan2(to_player.x) + std::f32::consts::FRAC_PI_2
    }
}

fn terrain_hash(tile_pos: IVec2) -> i32 {
    tile_pos.x * 31 + tile_pos.y * 57 + tile_pos.x * tile_pos.y * 3
}

fn macro_terrain_hash(tile_pos: IVec2, region: i32) -> i32 {
    let cell = ivec2(tile_pos.x.div_euclid(region), tile_pos.y.div_euclid(region));
    terrain_hash(cell)
}

fn connection_mask(world: &World, tile_pos: IVec2, kind: TileKind) -> u8 {
    let same = |other: Option<TileKind>| match kind {
        TileKind::Road | TileKind::Rubble => {
            matches!(other, Some(TileKind::Road) | Some(TileKind::Rubble))
        }
        TileKind::Water => matches!(other, Some(TileKind::Water)),
        _ => false,
    };

    let mut mask = 0u8;
    if same(world.map.tile_kind(tile_pos + ivec2(0, -1))) {
        mask |= 0b0001;
    }
    if same(world.map.tile_kind(tile_pos + ivec2(1, 0))) {
        mask |= 0b0010;
    }
    if same(world.map.tile_kind(tile_pos + ivec2(0, 1))) {
        mask |= 0b0100;
    }
    if same(world.map.tile_kind(tile_pos + ivec2(-1, 0))) {
        mask |= 0b1000;
    }
    mask
}

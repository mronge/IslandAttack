use crate::constants::{MAP_PATH, MAP_SPRITESHEET_PATH};
use image::ImageReader;
use macroquad::prelude::{IVec2, Rect, Vec2, ivec2, vec2};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug)]
pub struct MapTile {
    pub atlas_id: u16,
    pub pos: IVec2,
}

#[derive(Clone, Debug)]
pub struct MapLayer {
    pub name: String,
    pub collider: bool,
    pub tiles: Vec<MapTile>,
}

#[derive(Clone, Debug)]
pub struct ImportedMap {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub layers: Vec<MapLayer>,
    preferred_spawn_tiles: Vec<bool>,
    collision_pixels: Vec<bool>,
}

#[derive(Deserialize)]
struct RawMap {
    #[serde(rename = "mapWidth")]
    map_width: usize,
    #[serde(rename = "mapHeight")]
    map_height: usize,
    #[serde(rename = "tileSize")]
    tile_size: u16,
    layers: Vec<RawLayer>,
}

#[derive(Deserialize)]
struct RawLayer {
    name: String,
    #[serde(default)]
    collider: bool,
    tiles: Vec<RawTile>,
}

#[derive(Deserialize)]
struct RawTile {
    id: String,
    x: i32,
    y: i32,
}

impl ImportedMap {
    pub fn load() -> Self {
        let raw = fs::read_to_string(MAP_PATH)
            .unwrap_or_else(|_| panic!("failed to read imported map: {MAP_PATH}"));
        let raw_map: RawMap =
            serde_json::from_str(&raw).unwrap_or_else(|_| panic!("failed to parse map: {MAP_PATH}"));

        let layers: Vec<MapLayer> = raw_map
            .layers
            .into_iter()
            .map(|layer| MapLayer {
                name: layer.name,
                collider: layer.collider,
                tiles: layer
                    .tiles
                    .into_iter()
                    .map(|tile| MapTile {
                        atlas_id: tile.id.parse().expect("invalid tile id in imported map"),
                        pos: ivec2(tile.x, tile.y),
                    })
                    .collect(),
            })
            .collect();

        let mut preferred_spawn_tiles = vec![false; raw_map.map_width * raw_map.map_height];

        // The exported map order is top-to-bottom, so walk it in reverse and let
        // upper layers overwrite lower ones to derive spawn preference from the visible tile.
        for layer in layers.iter().rev() {
            let is_solid = layer.collider;
            let is_preferred_spawn =
                !is_solid && layer_name_matches(&layer.name, &["sand", "ground", "grass"]);

            for tile in &layer.tiles {
                let Some(idx) = tile_index(raw_map.map_width, raw_map.map_height, tile.pos) else {
                    continue;
                };

                preferred_spawn_tiles[idx] = is_preferred_spawn;
            }
        }

        if !preferred_spawn_tiles.iter().any(|tile| *tile) {
            for layer in layers.iter().rev() {
                if layer.collider {
                    continue;
                }
                for tile in &layer.tiles {
                    let Some(idx) = tile_index(raw_map.map_width, raw_map.map_height, tile.pos) else {
                        continue;
                    };
                    preferred_spawn_tiles[idx] = true;
                }
            }
        }

        let tile_size_px = usize::from(raw_map.tile_size);
        let collision_pixels =
            build_collision_pixels(raw_map.map_width, raw_map.map_height, tile_size_px, &layers);

        Self {
            width: raw_map.map_width,
            height: raw_map.map_height,
            tile_size: f32::from(raw_map.tile_size),
            layers,
            preferred_spawn_tiles,
            collision_pixels,
        }
    }

    pub fn dimensions_px(&self) -> Vec2 {
        vec2(
            self.width as f32 * self.tile_size,
            self.height as f32 * self.tile_size,
        )
    }

    pub fn in_bounds(&self, tile: IVec2) -> bool {
        tile.x >= 0 && tile.y >= 0 && tile.x < self.width as i32 && tile.y < self.height as i32
    }

    pub fn tile_index(&self, tile: IVec2) -> Option<usize> {
        tile_index(self.width, self.height, tile)
    }

    pub fn tile_center(&self, tile: IVec2) -> Vec2 {
        vec2(
            tile.x as f32 * self.tile_size + self.tile_size * 0.5,
            tile.y as f32 * self.tile_size + self.tile_size * 0.5,
        )
    }

    pub fn collides_rect(&self, rect: Rect) -> bool {
        let pixel_width = self.width * self.tile_size as usize;
        let pixel_height = self.height * self.tile_size as usize;
        let min_x = rect.x.floor() as i32;
        let min_y = rect.y.floor() as i32;
        let max_x = (rect.x + rect.w - 0.001).floor() as i32;
        let max_y = (rect.y + rect.h - 0.001).floor() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x < 0 || y < 0 || x >= pixel_width as i32 || y >= pixel_height as i32 {
                    return true;
                }
                let idx = y as usize * pixel_width + x as usize;
                if self.collision_pixels[idx] {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_solid(&self, tile: IVec2) -> bool {
        let Some(idx) = self.tile_index(tile) else {
            return true;
        };
        let tile_size = self.tile_size as usize;
        let pixel_width = self.width * tile_size;
        let x0 = (idx % self.width) * tile_size;
        let y0 = (idx / self.width) * tile_size;
        for y in 0..tile_size {
            let row = (y0 + y) * pixel_width + x0;
            for x in 0..tile_size {
                if self.collision_pixels[row + x] {
                    return true;
                }
            }
        }
        false
    }

    pub fn default_spawn_point_for(&self, size: Vec2) -> Vec2 {
        let target = vec2(self.width as f32 * 0.5, self.height as f32 * 0.5);
        let mut best_tile = None;
        let mut best_distance = f32::MAX;

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let tile = ivec2(x, y);
                let Some(idx) = self.tile_index(tile) else {
                    continue;
                };
                if !self.preferred_spawn_tiles[idx] {
                    continue;
                }

                let center = self.tile_center(tile);
                let rect = Rect::new(
                    center.x - size.x * 0.5,
                    center.y - size.y * 0.5,
                    size.x,
                    size.y,
                );
                if self.collides_rect(rect) {
                    continue;
                }

                let tile_center = vec2(x as f32 + 0.5, y as f32 + 0.5);
                let distance = tile_center.distance_squared(target);
                if distance < best_distance {
                    best_distance = distance;
                    best_tile = Some(tile);
                }
            }
        }

        best_tile
            .map(|tile| self.tile_center(tile))
            .unwrap_or_else(|| self.dimensions_px() * 0.5)
    }

    pub fn enemy_spawn_points(
        &self,
        size: Vec2,
        count: usize,
        avoid_center: Vec2,
        min_distance: f32,
    ) -> Vec<Vec2> {
        let mut candidates = Vec::new();

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let tile = ivec2(x, y);
                let Some(idx) = self.tile_index(tile) else {
                    continue;
                };
                if !self.preferred_spawn_tiles[idx] {
                    continue;
                }

                let center = self.tile_center(tile);
                if center.distance(avoid_center) < min_distance {
                    continue;
                }

                let rect = Rect::new(
                    center.x - size.x * 0.5,
                    center.y - size.y * 0.5,
                    size.x,
                    size.y,
                );
                if self.collides_rect(rect) {
                    continue;
                }

                candidates.push(center);
            }
        }

        candidates.sort_by(|a, b| {
            b.distance_squared(avoid_center)
                .total_cmp(&a.distance_squared(avoid_center))
        });
        candidates.truncate(count);
        candidates
    }
}

fn tile_index(width: usize, height: usize, tile: IVec2) -> Option<usize> {
    (tile.x >= 0 && tile.y >= 0 && tile.x < width as i32 && tile.y < height as i32)
        .then_some(tile.y as usize * width + tile.x as usize)
}

fn layer_name_matches(name: &str, needles: &[&str]) -> bool {
    let lower = name.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

fn build_collision_pixels(
    map_width: usize,
    map_height: usize,
    tile_size: usize,
    layers: &[MapLayer],
) -> Vec<bool> {
    let atlas = ImageReader::open(MAP_SPRITESHEET_PATH)
        .unwrap_or_else(|_| panic!("failed to open map spritesheet: {MAP_SPRITESHEET_PATH}"))
        .decode()
        .unwrap_or_else(|_| panic!("failed to decode map spritesheet: {MAP_SPRITESHEET_PATH}"))
        .to_rgba8();
    let atlas_cols = (atlas.width() as usize / tile_size).max(1);
    let pixel_width = map_width * tile_size;
    let pixel_height = map_height * tile_size;
    let mut collision_pixels = vec![false; pixel_width * pixel_height];
    let mut topmost_water_pixels = vec![false; pixel_width * pixel_height];
    let mut masks = HashMap::<u16, Vec<bool>>::new();

    for layer in layers.iter().rev() {
        let is_water_layer = layer.collider && layer_name_matches(&layer.name, &["water"]);
        for tile in &layer.tiles {
            let Some(_) = tile_index(map_width, map_height, tile.pos) else {
                continue;
            };

            let mask = masks
                .entry(tile.atlas_id)
                .or_insert_with(|| tile_alpha_mask(&atlas, atlas_cols, tile_size, tile.atlas_id));
            let world_x = tile.pos.x as usize * tile_size;
            let world_y = tile.pos.y as usize * tile_size;

            for local_y in 0..tile_size {
                let src_row = local_y * tile_size;
                let dst_row = (world_y + local_y) * pixel_width + world_x;
                for local_x in 0..tile_size {
                    if mask[src_row + local_x] {
                        let idx = dst_row + local_x;
                        collision_pixels[idx] = layer.collider;
                        topmost_water_pixels[idx] = is_water_layer;
                    }
                }
            }
        }
    }

    // Shoreline tiles are cliffs. Treat any tile that still shows water after
    // layering as blocked, and also block the immediately adjacent land tiles
    // so the collider sits one tile in from the water.
    let mut water_tiles = vec![false; map_width * map_height];
    for tile_y in 0..map_height {
        for tile_x in 0..map_width {
            let x0 = tile_x * tile_size;
            let y0 = tile_y * tile_size;
            let mut has_visible_water = false;

            'scan: for local_y in 0..tile_size {
                let row = (y0 + local_y) * pixel_width + x0;
                for local_x in 0..tile_size {
                    if topmost_water_pixels[row + local_x] {
                        has_visible_water = true;
                        break 'scan;
                    }
                }
            }

            if has_visible_water {
                water_tiles[tile_y * map_width + tile_x] = true;
                for local_y in 0..tile_size {
                    let row = (y0 + local_y) * pixel_width + x0;
                    for local_x in 0..tile_size {
                        collision_pixels[row + local_x] = true;
                    }
                }
            }
        }
    }

    for tile_y in 0..map_height {
        for tile_x in 0..map_width {
            let idx = tile_y * map_width + tile_x;
            if water_tiles[idx] {
                continue;
            }

            let touches_water = [
                (tile_x as i32 - 1, tile_y as i32),
                (tile_x as i32 + 1, tile_y as i32),
                (tile_x as i32, tile_y as i32 - 1),
                (tile_x as i32, tile_y as i32 + 1),
                (tile_x as i32 - 1, tile_y as i32 - 1),
                (tile_x as i32 + 1, tile_y as i32 - 1),
                (tile_x as i32 - 1, tile_y as i32 + 1),
                (tile_x as i32 + 1, tile_y as i32 + 1),
            ]
            .into_iter()
            .any(|(nx, ny)| {
                nx >= 0
                    && ny >= 0
                    && nx < map_width as i32
                    && ny < map_height as i32
                    && water_tiles[ny as usize * map_width + nx as usize]
            });

            if !touches_water {
                continue;
            }

            let x0 = tile_x * tile_size;
            let y0 = tile_y * tile_size;
            for local_y in 0..tile_size {
                let row = (y0 + local_y) * pixel_width + x0;
                for local_x in 0..tile_size {
                    collision_pixels[row + local_x] = true;
                }
            }
        }
    }

    collision_pixels
}

fn tile_alpha_mask(
    atlas: &image::RgbaImage,
    atlas_cols: usize,
    tile_size: usize,
    atlas_id: u16,
) -> Vec<bool> {
    let mut mask = vec![false; tile_size * tile_size];
    let tile_x = (atlas_id as usize % atlas_cols) * tile_size;
    let tile_y = (atlas_id as usize / atlas_cols) * tile_size;

    for y in 0..tile_size {
        for x in 0..tile_size {
            let pixel = atlas.get_pixel((tile_x + x) as u32, (tile_y + y) as u32);
            mask[y * tile_size + x] = pixel[3] > 0;
        }
    }

    mask
}

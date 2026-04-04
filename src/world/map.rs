use crate::constants::{MAP_PATH, MAP_SPRITESHEET_PATH};
use crate::entities::EnemyKind;
use image::ImageReader;
use macroquad::prelude::{IVec2, Rect, Vec2, ivec2, vec2};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
    enemy_spawns: Vec<EnemySpawn>,
    barracks_spawns: Vec<BarracksSpawn>,
    preferred_spawn_tiles: Vec<bool>,
    collision_pixels: Vec<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemySpawn {
    pub tile: IVec2,
    pub kind: EnemyKind,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarracksSpawn {
    pub top_left: IVec2,
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
        let raw_map: RawMap = serde_json::from_str(&raw)
            .unwrap_or_else(|_| panic!("failed to parse map: {MAP_PATH}"));

        let mut layers = Vec::new();
        let mut enemy_spawns = Vec::new();
        let mut barracks_spawns = Vec::new();

        for layer in raw_map.layers {
            if is_enemy_spawn_layer(&layer.name) {
                enemy_spawns.extend(layer.tiles.into_iter().map(enemy_spawn_from_raw_tile));
                continue;
            }
            if is_barracks_layer(&layer.name) {
                barracks_spawns.extend(barracks_spawns_from_raw_tiles(layer.tiles));
                continue;
            }

            layers.push(MapLayer {
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
            });
        }

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
                    let Some(idx) = tile_index(raw_map.map_width, raw_map.map_height, tile.pos)
                    else {
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
            enemy_spawns,
            barracks_spawns,
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

    pub fn collides_point(&self, point: Vec2) -> bool {
        let pixel_width = self.width * self.tile_size as usize;
        let pixel_height = self.height * self.tile_size as usize;
        let x = point.x.floor() as i32;
        let y = point.y.floor() as i32;

        if x < 0 || y < 0 || x >= pixel_width as i32 || y >= pixel_height as i32 {
            return true;
        }

        self.collision_pixels[y as usize * pixel_width + x as usize]
    }

    pub fn has_line_of_sight(&self, from: Vec2, to: Vec2) -> bool {
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
            if self.collides_point(sample) {
                return false;
            }
        }

        !self.collides_point(to)
    }

    pub fn collision_spans_in_rect(&self, rect: Rect) -> Vec<Rect> {
        let pixel_width = self.width * self.tile_size as usize;
        let pixel_height = self.height * self.tile_size as usize;
        let min_x = rect.x.floor().max(0.0) as i32;
        let min_y = rect.y.floor().max(0.0) as i32;
        let max_x = (rect.x + rect.w).ceil().min(pixel_width as f32) as i32;
        let max_y = (rect.y + rect.h).ceil().min(pixel_height as f32) as i32;
        let mut spans = Vec::new();

        for y in min_y..max_y {
            let row_start = y as usize * pixel_width;
            let mut x = min_x;

            while x < max_x {
                let idx = row_start + x as usize;
                if !self.collision_pixels[idx] {
                    x += 1;
                    continue;
                }

                let span_start = x;
                x += 1;
                while x < max_x && self.collision_pixels[row_start + x as usize] {
                    x += 1;
                }

                spans.push(Rect::new(
                    span_start as f32,
                    y as f32,
                    (x - span_start) as f32,
                    1.0,
                ));
            }
        }

        spans
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

    pub fn enemy_spawns(&self) -> &[EnemySpawn] {
        &self.enemy_spawns
    }

    pub fn barracks_spawns(&self) -> &[BarracksSpawn] {
        &self.barracks_spawns
    }
}

fn tile_index(width: usize, height: usize, tile: IVec2) -> Option<usize> {
    (tile.x >= 0 && tile.y >= 0 && tile.x < width as i32 && tile.y < height as i32)
        .then_some(tile.y as usize * width + tile.x as usize)
}

fn enemy_spawn_from_raw_tile(tile: RawTile) -> EnemySpawn {
    match tile.id.as_str() {
        "0" | "4" => EnemySpawn {
            tile: ivec2(tile.x, tile.y),
            kind: EnemyKind::Soldier,
            count: 1,
        },
        "1" | "5" => EnemySpawn {
            tile: ivec2(tile.x, tile.y),
            kind: EnemyKind::Soldier,
            count: 2,
        },
        "2" | "6" => EnemySpawn {
            tile: ivec2(tile.x, tile.y),
            kind: EnemyKind::Turret,
            count: 1,
        },
        other => panic!("unknown enemy spawn tile id: {other}"),
    }
}

fn barracks_spawns_from_raw_tiles(tiles: Vec<RawTile>) -> Vec<BarracksSpawn> {
    let markers: HashMap<IVec2, String> = tiles
        .into_iter()
        .map(|tile| (ivec2(tile.x, tile.y), tile.id))
        .collect();
    let mut top_lefts = markers
        .iter()
        .filter_map(|(pos, id)| (id == "0").then_some(*pos))
        .collect::<Vec<_>>();
    let mut used = HashSet::new();
    let mut spawns = Vec::new();

    top_lefts.sort_by_key(|pos| (pos.y, pos.x));

    for top_left in top_lefts {
        let footprint = [
            (top_left, "0"),
            (top_left + ivec2(1, 0), "1"),
            (top_left + ivec2(0, 1), "2"),
            (top_left + ivec2(1, 1), "3"),
        ];

        for (pos, expected) in footprint {
            match markers.get(&pos).map(String::as_str) {
                Some(id) if id == expected => {
                    used.insert(pos);
                }
                Some(id) => {
                    panic!(
                        "invalid barracks marker at ({}, {}): expected id {}, found {}",
                        pos.x, pos.y, expected, id
                    );
                }
                None => {
                    panic!(
                        "missing barracks marker at ({}, {}): expected id {}",
                        pos.x, pos.y, expected
                    );
                }
            }
        }

        spawns.push(BarracksSpawn { top_left });
    }

    if used.len() != markers.len() {
        let stray = markers
            .keys()
            .find(|pos| !used.contains(pos))
            .copied()
            .expect("expected a stray barracks marker");
        panic!(
            "stray barracks marker at ({}, {}) is not part of a complete 2x2 footprint",
            stray.x, stray.y
        );
    }

    spawns
}

fn layer_name_matches(name: &str, needles: &[&str]) -> bool {
    let lower = name.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

fn is_enemy_spawn_layer(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "enemy" | "enemies"
    )
}

fn is_barracks_layer(name: &str) -> bool {
    matches!(name.trim().to_ascii_lowercase().as_str(), "barracks")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enemy_spawn_marker_ids() {
        assert_eq!(
            enemy_spawn_from_raw_tile(RawTile {
                id: "4".to_owned(),
                x: 1,
                y: 2,
            }),
            EnemySpawn {
                tile: ivec2(1, 2),
                kind: EnemyKind::Soldier,
                count: 1,
            }
        );
        assert_eq!(
            enemy_spawn_from_raw_tile(RawTile {
                id: "5".to_owned(),
                x: 3,
                y: 4,
            }),
            EnemySpawn {
                tile: ivec2(3, 4),
                kind: EnemyKind::Soldier,
                count: 2,
            }
        );
        assert_eq!(
            enemy_spawn_from_raw_tile(RawTile {
                id: "6".to_owned(),
                x: 5,
                y: 6,
            }),
            EnemySpawn {
                tile: ivec2(5, 6),
                kind: EnemyKind::Turret,
                count: 1,
            }
        );
        assert_eq!(
            enemy_spawn_from_raw_tile(RawTile {
                id: "1".to_owned(),
                x: 7,
                y: 8,
            })
            .count,
            2
        );
    }

    #[test]
    fn recognizes_enemy_spawn_layer_names() {
        assert!(is_enemy_spawn_layer("Enemy"));
        assert!(is_enemy_spawn_layer("Enemies"));
        assert!(is_enemy_spawn_layer(" enemies "));
        assert!(!is_enemy_spawn_layer("Land enemies"));
        assert!(!is_enemy_spawn_layer("Enemy markers"));
    }

    #[test]
    fn parses_barracks_spawn_blocks() {
        let spawns = barracks_spawns_from_raw_tiles(vec![
            RawTile {
                id: "0".to_owned(),
                x: 10,
                y: 11,
            },
            RawTile {
                id: "1".to_owned(),
                x: 11,
                y: 11,
            },
            RawTile {
                id: "2".to_owned(),
                x: 10,
                y: 12,
            },
            RawTile {
                id: "3".to_owned(),
                x: 11,
                y: 12,
            },
        ]);

        assert_eq!(
            spawns,
            vec![BarracksSpawn {
                top_left: ivec2(10, 11)
            }]
        );
    }

    #[test]
    fn recognizes_barracks_layer_name() {
        assert!(is_barracks_layer("Barracks"));
        assert!(is_barracks_layer(" barracks "));
        assert!(!is_barracks_layer("Barracks markers"));
    }
}

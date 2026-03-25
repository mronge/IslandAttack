use crate::constants::MAP_PATH;
use macroquad::prelude::{IVec2, Rect, Vec2, ivec2, vec2};
use serde::Deserialize;
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
    solid_tiles: Vec<bool>,
    preferred_spawn_tiles: Vec<bool>,
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

        let mut solid_tiles = vec![true; raw_map.map_width * raw_map.map_height];
        let mut walkable_tiles = vec![false; raw_map.map_width * raw_map.map_height];
        let mut preferred_spawn_tiles = vec![false; raw_map.map_width * raw_map.map_height];

        // The exported map order is top-to-bottom, so walk it in reverse and let
        // upper layers overwrite lower ones to derive gameplay from the visible tile.
        for layer in layers.iter().rev() {
            let is_solid = layer_is_solid(layer);
            let is_preferred_spawn =
                !is_solid && layer_name_matches(&layer.name, &["sand", "ground", "grass"]);

            for tile in &layer.tiles {
                let Some(idx) = tile_index(raw_map.map_width, raw_map.map_height, tile.pos) else {
                    continue;
                };

                solid_tiles[idx] = is_solid;
                walkable_tiles[idx] = !is_solid;
                preferred_spawn_tiles[idx] = is_preferred_spawn;
            }
        }

        if !preferred_spawn_tiles.iter().any(|tile| *tile) {
            preferred_spawn_tiles.clone_from(&walkable_tiles);
        }

        Self {
            width: raw_map.map_width,
            height: raw_map.map_height,
            tile_size: f32::from(raw_map.tile_size),
            layers,
            solid_tiles,
            preferred_spawn_tiles,
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
        let min = ivec2(
            (rect.x / self.tile_size).floor() as i32,
            (rect.y / self.tile_size).floor() as i32,
        );
        let max = ivec2(
            ((rect.x + rect.w - 0.001) / self.tile_size).floor() as i32,
            ((rect.y + rect.h - 0.001) / self.tile_size).floor() as i32,
        );

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let tile = ivec2(x, y);
                if !self.in_bounds(tile) || self.is_solid(tile) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_solid(&self, tile: IVec2) -> bool {
        self.tile_index(tile)
            .and_then(|idx| self.solid_tiles.get(idx))
            .copied()
            .unwrap_or(true)
    }

    pub fn default_spawn_point(&self) -> Vec2 {
        let target = vec2(self.width as f32 * 0.5, self.height as f32 * 0.5);
        let mut best_tile = None;
        let mut best_distance = f32::MAX;

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let tile = ivec2(x, y);
                let Some(idx) = self.tile_index(tile) else {
                    continue;
                };
                if !self.preferred_spawn_tiles[idx] || self.solid_tiles[idx] {
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
}

fn tile_index(width: usize, height: usize, tile: IVec2) -> Option<usize> {
    (tile.x >= 0 && tile.y >= 0 && tile.x < width as i32 && tile.y < height as i32)
        .then_some(tile.y as usize * width + tile.x as usize)
}

fn layer_is_solid(layer: &MapLayer) -> bool {
    layer.collider || layer_name_matches(&layer.name, &["water", "wall", "collision", "layer_2"])
}

fn layer_name_matches(name: &str, needles: &[&str]) -> bool {
    let lower = name.to_ascii_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

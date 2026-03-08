use crate::constants::TILE_SIZE;
use macroquad::prelude::{IVec2, Rect, Vec2, ivec2, vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileKind {
    Grass,
    Road,
    Water,
    Wall,
    Rubble,
    EnemySpawn,
    HostageCage,
    Extraction,
    PlayerSpawn,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub hp: u8,
}

impl Tile {
    pub fn new(kind: TileKind) -> Self {
        Self {
            hp: kind.max_hp(),
            kind,
        }
    }
}

impl TileKind {
    pub fn solid(self) -> bool {
        matches!(self, Self::Water | Self::Wall | Self::HostageCage)
    }

    pub fn destructible(self) -> bool {
        matches!(self, Self::Wall | Self::HostageCage)
    }

    pub fn max_hp(self) -> u8 {
        match self {
            Self::Wall => 3,
            Self::HostageCage => 2,
            _ => 0,
        }
    }

    pub fn destroyed_variant(self) -> Self {
        match self {
            Self::Wall | Self::HostageCage => Self::Rubble,
            _ => self,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelData {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileKind>,
}

#[derive(Clone, Debug)]
pub struct TileMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl TileMap {
    pub fn from_level_data(level: &LevelData) -> Self {
        Self {
            width: level.width,
            height: level.height,
            tiles: level.tiles.iter().copied().map(Tile::new).collect(),
        }
    }

    pub fn dimensions_px(&self) -> Vec2 {
        vec2(
            self.width as f32 * TILE_SIZE,
            self.height as f32 * TILE_SIZE,
        )
    }

    pub fn in_bounds(&self, tile: IVec2) -> bool {
        tile.x >= 0 && tile.y >= 0 && tile.x < self.width as i32 && tile.y < self.height as i32
    }

    pub fn index(&self, tile: IVec2) -> Option<usize> {
        if self.in_bounds(tile) {
            Some(tile.y as usize * self.width + tile.x as usize)
        } else {
            None
        }
    }

    pub fn tile(&self, tile: IVec2) -> Option<&Tile> {
        self.index(tile).map(|idx| &self.tiles[idx])
    }

    pub fn tile_mut(&mut self, tile: IVec2) -> Option<&mut Tile> {
        self.index(tile).map(move |idx| &mut self.tiles[idx])
    }

    pub fn tile_kind(&self, tile: IVec2) -> Option<TileKind> {
        self.tile(tile).map(|tile| tile.kind)
    }

    pub fn world_to_tile(&self, world: Vec2) -> Option<IVec2> {
        let tile = ivec2(
            (world.x / TILE_SIZE).floor() as i32,
            (world.y / TILE_SIZE).floor() as i32,
        );
        self.in_bounds(tile).then_some(tile)
    }

    pub fn tile_center(&self, tile: IVec2) -> Vec2 {
        vec2(
            tile.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            tile.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        )
    }

    pub fn collides_rect(&self, rect: Rect) -> bool {
        let min = ivec2(
            (rect.x / TILE_SIZE).floor() as i32,
            (rect.y / TILE_SIZE).floor() as i32,
        );
        let max = ivec2(
            ((rect.x + rect.w - 0.001) / TILE_SIZE).floor() as i32,
            ((rect.y + rect.h - 0.001) / TILE_SIZE).floor() as i32,
        );

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let tile = ivec2(x, y);
                if !self.in_bounds(tile) {
                    return true;
                }
                if self.tile_kind(tile).is_some_and(TileKind::solid) {
                    return true;
                }
            }
        }

        false
    }

    pub fn damage_at_world(&mut self, world: Vec2, damage: u8) -> bool {
        let Some(tile_pos) = self.world_to_tile(world) else {
            return false;
        };

        let Some(tile) = self.tile_mut(tile_pos) else {
            return false;
        };

        if tile.kind.destructible() {
            tile.hp = tile.hp.saturating_sub(damage);
            if tile.hp == 0 {
                tile.kind = tile.kind.destroyed_variant();
            }
            return true;
        }

        tile.kind.solid()
    }
}

pub fn default_level() -> LevelData {
    let width = 40;
    let height = 36;
    let mut tiles = vec![TileKind::Grass; width * height];

    let mut set_tile = |x: usize, y: usize, kind: TileKind| {
        tiles[y * width + x] = kind;
    };

    for y in 0..height {
        set_tile(19, y, TileKind::Road);
        set_tile(20, y, TileKind::Road);
        set_tile(21, y, TileKind::Road);
    }

    for y in 4..16 {
        for x in 2..8 {
            set_tile(x, y, TileKind::Water);
        }
    }

    for y in 18..30 {
        for x in 31..37 {
            set_tile(x, y, TileKind::Water);
        }
    }

    for y in 6..10 {
        for x in 13..17 {
            set_tile(x, y, TileKind::Wall);
        }
    }

    for y in 18..23 {
        for x in 24..28 {
            set_tile(x, y, TileKind::Wall);
        }
    }

    for y in 26..30 {
        for x in 10..14 {
            set_tile(x, y, TileKind::Wall);
        }
    }

    set_tile(20, 33, TileKind::PlayerSpawn);
    set_tile(20, 2, TileKind::Extraction);

    set_tile(16, 12, TileKind::HostageCage);
    set_tile(24, 16, TileKind::HostageCage);
    set_tile(12, 24, TileKind::HostageCage);

    set_tile(15, 8, TileKind::EnemySpawn);
    set_tile(25, 10, TileKind::EnemySpawn);
    set_tile(11, 20, TileKind::EnemySpawn);
    set_tile(29, 26, TileKind::EnemySpawn);

    LevelData {
        width,
        height,
        tiles,
    }
}

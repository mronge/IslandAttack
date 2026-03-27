use crate::entities::Direction;
use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Assets {
    atlas: Texture2D,
    directional_sprites: HashMap<DirectionalSpriteId, DirectionalSpriteSet>,
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub texture: Texture2D,
    pub source: Option<Rect>,
    pub draw_size: Vec2,
    pub anchor: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirectionalSpriteId {
    Jeep,
    Soldier,
}

#[derive(Clone)]
struct DirectionalSpriteSet {
    up: SpriteAsset,
    down: SpriteAsset,
    left: SpriteAsset,
    right: SpriteAsset,
}

impl Assets {
    pub async fn load() -> Self {
        let atlas = load_texture(crate::constants::MAP_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load map spritesheet: {}",
                    crate::constants::MAP_SPRITESHEET_PATH
                )
            });
        atlas.set_filter(FilterMode::Nearest);
        let jeep_sheet = load_texture(crate::constants::JEEP_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load jeep spritesheet: {}",
                    crate::constants::JEEP_SPRITESHEET_PATH
                )
            });
        jeep_sheet.set_filter(FilterMode::Nearest);
        let soldier_sheet = load_texture(crate::constants::SOLDIER_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load soldier spritesheet: {}",
                    crate::constants::SOLDIER_SPRITESHEET_PATH
                )
            });
        soldier_sheet.set_filter(FilterMode::Nearest);

        let mut directional_sprites = HashMap::new();
        register_directional_sheet_sprite_set(
            &mut directional_sprites,
            DirectionalSpriteId::Jeep,
            &jeep_sheet,
            vec2(64.0, 64.0),
            DirectionalFrameMap {
                up: 5,
                down: 1,
                left: 7,
                right: 3,
            },
            vec2(64.0, 64.0),
            vec2(32.0, 32.0),
        );
        register_directional_sheet_sprite_set(
            &mut directional_sprites,
            DirectionalSpriteId::Soldier,
            &soldier_sheet,
            vec2(32.0, 32.0),
            DirectionalFrameMap {
                up: 4,
                right: 7,
                down: 0,
                left: 1,
            },
            vec2(32.0, 32.0),
            vec2(16.0, 16.0),
        );

        Self {
            atlas,
            directional_sprites,
        }
    }

    pub fn atlas(&self) -> &Texture2D {
        &self.atlas
    }

    pub fn directional_sprite(&self, id: DirectionalSpriteId, dir: Direction) -> &SpriteAsset {
        let set = self
            .directional_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing directional sprite set: {id:?}"));
        match dir {
            Direction::Up => &set.up,
            Direction::Down => &set.down,
            Direction::Left => &set.left,
            Direction::Right => &set.right,
        }
    }
}

#[derive(Clone, Copy)]
struct DirectionalFrameMap {
    up: u32,
    down: u32,
    left: u32,
    right: u32,
}

fn sprite_from_sheet(
    texture: Texture2D,
    frame_size: Vec2,
    frame_index: u32,
    draw_size: Vec2,
    anchor: Vec2,
) -> SpriteAsset {
    let columns = (texture.width() / frame_size.x).floor().max(1.0) as u32;
    let frame_x = frame_index % columns;
    let frame_y = frame_index / columns;
    SpriteAsset {
        texture,
        source: Some(Rect::new(
            frame_x as f32 * frame_size.x,
            frame_y as f32 * frame_size.y,
            frame_size.x,
            frame_size.y,
        )),
        draw_size,
        anchor,
    }
}

fn register_directional_sheet_sprite_set(
    sprites: &mut HashMap<DirectionalSpriteId, DirectionalSpriteSet>,
    id: DirectionalSpriteId,
    texture: &Texture2D,
    frame_size: Vec2,
    frames: DirectionalFrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) {
    sprites.insert(
        id,
        DirectionalSpriteSet {
            up: sprite_from_sheet(texture.clone(), frame_size, frames.up, draw_size, anchor),
            down: sprite_from_sheet(texture.clone(), frame_size, frames.down, draw_size, anchor),
            left: sprite_from_sheet(texture.clone(), frame_size, frames.left, draw_size, anchor),
            right: sprite_from_sheet(texture.clone(), frame_size, frames.right, draw_size, anchor),
        },
    );
}

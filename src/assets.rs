use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Assets {
    atlas: Texture2D,
    sprites: HashMap<String, SpriteAsset>,
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub texture: Texture2D,
    pub source: Option<Rect>,
    pub draw_size: Vec2,
    pub anchor: Vec2,
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

        let mut sprites = HashMap::new();
        register_sheet_sprites(
            &mut sprites,
            &jeep_sheet,
            vec2(64.0, 64.0),
            &[
                ("jeep_up", 5, vec2(64.0, 64.0), vec2(32.0, 32.0)),
                ("jeep_down", 1, vec2(64.0, 64.0), vec2(32.0, 32.0)),
                ("jeep_left", 7, vec2(64.0, 64.0), vec2(32.0, 32.0)),
                ("jeep_right", 3, vec2(64.0, 64.0), vec2(32.0, 32.0)),
            ],
        );

        Self {
            atlas,
            sprites,
        }
    }

    pub fn atlas(&self) -> &Texture2D {
        &self.atlas
    }

    pub fn sprite(&self, name: &str) -> &SpriteAsset {
        self.sprites
            .get(name)
            .unwrap_or_else(|| panic!("missing sprite asset: {name}"))
    }
}

fn insert_sprite(
    sprites: &mut HashMap<String, SpriteAsset>,
    name: &str,
    texture: Texture2D,
    source: Option<Rect>,
    draw_size: Vec2,
    anchor: Vec2,
) {
    sprites.insert(
        name.to_owned(),
        SpriteAsset {
            texture,
            source,
            draw_size,
            anchor,
        },
    );
}

fn register_sheet_sprites(
    sprites: &mut HashMap<String, SpriteAsset>,
    texture: &Texture2D,
    frame_size: Vec2,
    entries: &[(&str, u32, Vec2, Vec2)],
) {
    let columns = (texture.width() / frame_size.x).floor().max(1.0) as u32;

    for (name, frame_index, draw_size, anchor) in entries {
        let frame_x = frame_index % columns;
        let frame_y = frame_index / columns;
        insert_sprite(
            sprites,
            name,
            texture.clone(),
            Some(Rect::new(
                frame_x as f32 * frame_size.x,
                frame_y as f32 * frame_size.y,
                frame_size.x,
                frame_size.y,
            )),
            *draw_size,
            *anchor,
        );
    }
}

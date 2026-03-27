use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const MANIFEST_PATH: &str = "output/imagegen/final/manifest.json";
const JEEP_SPRITESHEET_PATH: &str = "output/imagegen/raw/jeep.png";

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

#[derive(Deserialize)]
struct ManifestEntry {
    file: String,
    draw_width: f32,
    draw_height: f32,
    anchor_x: f32,
    anchor_y: f32,
}

impl Assets {
    pub async fn load() -> Self {
        let raw = fs::read_to_string(MANIFEST_PATH).expect("failed to read asset manifest");
        let manifest: HashMap<String, ManifestEntry> =
            serde_json::from_str(&raw).expect("failed to parse asset manifest");
        let atlas = load_texture(crate::constants::MAP_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load map spritesheet: {}",
                    crate::constants::MAP_SPRITESHEET_PATH
                )
            });
        atlas.set_filter(FilterMode::Nearest);
        let jeep_sheet = load_texture(JEEP_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| panic!("failed to load jeep spritesheet: {JEEP_SPRITESHEET_PATH}"));
        jeep_sheet.set_filter(FilterMode::Nearest);

        let mut sprites = HashMap::new();
        for (name, entry) in manifest {
            let texture = load_texture(&entry.file)
                .await
                .unwrap_or_else(|_| panic!("failed to load asset texture: {}", entry.file));
            texture.set_filter(FilterMode::Nearest);
            insert_sprite(
                &mut sprites,
                &name,
                texture,
                None,
                vec2(entry.draw_width, entry.draw_height),
                vec2(entry.anchor_x, entry.anchor_y),
            );
        }

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

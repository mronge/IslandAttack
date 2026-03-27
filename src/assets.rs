use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const MANIFEST_PATH: &str = "output/imagegen/final/manifest.json";
const JEEP_SPRITESHEET_PATH: &str = "output/imagegen/raw/jeep.png";

#[derive(Clone)]
pub struct Assets {
    atlas: Texture2D,
    jeep_sheet: Texture2D,
    sprites: HashMap<String, SpriteAsset>,
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub texture: Texture2D,
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
            sprites.insert(
                name,
                SpriteAsset {
                    texture,
                    draw_size: vec2(entry.draw_width, entry.draw_height),
                    anchor: vec2(entry.anchor_x, entry.anchor_y),
                },
            );
        }

        Self {
            atlas,
            jeep_sheet,
            sprites,
        }
    }

    pub fn atlas(&self) -> &Texture2D {
        &self.atlas
    }

    pub fn jeep_sheet(&self) -> &Texture2D {
        &self.jeep_sheet
    }

    pub fn sprite(&self, name: &str) -> &SpriteAsset {
        self.sprites
            .get(name)
            .unwrap_or_else(|| panic!("missing sprite asset: {name}"))
    }
}

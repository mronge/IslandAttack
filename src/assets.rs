use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const MANIFEST_PATH: &str = "output/imagegen/final/manifest.json";

#[derive(Clone)]
pub struct Assets {
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

        Self { sprites }
    }

    pub fn sprite(&self, name: &str) -> &SpriteAsset {
        self.sprites
            .get(name)
            .unwrap_or_else(|| panic!("missing sprite asset: {name}"))
    }
}

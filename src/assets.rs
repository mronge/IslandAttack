use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const SPRITE_SHEET_PATH: &str = "output/imagegen/final/nes_sprite_sheet.png";
const SPRITE_METADATA_PATH: &str = "output/imagegen/final/nes_sprite_sheet.json";

#[derive(Clone)]
pub struct Assets {
    sprite_sheet: Texture2D,
    sprite_regions: HashMap<String, Rect>,
}

#[derive(Deserialize)]
struct SpriteRegion {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Assets {
    pub async fn load() -> Self {
        let sprite_sheet = load_texture(SPRITE_SHEET_PATH)
            .await
            .expect("failed to load sprite sheet");
        sprite_sheet.set_filter(FilterMode::Nearest);

        let raw = fs::read_to_string(SPRITE_METADATA_PATH).expect("failed to read sprite metadata");
        let parsed: HashMap<String, SpriteRegion> =
            serde_json::from_str(&raw).expect("failed to parse sprite metadata");
        let sprite_regions = parsed
            .into_iter()
            .map(|(name, region)| (name, Rect::new(region.x, region.y, region.w, region.h)))
            .collect();

        Self {
            sprite_sheet,
            sprite_regions,
        }
    }

    pub fn texture(&self) -> &Texture2D {
        &self.sprite_sheet
    }

    pub fn region(&self, name: &str) -> Rect {
        self.sprite_regions
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("missing sprite region: {name}"))
    }
}

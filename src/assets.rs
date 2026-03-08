use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const SUBTILE_SHEET_PATH: &str = "output/imagegen/final/subtile_sheet.png";
const SUBTILE_DEFS_PATH: &str = "output/imagegen/final/subtile_defs.json";
const METASPRITES_PATH: &str = "output/imagegen/final/metasprites.json";

#[derive(Clone)]
pub struct Assets {
    subtile_sheet: Texture2D,
    subtile_regions: HashMap<String, Rect>,
    metasprites: HashMap<String, MetaSprite>,
}

#[derive(Deserialize)]
struct RegionDef {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Clone, Deserialize)]
pub struct MetaSprite {
    pub w: f32,
    pub h: f32,
    pub parts: Vec<MetaPart>,
}

#[derive(Clone, Deserialize)]
pub struct MetaPart {
    pub tile: String,
    pub x: f32,
    pub y: f32,
}

impl Assets {
    pub async fn load() -> Self {
        let subtile_sheet = load_texture(SUBTILE_SHEET_PATH)
            .await
            .expect("failed to load subtile sheet");
        subtile_sheet.set_filter(FilterMode::Nearest);

        let subtile_raw =
            fs::read_to_string(SUBTILE_DEFS_PATH).expect("failed to read subtile metadata");
        let parsed_regions: HashMap<String, RegionDef> =
            serde_json::from_str(&subtile_raw).expect("failed to parse subtile metadata");
        let subtile_regions = parsed_regions
            .into_iter()
            .map(|(name, region)| (name, Rect::new(region.x, region.y, region.w, region.h)))
            .collect();

        let metas_raw = fs::read_to_string(METASPRITES_PATH).expect("failed to read metasprites");
        let metasprites: HashMap<String, MetaSprite> =
            serde_json::from_str(&metas_raw).expect("failed to parse metasprites");

        Self {
            subtile_sheet,
            subtile_regions,
            metasprites,
        }
    }

    pub fn texture(&self) -> &Texture2D {
        &self.subtile_sheet
    }

    pub fn subtile_region(&self, name: &str) -> Rect {
        self.subtile_regions
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("missing subtile region: {name}"))
    }

    pub fn metasprite(&self, name: &str) -> &MetaSprite {
        self.metasprites
            .get(name)
            .unwrap_or_else(|| panic!("missing metasprite: {name}"))
    }
}

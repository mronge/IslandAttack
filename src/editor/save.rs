use crate::world::LevelData;
use std::fs;
use std::path::Path;

pub fn save_level(path: &str, level: &LevelData) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(level).map_err(|err| err.to_string())?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, serialized).map_err(|err| err.to_string())
}

pub fn load_level(path: &str) -> Option<LevelData> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

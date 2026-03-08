use crate::constants::TILE_SIZE;
use crate::render::camera::screen_to_world;
use crate::world::{LevelData, TileKind, World};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct EditorAction {
    pub rebuild_world: bool,
    pub save: bool,
    pub load: bool,
    pub playtest: bool,
}

pub struct EditorState {
    pub brush: TileKind,
    pub camera_center: Vec2,
}

impl EditorState {
    pub fn new(world: &World) -> Self {
        Self {
            brush: TileKind::Wall,
            camera_center: world.player.pos,
        }
    }

    pub fn update(&mut self, dt: f32, level: &mut LevelData, world: &World) -> EditorAction {
        self.handle_brush_hotkeys();

        let camera_speed = 140.0;
        if is_key_down(KeyCode::Left) {
            self.camera_center.x -= camera_speed * dt;
        }
        if is_key_down(KeyCode::Right) {
            self.camera_center.x += camera_speed * dt;
        }
        if is_key_down(KeyCode::Up) {
            self.camera_center.y -= camera_speed * dt;
        }
        if is_key_down(KeyCode::Down) {
            self.camera_center.y += camera_speed * dt;
        }

        let mut action = EditorAction::default();

        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            if let Some(world_pos) = screen_to_world(
                vec2(mouse_position().0, mouse_position().1),
                self.camera_center,
            ) {
                let tile = ivec2(
                    (world_pos.x / TILE_SIZE).floor() as i32,
                    (world_pos.y / TILE_SIZE).floor() as i32,
                );

                if tile.x >= 0
                    && tile.y >= 0
                    && tile.x < level.width as i32
                    && tile.y < level.height as i32
                {
                    let index = tile.y as usize * level.width + tile.x as usize;
                    let kind = if is_mouse_button_down(MouseButton::Right) {
                        TileKind::Grass
                    } else {
                        self.brush
                    };

                    if kind == TileKind::PlayerSpawn {
                        for current in &mut level.tiles {
                            if *current == TileKind::PlayerSpawn {
                                *current = TileKind::Grass;
                            }
                        }
                    }

                    if level.tiles[index] != kind {
                        level.tiles[index] = kind;
                        action.rebuild_world = true;
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::F5) {
            action.save = true;
        }
        if is_key_pressed(KeyCode::F9) {
            action.load = true;
        }
        if is_key_pressed(KeyCode::Enter) {
            action.playtest = true;
        }

        let map_size = world.map.dimensions_px();
        self.camera_center.x = self.camera_center.x.clamp(0.0, map_size.x.max(1.0));
        self.camera_center.y = self.camera_center.y.clamp(0.0, map_size.y.max(1.0));

        action
    }

    fn handle_brush_hotkeys(&mut self) {
        if is_key_pressed(KeyCode::Key1) {
            self.brush = TileKind::Grass;
        }
        if is_key_pressed(KeyCode::Key2) {
            self.brush = TileKind::Road;
        }
        if is_key_pressed(KeyCode::Key3) {
            self.brush = TileKind::Water;
        }
        if is_key_pressed(KeyCode::Key4) {
            self.brush = TileKind::Wall;
        }
        if is_key_pressed(KeyCode::Key5) {
            self.brush = TileKind::EnemySpawn;
        }
        if is_key_pressed(KeyCode::Key6) {
            self.brush = TileKind::HostageCage;
        }
        if is_key_pressed(KeyCode::Key7) {
            self.brush = TileKind::Extraction;
        }
        if is_key_pressed(KeyCode::Key8) {
            self.brush = TileKind::PlayerSpawn;
        }
    }
}

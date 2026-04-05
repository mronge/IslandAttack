use crate::entities::{ActorAnimState, Facing4, Facing8};
use crate::world::MissionResult;
use macroquad::audio::{Sound, load_sound};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Assets {
    atlas: Texture2D,
    facing4_sprites: HashMap<Facing4SpriteId, Facing4SpriteSet>,
    animated_facing4_sprites: HashMap<Facing4SpriteId, AnimatedFacing4SpriteSet>,
    facing8_sprites: HashMap<Facing8SpriteId, Facing8SpriteSet>,
    static_sprites: HashMap<StaticSpriteId, Vec<SpriteAsset>>,
}

#[derive(Clone)]
pub struct SpriteAsset {
    pub texture: Texture2D,
    pub source: Option<Rect>,
    pub draw_size: Vec2,
    pub anchor: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing4SpriteId {
    Jeep,
    Soldier,
    Pow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing8SpriteId {
    Turret,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StaticSpriteId {
    Barracks,
    BarracksDestroyed,
    TurretDestroyed,
}

#[derive(Clone)]
struct Facing4SpriteSet {
    up: SpriteAsset,
    down: SpriteAsset,
    left: SpriteAsset,
    right: SpriteAsset,
}

#[derive(Clone)]
struct Facing8SpriteSet {
    north: SpriteAsset,
    north_east: SpriteAsset,
    east: SpriteAsset,
    south_east: SpriteAsset,
    south: SpriteAsset,
    south_west: SpriteAsset,
    west: SpriteAsset,
    north_west: SpriteAsset,
}

#[derive(Clone)]
struct AnimatedSpriteClip {
    idle: SpriteAsset,
    walk: Vec<SpriteAsset>,
    shoot: Option<SpriteAsset>,
}

#[derive(Clone)]
struct AnimatedFacing4SpriteSet {
    up: AnimatedSpriteClip,
    down: AnimatedSpriteClip,
    left: AnimatedSpriteClip,
    right: AnimatedSpriteClip,
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
        let turret_sheet = load_texture(crate::constants::TURRET_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load turret spritesheet: {}",
                    crate::constants::TURRET_SPRITESHEET_PATH
                )
            });
        turret_sheet.set_filter(FilterMode::Nearest);
        let pow_sheet = load_texture(crate::constants::POW_SPRITESHEET_PATH)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "failed to load pow spritesheet: {}",
                    crate::constants::POW_SPRITESHEET_PATH
                )
            });
        pow_sheet.set_filter(FilterMode::Nearest);

        let mut facing4_sprites = HashMap::new();
        let mut animated_facing4_sprites = HashMap::new();
        let mut facing8_sprites = HashMap::new();
        let mut static_sprites = HashMap::new();
        register_facing4_sheet_sprite_set(
            &mut facing4_sprites,
            Facing4SpriteId::Jeep,
            &jeep_sheet,
            vec2(64.0, 64.0),
            Facing4FrameMap {
                up: 5,
                down: 1,
                left: 7,
                right: 3,
            },
            vec2(64.0, 64.0),
            vec2(32.0, 32.0),
        );
        register_animated_facing4_sheet_sprite_set(
            &mut animated_facing4_sprites,
            Facing4SpriteId::Soldier,
            &soldier_sheet,
            vec2(32.0, 32.0),
            Facing4AnimationFrameMap {
                up: AnimationFrameMap {
                    idle: 12,
                    shoot: Some(13),
                    walk: &[14, 15],
                },
                down: AnimationFrameMap {
                    idle: 0,
                    shoot: Some(1),
                    walk: &[2, 3],
                },
                left: AnimationFrameMap {
                    idle: 4,
                    shoot: Some(5),
                    walk: &[6, 7],
                },
                right: AnimationFrameMap {
                    idle: 8,
                    shoot: Some(9),
                    walk: &[10, 11],
                },
            },
            vec2(32.0, 32.0),
            vec2(16.0, 16.0),
        );
        register_animated_facing4_sheet_sprite_set(
            &mut animated_facing4_sprites,
            Facing4SpriteId::Pow,
            &pow_sheet,
            vec2(32.0, 32.0),
            Facing4AnimationFrameMap {
                up: AnimationFrameMap {
                    idle: 9,
                    shoot: None,
                    walk: &[10, 11],
                },
                down: AnimationFrameMap {
                    idle: 0,
                    shoot: None,
                    walk: &[1, 2],
                },
                left: AnimationFrameMap {
                    idle: 3,
                    shoot: None,
                    walk: &[4, 5],
                },
                right: AnimationFrameMap {
                    idle: 6,
                    shoot: None,
                    walk: &[7, 8],
                },
            },
            vec2(32.0, 32.0),
            vec2(16.0, 16.0),
        );
        register_facing8_sheet_sprite_set(
            &mut facing8_sprites,
            Facing8SpriteId::Turret,
            &turret_sheet,
            vec2(32.0, 32.0),
            Facing8FrameMap {
                south: 0,
                south_east: 1,
                east: 2,
                north_east: 3,
                north: 4,
                north_west: 5,
                west: 6,
                south_west: 7,
            },
            vec2(32.0, 32.0),
            vec2(16.0, 16.0),
        );
        static_sprites.insert(
            StaticSpriteId::TurretDestroyed,
            load_static_sprite_variants(
                crate::constants::TURRET_DESTROYED_SPRITE_PATH,
                vec2(32.0, 32.0),
                vec2(16.0, 16.0),
            )
            .await,
        );
        static_sprites.insert(
            StaticSpriteId::Barracks,
            load_static_sprite_variants(
                crate::constants::BARRACKS_SPRITE_PATH,
                vec2(64.0, 64.0),
                vec2(32.0, 32.0),
            )
            .await,
        );
        static_sprites.insert(
            StaticSpriteId::BarracksDestroyed,
            load_static_sprite_variants(
                crate::constants::BARRACKS_DESTROYED_SPRITE_PATH,
                vec2(64.0, 64.0),
                vec2(32.0, 32.0),
            )
            .await,
        );

        Self {
            atlas,
            facing4_sprites,
            animated_facing4_sprites,
            facing8_sprites,
            static_sprites,
        }
    }

    pub fn atlas(&self) -> &Texture2D {
        &self.atlas
    }

    pub fn facing4_sprite(&self, id: Facing4SpriteId, facing: Facing4) -> &SpriteAsset {
        let set = self
            .facing4_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing facing4 sprite set: {id:?}"));
        match facing {
            Facing4::North => &set.up,
            Facing4::South => &set.down,
            Facing4::West => &set.left,
            Facing4::East => &set.right,
        }
    }

    pub fn animated_facing4_sprite(
        &self,
        id: Facing4SpriteId,
        facing: Facing4,
        state: ActorAnimState,
        frame_index: usize,
    ) -> &SpriteAsset {
        let set = self
            .animated_facing4_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing animated facing4 sprite set: {id:?}"));
        let clip = match facing {
            Facing4::North => &set.up,
            Facing4::South => &set.down,
            Facing4::West => &set.left,
            Facing4::East => &set.right,
        };

        match state {
            ActorAnimState::Idle => &clip.idle,
            // Some actors, like the POW, have no dedicated shoot frame. Keep
            // a single 4-way animation path by falling back to idle.
            ActorAnimState::Shoot => clip.shoot.as_ref().unwrap_or(&clip.idle),
            ActorAnimState::Walk => &clip.walk[frame_index % clip.walk.len()],
        }
    }

    pub fn facing8_sprite(&self, id: Facing8SpriteId, facing: Facing8) -> &SpriteAsset {
        let set = self
            .facing8_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing facing8 sprite set: {id:?}"));
        match facing {
            Facing8::North => &set.north,
            Facing8::NorthEast => &set.north_east,
            Facing8::East => &set.east,
            Facing8::SouthEast => &set.south_east,
            Facing8::South => &set.south,
            Facing8::SouthWest => &set.south_west,
            Facing8::West => &set.west,
            Facing8::NorthWest => &set.north_west,
        }
    }

    pub fn static_sprite(&self, id: StaticSpriteId, variant_seed: u64) -> &SpriteAsset {
        let variants = self
            .static_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing static sprite: {id:?}"));
        &variants[(variant_seed as usize) % variants.len()]
    }
}

pub async fn load_splash_screen() -> Texture2D {
    let splash_screen = load_texture(crate::constants::SPLASH_SCREEN_PATH)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "failed to load splash screen: {}",
                crate::constants::SPLASH_SCREEN_PATH
            )
        });
    splash_screen.set_filter(FilterMode::Nearest);
    splash_screen
}

pub async fn load_theme_music() -> Sound {
    load_sound(crate::constants::THEME_MUSIC_PATH)
        .await
        .unwrap_or_else(|_| {
            panic!(
                "failed to load theme music: {}",
                crate::constants::THEME_MUSIC_PATH
            )
        })
}

pub async fn load_result_sound(result: MissionResult) -> Sound {
    let path = match result {
        MissionResult::Success => crate::constants::SUCCESS_MUSIC_PATH,
        MissionResult::Failure => crate::constants::FAILURE_MUSIC_PATH,
    };

    load_sound(path)
        .await
        .unwrap_or_else(|_| panic!("failed to load result music: {path}"))
}

#[derive(Clone, Copy)]
struct Facing4FrameMap {
    up: u32,
    down: u32,
    left: u32,
    right: u32,
}

#[derive(Clone, Copy)]
struct AnimationFrameMap {
    idle: u32,
    walk: &'static [u32],
    shoot: Option<u32>,
}

#[derive(Clone, Copy)]
struct Facing4AnimationFrameMap {
    up: AnimationFrameMap,
    down: AnimationFrameMap,
    left: AnimationFrameMap,
    right: AnimationFrameMap,
}

#[derive(Clone, Copy)]
struct Facing8FrameMap {
    north: u32,
    north_east: u32,
    east: u32,
    south_east: u32,
    south: u32,
    south_west: u32,
    west: u32,
    north_west: u32,
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

fn register_facing4_sheet_sprite_set(
    sprites: &mut HashMap<Facing4SpriteId, Facing4SpriteSet>,
    id: Facing4SpriteId,
    texture: &Texture2D,
    frame_size: Vec2,
    frames: Facing4FrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) {
    sprites.insert(
        id,
        Facing4SpriteSet {
            up: sprite_from_sheet(texture.clone(), frame_size, frames.up, draw_size, anchor),
            down: sprite_from_sheet(texture.clone(), frame_size, frames.down, draw_size, anchor),
            left: sprite_from_sheet(texture.clone(), frame_size, frames.left, draw_size, anchor),
            right: sprite_from_sheet(texture.clone(), frame_size, frames.right, draw_size, anchor),
        },
    );
}

fn animated_clip_from_sheet(
    texture: Texture2D,
    frame_size: Vec2,
    frames: AnimationFrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) -> AnimatedSpriteClip {
    AnimatedSpriteClip {
        idle: sprite_from_sheet(texture.clone(), frame_size, frames.idle, draw_size, anchor),
        walk: frames
            .walk
            .iter()
            .map(|frame| sprite_from_sheet(texture.clone(), frame_size, *frame, draw_size, anchor))
            .collect(),
        shoot: frames
            .shoot
            .map(|frame| sprite_from_sheet(texture, frame_size, frame, draw_size, anchor)),
    }
}

fn register_facing8_sheet_sprite_set(
    sprites: &mut HashMap<Facing8SpriteId, Facing8SpriteSet>,
    id: Facing8SpriteId,
    texture: &Texture2D,
    frame_size: Vec2,
    frames: Facing8FrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) {
    // Keep 8-way frame order centralized here so additional turret variants can
    // register their sheets without duplicating frame indexing logic in render code.
    sprites.insert(
        id,
        Facing8SpriteSet {
            north: sprite_from_sheet(texture.clone(), frame_size, frames.north, draw_size, anchor),
            north_east: sprite_from_sheet(
                texture.clone(),
                frame_size,
                frames.north_east,
                draw_size,
                anchor,
            ),
            east: sprite_from_sheet(texture.clone(), frame_size, frames.east, draw_size, anchor),
            south_east: sprite_from_sheet(
                texture.clone(),
                frame_size,
                frames.south_east,
                draw_size,
                anchor,
            ),
            south: sprite_from_sheet(texture.clone(), frame_size, frames.south, draw_size, anchor),
            south_west: sprite_from_sheet(
                texture.clone(),
                frame_size,
                frames.south_west,
                draw_size,
                anchor,
            ),
            west: sprite_from_sheet(texture.clone(), frame_size, frames.west, draw_size, anchor),
            north_west: sprite_from_sheet(
                texture.clone(),
                frame_size,
                frames.north_west,
                draw_size,
                anchor,
            ),
        },
    );
}

fn register_animated_facing4_sheet_sprite_set(
    sprites: &mut HashMap<Facing4SpriteId, AnimatedFacing4SpriteSet>,
    id: Facing4SpriteId,
    texture: &Texture2D,
    frame_size: Vec2,
    frames: Facing4AnimationFrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) {
    sprites.insert(
        id,
        AnimatedFacing4SpriteSet {
            up: animated_clip_from_sheet(texture.clone(), frame_size, frames.up, draw_size, anchor),
            down: animated_clip_from_sheet(
                texture.clone(),
                frame_size,
                frames.down,
                draw_size,
                anchor,
            ),
            left: animated_clip_from_sheet(
                texture.clone(),
                frame_size,
                frames.left,
                draw_size,
                anchor,
            ),
            right: animated_clip_from_sheet(
                texture.clone(),
                frame_size,
                frames.right,
                draw_size,
                anchor,
            ),
        },
    );
}

async fn load_static_sprite_variants(
    path: &str,
    draw_size: Vec2,
    anchor: Vec2,
) -> Vec<SpriteAsset> {
    let mut variants = Vec::new();
    variants.push(load_static_sprite(path, draw_size, anchor).await);

    for variant_path in numbered_variant_paths(path) {
        if !variant_path.exists() {
            break;
        }

        variants.push(
            load_static_sprite(
                variant_path.to_str().expect("invalid static sprite path"),
                draw_size,
                anchor,
            )
            .await,
        );
    }

    variants
}

async fn load_static_sprite(path: &str, draw_size: Vec2, anchor: Vec2) -> SpriteAsset {
    let texture = load_texture(path)
        .await
        .unwrap_or_else(|_| panic!("failed to load static sprite: {path}"));
    texture.set_filter(FilterMode::Nearest);
    SpriteAsset {
        texture,
        source: None,
        draw_size,
        anchor,
    }
}

fn numbered_variant_paths(path: &str) -> impl Iterator<Item = PathBuf> + '_ {
    let base = Path::new(path);
    let stem = base
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_else(|| panic!("invalid static sprite filename: {path}"))
        .to_owned();
    let ext = base
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_else(|| panic!("missing static sprite extension: {path}"))
        .to_owned();
    let parent = base.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

    (1..).map(move |index| parent.join(format!("{stem}{index}.{ext}")))
}

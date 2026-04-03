use crate::entities::{Direction, EnemyAnimState};
use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Assets {
    atlas: Texture2D,
    directional_sprites: HashMap<DirectionalSpriteId, DirectionalSpriteSet>,
    animated_directional_sprites: HashMap<DirectionalSpriteId, DirectionalAnimatedSpriteSet>,
    turret_sprites: Vec<SpriteAsset>,
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

#[derive(Clone)]
struct AnimatedSpriteClip {
    idle: SpriteAsset,
    walk: Vec<SpriteAsset>,
    shoot: SpriteAsset,
}

#[derive(Clone)]
struct DirectionalAnimatedSpriteSet {
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

        let mut directional_sprites = HashMap::new();
        let mut animated_directional_sprites = HashMap::new();
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
        register_directional_animated_sheet_sprite_set(
            &mut animated_directional_sprites,
            DirectionalSpriteId::Soldier,
            &soldier_sheet,
            vec2(32.0, 32.0),
            DirectionalAnimationFrameMap {
                up: AnimationFrameMap {
                    idle: 12,
                    shoot: 13,
                    walk: &[14, 15],
                },
                down: AnimationFrameMap {
                    idle: 0,
                    shoot: 1,
                    walk: &[2, 3],
                },
                left: AnimationFrameMap {
                    idle: 4,
                    shoot: 5,
                    walk: &[6, 7],
                },
                right: AnimationFrameMap {
                    idle: 8,
                    shoot: 9,
                    walk: &[10, 11],
                },
            },
            vec2(32.0, 32.0),
            vec2(16.0, 16.0),
        );
        // The turret sheet is a single row of 8 pre-rotated frames, so keep
        // them as a flat array and let render code choose the correct one from
        // the current aim vector.
        let turret_sprites = (0..8)
            .map(|frame| {
                sprite_from_sheet(
                    turret_sheet.clone(),
                    vec2(32.0, 32.0),
                    frame,
                    vec2(32.0, 32.0),
                    vec2(16.0, 16.0),
                )
            })
            .collect();

        Self {
            atlas,
            directional_sprites,
            animated_directional_sprites,
            turret_sprites,
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

    pub fn animated_directional_sprite(
        &self,
        id: DirectionalSpriteId,
        dir: Direction,
        state: EnemyAnimState,
        frame_index: usize,
    ) -> &SpriteAsset {
        let set = self
            .animated_directional_sprites
            .get(&id)
            .unwrap_or_else(|| panic!("missing animated directional sprite set: {id:?}"));
        let clip = match dir {
            Direction::Up => &set.up,
            Direction::Down => &set.down,
            Direction::Left => &set.left,
            Direction::Right => &set.right,
        };

        match state {
            EnemyAnimState::Idle => &clip.idle,
            EnemyAnimState::Shoot => &clip.shoot,
            EnemyAnimState::Walk => &clip.walk[frame_index % clip.walk.len()],
        }
    }

    pub fn turret_sprite(&self, frame_index: usize) -> &SpriteAsset {
        // Frame selection is normalized by the caller, but wrap anyway so the
        // asset accessor stays safe if the sheet order changes later.
        &self.turret_sprites[frame_index % self.turret_sprites.len()]
    }
}

#[derive(Clone, Copy)]
struct DirectionalFrameMap {
    up: u32,
    down: u32,
    left: u32,
    right: u32,
}

#[derive(Clone, Copy)]
struct AnimationFrameMap {
    idle: u32,
    walk: &'static [u32],
    shoot: u32,
}

#[derive(Clone, Copy)]
struct DirectionalAnimationFrameMap {
    up: AnimationFrameMap,
    down: AnimationFrameMap,
    left: AnimationFrameMap,
    right: AnimationFrameMap,
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
        shoot: sprite_from_sheet(texture, frame_size, frames.shoot, draw_size, anchor),
    }
}

fn register_directional_animated_sheet_sprite_set(
    sprites: &mut HashMap<DirectionalSpriteId, DirectionalAnimatedSpriteSet>,
    id: DirectionalSpriteId,
    texture: &Texture2D,
    frame_size: Vec2,
    frames: DirectionalAnimationFrameMap,
    draw_size: Vec2,
    anchor: Vec2,
) {
    sprites.insert(
        id,
        DirectionalAnimatedSpriteSet {
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

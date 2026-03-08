use crate::constants::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::render::sprites::viewport_scale;
use crate::world::World;
use macroquad::prelude::*;

pub fn initial_play_camera_center(world: &World) -> Vec2 {
    clamp_camera_center(world.player.pos, world)
}

pub fn update_play_camera_center(current_center: Vec2, world: &World) -> Vec2 {
    let half = vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5);
    let margin_x = VIEW_WIDTH / 3.0;
    let margin_y = VIEW_HEIGHT / 3.0;
    let mut center = clamp_camera_center(current_center, world);
    let mut top_left = center - half;
    let player = world.player.pos;

    let left_limit = top_left.x + margin_x;
    if player.x < left_limit {
        top_left.x = player.x - margin_x;
    }

    let right_limit = top_left.x + VIEW_WIDTH - margin_x;
    if player.x > right_limit {
        top_left.x = player.x - (VIEW_WIDTH - margin_x);
    }

    let top_limit = top_left.y + margin_y;
    if player.y < top_limit {
        top_left.y = player.y - margin_y;
    }

    let bottom_limit = top_left.y + VIEW_HEIGHT - margin_y;
    if player.y > bottom_limit {
        top_left.y = player.y - (VIEW_HEIGHT - margin_y);
    }

    center = top_left + half;
    clamp_camera_center(center, world)
}

pub fn clamp_camera_center(center: Vec2, world: &World) -> Vec2 {
    let size = world.map.dimensions_px();
    let half = vec2(VIEW_WIDTH * 0.5, VIEW_HEIGHT * 0.5);

    let x = if size.x <= VIEW_WIDTH {
        size.x * 0.5
    } else {
        center.x.clamp(half.x, size.x - half.x)
    };

    let y = if size.y <= VIEW_HEIGHT {
        size.y * 0.5
    } else {
        center.y.clamp(half.y, size.y - half.y)
    };

    vec2(x, y)
}

pub fn screen_to_world(screen: Vec2, camera_center: Vec2) -> Option<Vec2> {
    let (scale, origin) = viewport_scale();
    if scale <= 0.0 {
        return None;
    }

    let local = screen - origin;
    if local.x < 0.0
        || local.y < 0.0
        || local.x > VIEW_WIDTH * scale
        || local.y > VIEW_HEIGHT * scale
    {
        return None;
    }

    let internal = local / scale;
    Some(camera_center - vec2(VIEW_WIDTH, VIEW_HEIGHT) * 0.5 + internal)
}

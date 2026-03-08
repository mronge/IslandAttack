use macroquad::prelude::*;

pub fn rect_from_center(center: Vec2, size: Vec2) -> Rect {
    Rect::new(
        center.x - size.x * 0.5,
        center.y - size.y * 0.5,
        size.x,
        size.y,
    )
}

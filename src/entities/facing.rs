use macroquad::prelude::*;

use crate::entities::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing4 {
    North,
    South,
    West,
    East,
}

impl Facing4 {
    pub fn from_direction(dir: Direction) -> Self {
        match dir {
            Direction::Up => Self::North,
            Direction::Down => Self::South,
            Direction::Left => Self::West,
            Direction::Right => Self::East,
        }
    }

    pub fn from_vec(delta: Vec2) -> Self {
        Self::from_direction(Direction::from_vec(delta))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing8 {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Facing8 {
    pub fn from_vec(delta: Vec2) -> Self {
        if delta.length_squared() <= f32::EPSILON {
            return Self::South;
        }

        let angle = delta.y.atan2(delta.x);
        let octant = ((angle / std::f32::consts::FRAC_PI_4).round() as i32).rem_euclid(8);

        match octant {
            0 => Self::East,
            1 => Self::SouthEast,
            2 => Self::South,
            3 => Self::SouthWest,
            4 => Self::West,
            5 => Self::NorthWest,
            6 => Self::North,
            7 => Self::NorthEast,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Facing4, Facing8};
    use crate::entities::Direction;
    use macroquad::prelude::vec2;

    #[test]
    fn maps_four_way_direction_and_vectors_to_facing4() {
        assert_eq!(Facing4::from_direction(Direction::Up), Facing4::North);
        assert_eq!(Facing4::from_direction(Direction::Down), Facing4::South);
        assert_eq!(Facing4::from_direction(Direction::Left), Facing4::West);
        assert_eq!(Facing4::from_direction(Direction::Right), Facing4::East);

        assert_eq!(Facing4::from_vec(vec2(0.0, -1.0)), Facing4::North);
        assert_eq!(Facing4::from_vec(vec2(0.0, 1.0)), Facing4::South);
        assert_eq!(Facing4::from_vec(vec2(-1.0, 0.0)), Facing4::West);
        assert_eq!(Facing4::from_vec(vec2(1.0, 0.0)), Facing4::East);
    }

    #[test]
    fn maps_cardinal_and_diagonal_vectors_to_eight_way_facing() {
        assert_eq!(Facing8::from_vec(vec2(1.0, 0.0)), Facing8::East);
        assert_eq!(Facing8::from_vec(vec2(1.0, -1.0)), Facing8::NorthEast);
        assert_eq!(Facing8::from_vec(vec2(0.0, -1.0)), Facing8::North);
        assert_eq!(Facing8::from_vec(vec2(-1.0, -1.0)), Facing8::NorthWest);
        assert_eq!(Facing8::from_vec(vec2(-1.0, 0.0)), Facing8::West);
        assert_eq!(Facing8::from_vec(vec2(-1.0, 1.0)), Facing8::SouthWest);
        assert_eq!(Facing8::from_vec(vec2(0.0, 1.0)), Facing8::South);
        assert_eq!(Facing8::from_vec(vec2(1.0, 1.0)), Facing8::SouthEast);
    }
}

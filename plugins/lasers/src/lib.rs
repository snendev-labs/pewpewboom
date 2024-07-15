use bevy::{prelude::*, utils::petgraph::graph::Edge};
use hexx::*;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct Laser;

impl Laser {
    pub const POWER: f32 = 1.;
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum Direction {
    North,
    South,
    Northeast,
    Southeast,
    Northwest,
    Southwest,
}

impl Direction {
    fn to_hexx(&self) -> EdgeDirection {
        match self {
            Self::North => EdgeDirection::FLAT_NORTH,
            Self::South => EdgeDirection::FLAT_SOUTH,
            Self::Northeast => EdgeDirection::FLAT_NORTH_EAST,
            Self::Southeast => EdgeDirection::FLAT_SOUTH_EAST,
            Self::Northwest => EdgeDirection::FLAT_NORTH_WEST,
            Self::Southwest => EdgeDirection::FLAT_SOUTH_WEST,
        }
    }
}

#[derive(Debug)]
#[derive(Bundle)]
pub struct LaserBundle {
    laser: Laser,
    direction: Direction,
}

#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

fn first_hit(
    laser_query: Query<(&Position, &Direction), With<Laser>>,
    tile_query: Query<&Position>,
) {
    for (laser_position, laser_direction) in &laser_query {
        let mut steps = 0;
        let mut position = laser_position;
        while steps < 100 {
            position = position.neighbor(laser_direction.to_hexx());
            for occupied_position in &tile_query {
                if position == occupied_position {}
            }
        }
    }
}

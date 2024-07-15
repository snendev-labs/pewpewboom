use bevy::prelude::*;
use hexx::*;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

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
    fn to_hex(&self) -> EdgeDirection {
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

impl Position {
    pub fn ray(&self, direction: Direction, limit: u32) -> Vec {
        let ray = Vec::new();
        let mut steps = 1;
        let mut next_neighbor = self;
        while steps < limit {
            next_neighbor = next_neighbor.neighbor(direction.to_hex());
            ray.append(next_neighbor)
        }
        ray
    }
}

pub neighbor_along_ray()

pub trait Tile {
    const POSITION: Position;
}

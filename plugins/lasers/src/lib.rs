use bevy::prelude::*;
use hexx::*;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {}
}

impl LaserPlugin {
    fn track_lasers(
        lasers: Query<(&Position, &Direction), With<Laser>>,
        colliders: Query<(
            &Position,
            Option<&Refraction>,
            Option<&Amplification>,
            Option<&Consumption>,
        )>,
    ) {
        for (laser_position, laser_direction) in &lasers {
            let mut hits = Vec::new();
            let mut current_source = *laser_position;
            'outer: loop {
                hits.push(current_source);
                let mut steps = 0;
                let next_neighbor: Position =
                    current_source.neighbor(laser_direction.to_hex()).into();
                while steps < 100 {
                    for (position, option_refraction, option_amplification, option_consumption) in
                        &colliders
                    {
                        if *position == next_neighbor {
                            current_source = *position;
                            break 'outer;
                        }
                    }
                    steps += 1
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct LaserSystems;

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

impl From<Hex> for Position {
    fn from(value: Hex) -> Self {
        Self(value)
    }
}

impl Position {
    pub fn ray(self, direction: &Direction, limit: u32) -> Vec<Position> {
        let mut ray = Vec::new();
        let mut steps = 1;
        let mut next_neighbor = self;
        while steps < limit {
            next_neighbor = next_neighbor.neighbor(direction.to_hex()).into();
            ray.push(next_neighbor);
            steps += 1;
        }
        ray
    }
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

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct Laser;

impl Laser {
    pub const POWER: f32 = 1.;
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]

pub struct Refraction;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Amplification(usize);

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect, Deref, DerefMut)]

pub struct Consumption(Entity);

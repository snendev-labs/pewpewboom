use std::fmt::Error;

use bevy::prelude::*;
use hexx::*;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaserPathEvent>()
            .add_event::<LaserHitEvent>()
            .add_systems(Update, Self::track_lasers.in_set(LaserSystems));
    }
}

impl LaserPlugin {
    #[allow(clippy::type_complexity)]
    fn track_lasers(
        lasers: Query<(&Position, &Direction, &Shooter), With<Laser>>,
        colliders: Query<
            (
                Entity,
                &Position,
                Option<&Refraction>,
                Option<&Reflection>,
                Option<&Amplification>,
                Option<&Consumption>,
            ),
            Or<(
                With<Refraction>,
                With<Reflection>,
                With<Amplification>,
                With<Consumption>,
            )>,
        >,
        mut laser_hit_events: EventWriter<LaserHitEvent>,
        mut laser_path_events: EventWriter<LaserPathEvent>,
    ) {
        'lasers: for (laser_position, laser_direction, laser_shooter) in &lasers {
            const LASER_RANGE: usize = 100;
            const BASE_LASER_STRENGTH: usize = 1;

            let mut path = Vec::new();
            let mut current_position = *laser_position;
            let mut current_direction = *laser_direction;
            let mut strength = BASE_LASER_STRENGTH;

            for _ in 0..LASER_RANGE {
                let next_position: Position =
                    current_position.neighbor(current_direction.as_hex()).into();
                path.push(next_position);

                if let Some((collider, _, refraction, reflection, amplification, consumption)) =
                    colliders
                        .iter()
                        .find(|(_, position, _, _, _, _)| **position == next_position)
                {
                    if let Some(refracted_direction) =
                        refraction.and_then(|refraction| refraction.refract(current_direction))
                    {
                        current_direction = refracted_direction;
                    }
                    if let Some(reflected_direction) =
                        reflection.and_then(|reflection| reflection.reflect(current_direction))
                    {
                        current_direction = reflected_direction;
                    }
                    if let Some(amplification) = amplification {
                        strength += **amplification;
                    }
                    if consumption.is_some()
                        && consumption
                            .unwrap()
                            .vulnerable
                            .iter()
                            .any(|&direction| direction == current_direction)
                    {
                        laser_hit_events.send(LaserHitEvent {
                            consumer: collider,
                            strength,
                            shooter: **laser_shooter,
                        });
                        laser_path_events.send(LaserPathEvent { path });
                        break 'lasers;
                    }
                }
                current_position = next_position;
            }
            laser_path_events.send(LaserPathEvent { path });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct LaserSystems;

#[derive(Event)]
pub struct LaserHitEvent {
    pub strength: usize,
    pub consumer: Entity,
    pub shooter: Entity,
}

#[derive(Event)]
pub struct LaserPathEvent {
    pub path: Vec<Position>,
}

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Shooter(Entity);

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

impl From<Hex> for Position {
    fn from(value: Hex) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[derive(Component, Reflect)]
pub enum Direction {
    #[default]
    North,
    South,
    Northeast,
    Southeast,
    Northwest,
    Southwest,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::North,
        Self::South,
        Self::Northeast,
        Self::Southeast,
        Self::Northwest,
        Self::Southwest,
    ];

    pub fn as_hex(&self) -> EdgeDirection {
        match self {
            Self::North => EdgeDirection::FLAT_NORTH,
            Self::South => EdgeDirection::FLAT_SOUTH,
            Self::Northeast => EdgeDirection::FLAT_NORTH_EAST,
            Self::Southeast => EdgeDirection::FLAT_SOUTH_EAST,
            Self::Northwest => EdgeDirection::FLAT_NORTH_WEST,
            Self::Southwest => EdgeDirection::FLAT_SOUTH_WEST,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::Northeast => Self::Southwest,
            Self::Southeast => Self::Northwest,
            Self::Northwest => Self::Southeast,
            Self::Southwest => Self::Northeast,
        }
    }

    pub fn left(&self) -> Self {
        match self {
            Self::North => Self::Northwest,
            Self::South => Self::Southeast,
            Self::Northeast => Self::North,
            Self::Southeast => Self::Northeast,
            Self::Northwest => Self::Southwest,
            Self::Southwest => Self::South,
        }
    }

    pub fn right(&self) -> Self {
        match self {
            Self::North => Self::Northeast,
            Self::South => Self::Southwest,
            Self::Northeast => Self::Southeast,
            Self::Southeast => Self::South,
            Self::Northwest => Self::North,
            Self::Southwest => Self::Northwest,
        }
    }

    pub fn front_directions(&self) -> [Self; 3] {
        match self {
            Self::North => [Self::Northeast, Self::North, Self::Northwest],
            Self::South => [Self::Southwest, Self::South, Self::Southeast],
            Self::Northeast => [Self::Southeast, Self::Northeast, Self::North],
            Self::Southeast => [Self::South, Self::Southeast, Self::Northeast],
            Self::Northwest => [Self::North, Self::Northwest, Self::Southwest],
            Self::Southwest => [Self::Northwest, Self::Southwest, Self::South],
        }
    }

    pub fn back_directions(&self) -> [Self; 3] {
        match self {
            Self::North => [Self::Southwest, Self::South, Self::Southeast],
            Self::South => [Self::Northeast, Self::North, Self::Northwest],
            Self::Northeast => [Self::Northwest, Self::Southwest, Self::South],
            Self::Southeast => [Self::North, Self::Northwest, Self::Southwest],
            Self::Northwest => [Self::South, Self::Southeast, Self::Northeast],
            Self::Southwest => [Self::Southeast, Self::Northeast, Self::North],
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
pub struct Refraction {
    facing: Direction,
}

impl Refraction {
    pub fn new(facing: Direction) -> Self {
        Refraction { facing }
    }

    pub fn refract(&self, incoming: Direction) -> Option<Direction> {
        if self
            .facing
            .back_directions()
            .iter()
            .any(|&direction| direction == incoming)
        {
            Some(self.facing.opposite())
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Reflection {
    facing: Direction,
}

impl Reflection {
    pub fn new(facing: Direction) -> Self {
        Reflection { facing }
    }

    pub fn reflect(&self, incoming: Direction) -> Option<Direction> {
        if incoming == self.facing.opposite() {
            Some(self.facing)
        } else if incoming == self.facing.left().opposite() {
            Some(self.facing.right())
        } else if incoming == self.facing.right().opposite() {
            Some(self.facing.left())
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Amplification(usize);

impl Amplification {
    pub fn new(strength: usize) -> Self {
        Amplification(strength)
    }
}

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct Consumption {
    entity: Entity,
    vulnerable: Vec<Direction>,
}

impl Consumption {
    fn new(tile: Entity, vulnerable: Vec<Direction>) -> Self {
        Consumption {
            entity: tile,
            vulnerable,
        }
    }

    pub fn bundle(tile: Entity, vulnerable: Vec<Direction>, position: Position) -> impl Bundle {
        (Consumption::new(tile, vulnerable), position)
    }
}

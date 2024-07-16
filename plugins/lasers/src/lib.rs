use bevy::prelude::*;
use hexx::*;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaserHitEvent>()
            .add_systems(Update, Self::track_lasers);
    }
}

impl LaserPlugin {
    fn track_lasers(
        lasers: Query<(&Position, &Direction), With<Laser>>,
        colliders: Query<
            (
                Entity,
                &Position,
                Option<&Refraction>,
                Option<&Amplification>,
                Option<&Consumption>,
            ),
            Or<(With<Refraction>, With<Amplification>, With<Consumption>)>,
        >,
        mut laser_hit_events: EventWriter<LaserHitEvent>,
    ) {
        'lasers: for (laser_position, laser_direction) in &lasers {
            const LASER_RANGE: usize = 100;
            const BASE_LASER_STRENGTH: usize = 1;

            let mut path = Vec::new();
            let mut current_position = *laser_position;
            let mut current_direction = *laser_direction;
            let mut strength = BASE_LASER_STRENGTH;

            for _ in 0..LASER_RANGE {
                let next_position: Position =
                    current_position.neighbor(current_direction.to_hex()).into();
                path.push(next_position);

                if let Some((collider, _, refraction, amplification, consumption)) = colliders
                    .iter()
                    .find(|(_, position, _, _, _)| **position == next_position)
                {
                    if let Some(refraction) = refraction {
                        current_direction = refraction.new_direction;
                    }
                    if let Some(amplification) = amplification {
                        strength += **amplification;
                    }
                    if consumption.is_some() {
                        laser_hit_events.send(LaserHitEvent {
                            consumer: collider,
                            strength,
                        });
                        break 'lasers;
                    }
                }
                current_position = next_position;
            }
        }
    }
}

#[derive(Event)]
pub struct LaserHitEvent {
    pub strength: usize,
    pub consumer: Entity,
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

#[derive(Clone, Copy, Debug, Default)]
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
pub struct Refraction {
    new_direction: Direction,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Amplification(usize);

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect, Deref, DerefMut)]

pub struct Consumption(Entity);

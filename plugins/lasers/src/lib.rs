use bevy::prelude::*;

use game_loop::{ActionCompleteEvent, GamePhase, MapSize};
use hexx::*;
use tilemap::{Tile, TilemapEntities};

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaserPathEvent>()
            .add_event::<LaserHitEvent>()
            .add_systems(
                Update,
                (Self::track_lasers, Self::despawn_lasers).in_set(LaserSystems),
            );
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
                Option<&YReflection>,
                Option<&Rotation>,
                Option<&Amplification>,
                Option<&Consumption>,
            ),
            Or<(
                With<Refraction>,
                With<Reflection>,
                With<YReflection>,
                With<Rotation>,
                With<Amplification>,
                With<Consumption>,
            )>,
        >,
        mut laser_hit_events: EventWriter<LaserHitEvent>,
        mut laser_path_events: EventWriter<LaserPathEvent>,
        games: Query<(Entity, &MapSize, &GamePhase)>,
        mut events: EventWriter<ActionCompleteEvent>,
    ) {
        let Ok((game, map_size, game_phase)) = games.get_single() else {
            return;
        };

        if !matches!(game_phase, GamePhase::Act) {
            return;
        };

        'all_lasers: for (laser_position, laser_direction, laser_shooter) in &lasers {
            let mut path = Vec::new();
            let mut current_position = *laser_position;
            path.push(current_position);
            let mut current_direction = *laser_direction;
            info!(
                "Simulating laser from starting position {:?} and direction {:?}",
                current_position, current_direction
            );
            let mut strength = Laser::POWER;

            loop {
                let next_position: Position =
                    current_position.neighbor(current_direction.as_hex()).into();
                // Change this later to exit the while loop appropriately and still progress the laser path outside the tilemap for visual effect
                // Need to consider edge case of reflector directly back on same path...
                if path.contains(&next_position) {
                    info!("Reached previously visited position");
                    break;
                }

                if (*current_position).unsigned_distance_to(Hex::ORIGIN)
                    > 3 * std::cmp::max(map_size.half_width, map_size.half_height) as u32
                {
                    info!("Laser out of bounds off map");
                    break;
                }

                if let Some((
                    collider,
                    _,
                    refraction,
                    reflection,
                    y_reflection,
                    rotation,
                    amplification,
                    consumption,
                )) = colliders
                    .iter()
                    .find(|(_, position, _, _, _, _, _, _)| **position == next_position)
                {
                    // Update the path with a new segment point upon collision with a tile
                    path.push(next_position);

                    if consumption.is_some()
                        && consumption
                            .unwrap()
                            .vulnerable
                            .iter()
                            .any(|&direction| direction == current_direction)
                    {
                        info!("Laser hit event found at {:?}", next_position);
                        laser_hit_events.send(LaserHitEvent {
                            consumer: collider,
                            strength,
                            shooter: **laser_shooter,
                        });
                        laser_path_events.send(LaserPathEvent { path: path.clone() });
                        continue 'all_lasers;
                    }

                    if let Some(refracted_direction) =
                        refraction.and_then(|refraction| refraction.refract(current_direction))
                    {
                        current_direction = refracted_direction;
                        info!("Laser refracted")
                    }
                    if let Some(reflected_direction) = reflection
                        .and_then(|reflection| Some(reflection.reflect(current_direction)))
                    {
                        current_direction = reflected_direction;
                    }
                    if let Some(y_reflection) = y_reflection {
                        current_direction = y_reflection.reflect(current_direction);
                    }
                    if let Some(rotation) = rotation {
                        current_direction = rotation.rotate(current_direction);
                    }
                    if let Some(amplification) = amplification {
                        strength += **amplification;
                    }
                }
                current_position = next_position;
            }

            // Update path with ending point, particularly important to mark if
            // no collisions or consumptions of the laser occur
            path.push(current_position);
            info!("Sent uninterrupted path event {:?}", path);
            laser_path_events.send(LaserPathEvent { path });
        }

        events.send(ActionCompleteEvent { game });
    }

    fn despawn_lasers(
        mut commands: Commands,
        games: Query<&GamePhase, Changed<GamePhase>>,
        lasers: Query<Entity, With<Laser>>,
    ) {
        if let Some(_) = games
            .get_single()
            .ok()
            .filter(|game_phase| matches!(game_phase, GamePhase::Draw))
        {
            for laser in &lasers {
                commands.entity(laser).despawn();
            }
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

impl Shooter {
    pub fn new(entity: Entity) -> Shooter {
        Self(entity)
    }

    pub fn inner(&self) -> Entity {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

impl From<Hex> for Position {
    fn from(value: Hex) -> Position {
        Self(value)
    }
}

impl From<Tile> for Position {
    fn from(value: Tile) -> Position {
        Self(*value)
    }
}

impl Position {
    pub fn get_tile_entity(&self, world: &mut World) -> Option<Entity> {
        let mut query = world.query::<&TilemapEntities>();
        query
            .get_single(world)
            .ok()
            .and_then(|tilemap| tilemap.tiles.get(&(**self)).copied())
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
        Self::from(self.as_hex().const_neg())
    }

    pub fn counterclockwise(&self, offset: u8) -> Self {
        Self::from(self.as_hex().rotate_ccw(offset))
    }

    pub fn clockwise(&self, offset: u8) -> Self {
        Self::from(self.as_hex().rotate_cw(offset))
    }

    // Returns the number of clockwise steps around hexagon to reach self from start
    pub fn steps_between(&self, start: Self) -> u8 {
        (self.as_hex().index() + 6 - start.as_hex().index()) % 6
    }

    pub fn front_directions(&self) -> [Self; 3] {
        [self.clockwise(1), *self, self.counterclockwise(1)]
    }

    pub fn back_directions(&self) -> [Self; 3] {
        let opposite_direction = self.opposite();
        [
            opposite_direction.clockwise(1),
            opposite_direction,
            opposite_direction.counterclockwise(1),
        ]
    }
}

impl From<EdgeDirection> for Direction {
    fn from(value: EdgeDirection) -> Self {
        match value {
            EdgeDirection::FLAT_NORTH => Self::North,
            EdgeDirection::FLAT_SOUTH => Self::South,
            EdgeDirection::FLAT_NORTH_EAST => Self::Northeast,
            EdgeDirection::FLAT_SOUTH_EAST => Self::Southeast,
            EdgeDirection::FLAT_NORTH_WEST => Self::Northwest,
            EdgeDirection::FLAT_SOUTH_WEST => Self::Southwest,
            _ => unreachable!("Edge direction should already produce something modded to within 0..6 in its inner field, so the previous cases should be comprehensive"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct Laser;

impl Laser {
    pub const POWER: usize = 1;
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

    pub fn reflect(&self, incoming: Direction) -> Direction {
        let opposite = self.facing.opposite();
        if incoming == opposite.counterclockwise(1) {
            self.facing.clockwise(1)
        } else if incoming == opposite.clockwise(1) {
            self.facing.counterclockwise(1)
        } else if incoming == self.facing.counterclockwise(1) {
            opposite.clockwise(1)
        } else if incoming == self.facing.clockwise(1) {
            opposite.counterclockwise(1)
        } else if incoming == self.facing {
            opposite
        } else {
            self.facing
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub enum YReflection {
    #[default]
    LeftTilt, // Represents a left-tilted "Y" inscribed in the hex
    RightTilt, // A right-tilted "Y" in the hex, the other possible orientation
}

impl YReflection {
    pub fn reflect(&self, incoming: Direction) -> Direction {
        match (self, incoming) {
            (YReflection::RightTilt, Direction::North) => Direction::Northeast,
            (YReflection::RightTilt, Direction::South) => Direction::Southeast,
            (YReflection::RightTilt, Direction::Northeast) => Direction::North,
            (YReflection::RightTilt, Direction::Southeast) => Direction::South,
            (YReflection::RightTilt, Direction::Northwest) => Direction::Southwest,
            (YReflection::RightTilt, Direction::Southwest) => Direction::Northwest,
            (YReflection::LeftTilt, Direction::North) => Direction::Northwest,
            (YReflection::LeftTilt, Direction::South) => Direction::Southwest,
            (YReflection::LeftTilt, Direction::Northeast) => Direction::Southeast,
            (YReflection::LeftTilt, Direction::Southeast) => Direction::Northeast,
            (YReflection::LeftTilt, Direction::Northwest) => Direction::North,
            (YReflection::LeftTilt, Direction::Southwest) => Direction::South,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Rotation(u8);

impl Rotation {
    pub fn new(offset: u8) -> Self {
        Rotation(offset)
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    fn rotate(&self, incoming: Direction) -> Direction {
        incoming.counterclockwise(self.get())
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

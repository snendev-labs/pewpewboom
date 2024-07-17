use bevy::{ecs::world::Command, prelude::*};

use health::Health;
use tiles::{
    lasers::{Consumption, Direction, Position},
    Tile, TilePlugin,
};
pub struct MountainPlugin;

impl Plugin for MountainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<MountainTile>::default());
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct MountainTile;

impl Tile for MountainTile {
    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        _direction: &Direction,
    ) -> impl Command {
        MountainActivate {
            tile: entity,
            position: *position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(MountainOnHit {
            tile: entity,
            strength,
        })
    }
}

pub struct MountainActivate {
    tile: Entity,
    position: Position,
}

impl Command for MountainActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Consumption::new(self.tile), self.position));
    }
}

pub struct MountainOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for MountainOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("MountainOnHit command should be fired for entity with a health component");
        **consumer_health -= self.strength;
    }
}

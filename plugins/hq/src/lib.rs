use bevy::{ecs::world::Command, prelude::*};

use health::Health;
use tiles::{
    lasers::{Consumption, Direction, Position},
    Tile, TilePlugin,
};
pub struct HQPlugin;

impl Plugin for HQPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<HQTile>::default());
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct HQTile;

impl Tile for HQTile {
    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        _direction: &Direction,
    ) -> impl Command {
        HQActivate {
            tile: entity,
            position: *position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(HQOnHit {
            tile: entity,
            strength,
        })
    }
}

pub struct HQActivate {
    tile: Entity,
    position: Position,
}

impl Command for HQActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Consumption::new(self.tile), self.position));
    }
}

pub struct HQOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for HQOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("HQOnHit command should be fired for entity with a health component");
        **consumer_health -= self.strength;
    }
}

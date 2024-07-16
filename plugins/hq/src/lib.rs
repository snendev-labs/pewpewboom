use bevy::{ecs::world::Command, prelude::*};

use health::Health;
use tiles::{
    lasers::{Consumption, Position},
    Tile,
};
pub struct HQPlugin;

impl Plugin for HQPlugin {
    fn build(&self, app: &mut App) {}
}

pub struct HQTile;

impl Tile for HQTile {
    fn activate(&self, entity: Entity, position: Position) -> impl Command {
        HQActivate { entity, position }
    }

    fn on_hit(&self, entity: Entity, strength: usize) -> Option<impl Command> {
        Some(HQOnHit { entity, strength })
    }
}

pub struct HQActivate {
    entity: Entity,
    position: Position,
}

impl Command for HQActivate {
    fn apply(self, world: &mut World) {
        world.spawn((
            Consumption {
                target: self.entity,
            },
            self.position,
        ));
    }
}

pub struct HQOnHit {
    entity: Entity,
    strength: usize,
}

impl Command for HQOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.entity)
            .expect("HQOnHit command should be fired for entity with a halth component");
        **consumer_health -= self.strength;
    }
}

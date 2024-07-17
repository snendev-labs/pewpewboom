use bevy::{ecs::world::Command, prelude::*};

use health::Health;
use merchandise::Money;
use tiles::{
    lasers::{Consumption, Direction, Position},
    Tile, TilePlugin,
};

pub struct ResourceDepositPlugin;

impl Plugin for ResourceDepositPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<ResourceDepositTile>::default());
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct ResourceDepositTile;

impl Tile for ResourceDepositTile {
    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        _direction: &Direction,
    ) -> impl Command {
        ResourceDepositActivate {
            tile: entity,
            position: *position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, shooter: Entity) -> Option<impl Command> {
        Some(ResourceDepositOnHit {
            tile: entity,
            strength,
            shooter,
        })
    }
}

pub struct ResourceDepositActivate {
    tile: Entity,
    position: Position,
}

impl Command for ResourceDepositActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Consumption::new(self.tile), self.position.clone()));
    }
}

pub struct ResourceDepositOnHit {
    tile: Entity,
    strength: usize,
    shooter: Entity,
}

impl Command for ResourceDepositOnHit {
    fn apply(self, world: &mut World) {
        let mut resource_health = world
            .get_mut::<Health>(self.tile)
            .expect("Resource tile should have health");

        **resource_health -= self.strength;
        if **resource_health == 0 {
            let mut money_query = world.query::<&mut Money>();
            let [resource_money, mut shooter_money] = money_query
                .get_many_mut(world, [self.tile, self.shooter])
                .expect("Shooting player and resource tile should both have money components");

            **shooter_money += **resource_money;
        }
    }
}

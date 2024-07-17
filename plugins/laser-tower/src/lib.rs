use bevy::{ecs::world::Command, prelude::*};

use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Consumption, Direction, Laser, Position},
    Tile, TilePlugin,
};

pub struct LaserTowerPlugin;

impl Plugin for LaserTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<LaserTower>::default());
        app.define_merchandise::<LaserTower>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct LaserTower;

impl Tile for LaserTower {
    fn activate(&self, tile: Entity, position: &Position, direction: &Direction) -> impl Command {
        LaserTowerActivate {
            tile,
            position: *position,
            direction: *direction,
        }
    }
}

impl Merchandise for LaserTower {
    const PRICE: Money = Money::new(10);
}

#[derive(Clone, Debug)]
pub struct LaserTowerActivate {
    tile: Entity,
    position: Position,
    direction: Direction,
}

impl Command for LaserTowerActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Consumption::new(self.tile), self.position));
        world.spawn((Laser, self.position, self.direction));
    }
}

#[derive(Clone, Debug)]
pub struct LaserTowerOnHit {
    _entity: Entity,
    // strength: Entity,
}

impl Command for LaserTowerOnHit {
    fn apply(self, _world: &mut World) {
        // world
        // .get_mut::<Health>()
        // .expect("Laser tower to have Health");
    }
}

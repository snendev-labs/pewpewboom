use bevy::{ecs::world::Command, prelude::*};

use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Direction, Position, Refraction},
    Tile, TilePlugin,
};

pub struct RefractorPlugin;

impl Plugin for RefractorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<RefractorTile>::default());
        app.define_merchandise::<RefractorTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct RefractorTile;

impl Merchandise for RefractorTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Refractor Tower";
}

impl Tile for RefractorTile {
    fn activate(
        &self,
        _entity: Entity,
        position: &Position,
        direction: &Direction,
    ) -> impl Command {
        RefractorActivate {
            position: *position,
            direction: *direction,
        }
    }
}

pub struct RefractorActivate {
    position: Position,
    direction: Direction,
}

impl Command for RefractorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Refraction::new(self.direction), self.position.clone()));
    }
}

// Possible on-hit I we want destroyable refractorss ...

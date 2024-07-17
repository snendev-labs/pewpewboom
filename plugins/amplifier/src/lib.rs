use bevy::{ecs::world::Command, prelude::*};

use merchandise::{Merchandise, Money};
use tiles::{
    lasers::{Amplification, Direction, Position},
    Tile, TilePlugin,
};

pub struct AmplifierPlugin;

impl Plugin for AmplifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<AmplifierTile>::default());
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct AmplifierTile;

impl Merchandise for AmplifierTile {
    const PRICE: Money = Money::new(3);
}

impl Tile for AmplifierTile {
    fn activate(
        &self,
        _entity: Entity,
        position: &Position,
        _direction: &Direction,
    ) -> impl Command {
        AmplifierActivate {
            position: *position,
        }
    }
}

pub struct AmplifierActivate {
    position: Position,
}

impl Command for AmplifierActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Amplification::new(1), self.position.clone()));
    }
}

// Possible on-hit I we want destroyable amplifiers ...

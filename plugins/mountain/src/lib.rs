use bevy::{color::palettes, ecs::world::Command, prelude::*};

use health::Health;
use tiles::{
    lasers::{Consumption, Direction, Position, Rotation},
    Tile, TileParameters, TilePlugin,
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
    fn spawn(parameters: TileParameters, _player: Entity) -> impl Command {
        MountainSpawn {
            position: parameters.position,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::BEIGE))
    }

    fn activate(
        &self,
        entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        MountainActivate {
            tile: entity,
            position: parameters.position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(MountainOnHit {
            tile: entity,
            strength,
        })
    }
}

pub struct MountainSpawn {
    position: Position,
}

impl Command for MountainSpawn {
    fn apply(self, world: &mut World) {
        world.spawn((MountainTile, self.position));
        // Needs the InGame added here too
    }
}

pub struct MountainActivate {
    tile: Entity,
    position: Position,
}

impl Command for MountainActivate {
    fn apply(self, world: &mut World) {
        world.spawn(Consumption::bundle(
            self.tile,
            Direction::ALL.to_vec(),
            self.position,
        ));
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

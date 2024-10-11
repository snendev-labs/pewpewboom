use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use health::Health;
use tiles::{
    lasers::{Consumption, Direction, Position},
    Owner, Tile, TileParameters, TilePlugin,
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
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        HQSpawn {
            position: parameters.position,
            player: player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::GREEN))
    }

    fn activate(
        &self,
        entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        HQActivate {
            tile: entity,
            position: parameters.position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(HQOnHit {
            tile: entity,
            strength,
        })
    }
}

pub struct HQSpawn {
    position: Position,
    player: Entity,
}

impl Command for HQSpawn {
    fn apply(self, world: &mut World) {
        if let Some(game) = world.get::<InGame>(self.player) {
            world.spawn((HQTile, self.position, Owner::new(self.player), game.clone()));
        }
    }
}

pub struct HQActivate {
    tile: Entity,
    position: Position,
}

impl Command for HQActivate {
    fn apply(self, world: &mut World) {
        world.spawn(Consumption::bundle(
            self.tile,
            Direction::ALL.to_vec(),
            self.position,
        ));
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

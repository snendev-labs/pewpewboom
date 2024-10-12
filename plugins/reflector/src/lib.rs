use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Direction, Position, Reflection},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct ReflectorPlugin;

impl Plugin for ReflectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<ReflectorTile>::default());
        app.define_merchandise::<ReflectorTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct ReflectorTile;

impl Tile for ReflectorTile {
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        ReflectorSpawn {
            position: parameters.position,
            direction: parameters.direction.unwrap_or_default(),
            player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

    fn activate(
        &self,
        _entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        ReflectorActivate {
            position: parameters.position,
            direction: parameters
                .direction
                .unwrap_or_else(|| panic!("Reflector needs a direction")),
        }
    }
}

impl Merchandise for ReflectorTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Reflector Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct ReflectorSpawn {
    position: Position,
    direction: Direction,
    player: Entity,
}

impl Command for ReflectorSpawn {
    fn apply(self, world: &mut World) {
        if let Some(game) = world.get::<InGame>(self.player) {
            world.spawn((
                ReflectorTile,
                TileParameters {
                    position: self.position,
                    direction: Some(self.direction),
                    ..default()
                },
                Owner::new(self.player),
                game.clone(),
            ));
        }
    }
}

pub struct ReflectorActivate {
    position: Position,
    direction: Direction,
}

impl Command for ReflectorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Reflection::new(self.direction), self.position));
    }
}

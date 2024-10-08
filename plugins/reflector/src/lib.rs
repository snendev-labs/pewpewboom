use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Direction, Position, Reflection, Rotation},
    Owner, Tile, TilePlugin,
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
    fn spawn(
        position: &Position,
        direction: &Direction,
        _rotation: &Rotation,
        player: &Entity,
    ) -> impl Command {
        ReflectorSpawn {
            position: *position,
            direction: *direction,
            player: *player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

    fn activate(
        &self,
        _entity: Entity,
        position: &Position,
        direction: &Direction,
        _rotation: &Rotation,
        _shooter: &Entity,
    ) -> impl Command {
        ReflectorActivate {
            position: *position,
            direction: *direction,
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
                self.position,
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

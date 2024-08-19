use bevy::{color::palettes, ecs::world::Command, prelude::*};

use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Direction, Position, Reflection, Rotation},
    Tile, TilePlugin,
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
    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

    fn activate(
        &self,
        _entity: Entity,
        position: &Position,
        direction: &Direction,
        _rotation: &Rotation,
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

pub struct ReflectorActivate {
    position: Position,
    direction: Direction,
}

impl Command for ReflectorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Reflection::new(self.direction), self.position));
    }
}

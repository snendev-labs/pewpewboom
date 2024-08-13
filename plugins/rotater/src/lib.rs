use bevy::{color::palettes, ecs::world::Command, prelude::*};

use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Direction, Position, Rotation},
    Tile, TilePlugin,
};

pub struct RotaterPlugin;

impl Plugin for RotaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<RotaterTile>::default());
        app.define_merchandise::<RotaterTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct RotaterTile;

impl Tile for RotaterTile {
    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

    fn activate(
        &self,
        _entity: Entity,
        position: &Position,
        direction: &Direction,
        rotation: &Rotation,
    ) -> impl Command {
        RotaterActivate {
            position: *position,
            rotation: *rotation,
        }
    }
}

impl Merchandise for RotaterTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Rotater";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct RotaterActivate {
    position: Position,
    rotation: Rotation,
}

impl Command for RotaterActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Rotation::new(self.rotation.get()), self.position));
    }
}

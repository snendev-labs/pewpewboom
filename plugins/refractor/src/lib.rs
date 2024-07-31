use bevy::{color::palettes, ecs::world::Command, prelude::*};

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

impl Tile for RefractorTile {
    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CADET_BLUE))
    }

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

impl Merchandise for RefractorTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Refractor Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
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

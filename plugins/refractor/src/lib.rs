use bevy::{color::palettes, ecs::world::Command, prelude::*};

use health::Health;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Consumption, Direction, Position, Refraction, Rotation},
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
        ColorMaterial::from_color(Color::Srgba(palettes::css::CORNFLOWER_BLUE))
    }

    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        direction: &Direction,
        _rotation: &Rotation,
    ) -> impl Command {
        RefractorActivate {
            tile: entity,
            position: *position,
            direction: *direction,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(RefractorOnHit {
            tile: entity,
            strength,
        })
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
    tile: Entity,
    position: Position,
    direction: Direction,
}

impl Command for RefractorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((
            Refraction::new(self.direction),
            Consumption::bundle(
                self.tile,
                self.direction.back_directions().to_vec(),
                self.position.clone(),
            ),
        ));
    }
}

pub struct RefractorOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for RefractorOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("RefractorOnHit entity should have Health component added to it");
        **consumer_health -= self.strength;
    }
}

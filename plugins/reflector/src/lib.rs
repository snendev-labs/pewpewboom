use bevy::{color::palettes, ecs::world::Command, prelude::*};

use health::Health;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Consumption, Direction, Position, Reflection},
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

    fn activate(&self, entity: Entity, position: &Position, direction: &Direction) -> impl Command {
        ReflectorActivate {
            tile: entity,
            position: *position,
            direction: *direction,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(ReflectorOnHit {
            tile: entity,
            strength,
        })
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
    tile: Entity,
    position: Position,
    direction: Direction,
}

impl Command for ReflectorActivate {
    fn apply(self, world: &mut World) {
        world.spawn((
            Reflection::new(self.direction),
            Consumption::bundle(
                self.tile,
                self.direction.back_directions().to_vec(),
                self.position.clone(),
            ),
        ));
    }
}

pub struct ReflectorOnHit {
    tile: Entity,
    strength: usize,
}
impl Command for ReflectorOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("RefractorOnHit entity should have Health component added to it");
        **consumer_health -= self.strength;
    }
}

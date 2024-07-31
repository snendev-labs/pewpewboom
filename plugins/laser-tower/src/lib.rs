use bevy::{color::palettes, ecs::world::Command, prelude::*};

use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Consumption, Direction, Laser, Position},
    Tile, TilePlugin,
};

pub struct LaserTowerPlugin;

impl Plugin for LaserTowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<LaserTower>::default());
        app.define_merchandise::<LaserTower>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct LaserTower;

impl Tile for LaserTower {
    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::CRIMSON))
    }

    fn activate(&self, tile: Entity, position: &Position, direction: &Direction) -> impl Command {
        LaserTowerActivate {
            tile,
            position: *position,
            direction: *direction,
        }
    }
}

impl Merchandise for LaserTower {
    const PRICE: Money = Money::new(10);
    const NAME: &'static str = "Laser Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

#[derive(Clone, Debug)]
pub struct LaserTowerActivate {
    tile: Entity,
    position: Position,
    direction: Direction,
}

impl Command for LaserTowerActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Consumption::new(self.tile), self.position));
        world.spawn((Laser, self.position, self.direction));
    }
}

#[derive(Clone, Debug)]
pub struct LaserTowerOnHit {
    tile: Entity,
    // strength: Entity,
}

impl Command for LaserTowerOnHit {
    fn apply(self, _world: &mut World) {
        // world
        // .get_mut::<Health>()
        // .expect("Laser tower to have Health");
    }
}

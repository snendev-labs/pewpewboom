use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Position, Rotation},
    Owner, Tile, TileParameters, TilePlugin,
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
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        RotaterSpawn {
            position: parameters.position,
            rotation: parameters.rotation.unwrap_or_default(),
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
        RotaterActivate {
            position: parameters.position,
            rotation: parameters
                .rotation
                .unwrap_or_else(|| panic!("Rotator needs a rotation")),
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

pub struct RotaterSpawn {
    position: Position,
    rotation: Rotation,
    player: Entity,
}

impl Command for RotaterSpawn {
    fn apply(self, world: &mut World) {
        if let Some(game) = world.get::<InGame>(self.player) {
            world.spawn((
                RotaterTile,
                TileParameters {
                    position: self.position,
                    rotation: Some(self.rotation),
                    ..default()
                },
                Owner::new(self.player),
                game.clone(),
            ));
        }
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

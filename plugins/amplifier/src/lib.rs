use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use tiles::{
    lasers::{Amplification, Direction, Position, Rotation},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct AmplifierPlugin;

impl Plugin for AmplifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<AmplifierTile>::default());
        app.define_merchandise::<AmplifierTile>();
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct AmplifierTile;

impl Tile for AmplifierTile {
    fn spawn(parameters: TileParameters, player: Entity) -> impl Command {
        AmplifierSpawn {
            position: parameters.position,
            player,
        }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::DARK_ORANGE))
    }

    fn activate(
        &self,
        _entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        AmplifierActivate {
            position: parameters.position,
        }
    }
}

impl Merchandise for AmplifierTile {
    const PRICE: Money = Money::new(3);
    const NAME: &'static str = "Amplifier Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct AmplifierSpawn {
    position: Position,
    player: Entity,
}

impl Command for AmplifierSpawn {
    fn apply(self, world: &mut World) {
        if let Some(game) = world.get::<InGame>(self.player) {
            world.spawn((
                AmplifierTile,
                self.position.clone(),
                Owner::new(self.player),
                game.clone(),
            ));
        }
    }
}

pub struct AmplifierActivate {
    position: Position,
}

impl Command for AmplifierActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Amplification::new(1), self.position.clone()));
    }
}

// on-hit for amplifier later

use bevy::{color::palettes, ecs::world::Command, prelude::*};

use game_loop::InGame;
use health::Health;
use popups::PopupEvent;
use tiles::{
    lasers::{Consumption, Direction, Position},
    Tile, TileParameters, TilePlugin,
};
pub struct MountainPlugin;

impl Plugin for MountainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<MountainTile>::default());
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct MountainTile;

impl Tile for MountainTile {
    fn spawn(position: Position, _player: Entity, game: Entity) -> impl Command {
        MountainSpawn { position, game }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        ColorMaterial::from_color(Color::Srgba(palettes::css::BLACK))
    }

    fn activate(
        &self,
        entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        MountainActivate {
            tile: entity,
            position: parameters.position,
        }
    }

    fn on_hit(&self, entity: Entity, strength: usize, _shooter: Entity) -> Option<impl Command> {
        Some(MountainOnHit {
            tile: entity,
            strength,
        })
    }
}

pub struct MountainSpawn {
    position: Position,
    game: Entity,
}

impl Command for MountainSpawn {
    fn apply(self, world: &mut World) {
        world.spawn((
            MountainTile,
            self.position,
            Health::new(5),
            InGame::new(self.game),
        ));
    }
}

pub struct MountainActivate {
    tile: Entity,
    position: Position,
}

impl Command for MountainActivate {
    fn apply(self, world: &mut World) {
        world.spawn(Consumption::bundle(
            self.tile,
            Direction::ALL.to_vec(),
            self.position,
        ));
    }
}

pub struct MountainOnHit {
    tile: Entity,
    strength: usize,
}

impl Command for MountainOnHit {
    fn apply(self, world: &mut World) {
        let mut consumer_health = world
            .get_mut::<Health>(self.tile)
            .expect("MountainOnHit command should be fired for entity with a health component");
        **consumer_health -= self.strength;
        world.trigger_targets(
            PopupEvent {
                text: String::from(format!("-{}", self.strength)),
            },
            self.tile,
        );
    }
}

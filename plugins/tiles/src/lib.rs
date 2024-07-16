use std::marker::PhantomData;

use bevy::{ecs::world::Command, prelude::*};

use game_loop::{GamePhase, InGame};
use lasers::{Direction, LaserHitEvent, LaserPlugin, LaserSystems, Position};

pub use lasers;

pub trait Tile {
    fn activate(&self, entity: Entity, position: &Position, direction: &Direction) -> impl Command;

    #[allow(unused_variables)]
    fn on_hit(&self, entity: Entity) -> Option<impl Command> {
        None as Option<fn(&mut World)>
    }
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LaserPlugin).configure_sets(
            Update,
            (TileSystems::Activate, LaserSystems, TileSystems::OnHit).chain(),
        );
    }
}

pub struct TilePlugin<T> {
    marker: PhantomData<T>,
}

impl<T> Default for TilePlugin<T> {
    fn default() -> Self {
        TilePlugin {
            marker: PhantomData::<T>,
        }
    }
}

impl<T> Plugin for TilePlugin<T>
where
    T: Tile + Component + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::activate_tiles.in_set(TileSystems::Activate),
                Self::handle_hit_tiles.in_set(TileSystems::OnHit),
            ),
        );
    }
}

impl<T> TilePlugin<T>
where
    T: Tile + Component,
{
    fn activate_tiles(
        mut commands: Commands,
        activated_games: Query<(Entity, &GamePhase), Changed<GamePhase>>,
        activated_tiles: Query<(Entity, &Position, &Direction, &T, &InGame)>,
    ) {
        let mut sorted_tiles = activated_tiles.iter().sort::<&InGame>().peekable();
        for (game, phase) in activated_games.iter().sort::<Entity>() {
            if *phase != GamePhase::Act {
                continue;
            }

            let (entity, position, direction, tile, _) = sorted_tiles
                .find(|(_, _, _, _, in_game)| ***in_game == game)
                .expect(
                    format!("failed to find tiles for game {:?}! invalid sort?", game).as_str(),
                );
            info!("Found tiles for game {:?}", game);
            info!("Processing entity {:?} in game {:?}", entity, game);
            commands.add(tile.activate(entity, position, direction));

            while sorted_tiles
                .peek()
                .is_some_and(|(_, _, _, _, in_game)| ***in_game == game)
            {
                let (entity, position, direction, tile, _) = sorted_tiles.next().unwrap();
                info!("Processing entity {:?} in game {:?}", entity, game);
                commands.add(tile.activate(entity, position, direction));
            }
        }
    }

    fn handle_hit_tiles(
        mut commands: Commands,
        mut collisions: EventReader<LaserHitEvent>,
        tiles: Query<(Entity, &Position, &T)>,
    ) {
        for event in collisions.read() {
            if let Ok((entity, _position, tile)) = tiles.get(event.consumer) {
                if let Some(command) = tile.on_hit(entity) {
                    commands.add(command);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum TileSystems {
    Activate,
    OnHit,
}

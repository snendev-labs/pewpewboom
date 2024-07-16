use std::marker::PhantomData;

use bevy::{ecs::world::Command, prelude::*};

use lasers::{LaserHitEvent, LaserSystems, Position};

pub use lasers;

pub trait Tile {
    fn activate(&self, entity: Entity, position: &Position) -> impl Command;

    fn on_hit(&self, entity: Entity) -> Option<impl Command>;
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
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

    fn activate_tiles(mut commands: Commands, activated_query: Query<(Entity, &Position, &T)>) {
        for (entity, position, tile) in &activated_query {
            commands.add(tile.activate(entity, position))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum TileSystems {
    Activate,
    OnHit,
}

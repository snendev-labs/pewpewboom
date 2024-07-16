use std::marker::PhantomData;

use bevy::{ecs::world::Command, prelude::*};

use lasers::{LaserHitEvent, Position};

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
                TileSystemSet::activate_tiles::<T>,
                TileSystemSet::handle_hit_tiles::<T>,
            )
                .in_set(TileSystemSet),
        );
    }
}

pub trait Tile {
    fn activate(&self, entity: Entity, position: &Position) -> impl Command;

    fn on_hit(&self, entity: Entity) -> Option<impl Command>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct TileSystemSet;

impl TileSystemSet {
    fn handle_hit_tiles<T: Tile + Component>(
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

    fn activate_tiles<T: Tile + Component>(
        mut commands: Commands,
        activated_query: Query<(Entity, &Position, &T)>,
    ) {
        for (entity, position, tile) in &activated_query {
            commands.add(tile.activate(entity, position))
        }
    }
}

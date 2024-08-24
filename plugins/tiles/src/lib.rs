use std::marker::PhantomData;

use bevy::{
    ecs::world::Command,
    prelude::{
        info, Added, App, AssetServer, Assets, Changed, ColorMaterial, Commands, Component, Entity,
        EventReader, Handle, IntoSystemConfigs, IntoSystemSetConfigs, Plugin, Query,
        RemovedComponents, Res, ResMut, SystemSet, Update, World,
    },
};

use game_loop::{GamePhase, InGame};
use lasers::{Direction, LaserHitEvent, LaserPlugin, LaserSystems, Position, Rotation};
use tilemap::EmptyTileMaterial;

pub use lasers;

pub trait Tile {
    fn material(asset_server: &AssetServer) -> ColorMaterial;

    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        direction: &Direction,
        rotation: &Rotation,
    ) -> impl Command;

    #[allow(unused_variables)]
    fn on_hit(&self, entity: Entity, strength: usize, shooter: Entity) -> Option<impl Command> {
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
                Self::add_tile_material.in_set(TileSystems::Add),
                Self::activate_tiles.in_set(TileSystems::Activate),
                Self::handle_hit_tiles.in_set(TileSystems::OnHit),
                Self::remove_tile_material.in_set(TileSystems::Remove),
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
        activated_tiles: Query<(Entity, &Position, &Direction, &Rotation, &T, &InGame)>,
    ) {
        let mut sorted_tiles = activated_tiles.iter().sort::<&InGame>().peekable();
        for (game, phase) in activated_games.iter().sort::<Entity>() {
            if !matches!(phase, GamePhase::Act) {
                continue;
            }

            let (entity, position, direction, rotation, tile, _) = sorted_tiles
                .find(|(_, _, _, _, _, in_game)| ***in_game == game)
                .unwrap_or_else(|| {
                    panic!("failed to find tiles for game {:?}! invalid sort?", game);
                });
            info!("Found tiles for game {:?}", game);
            info!("Processing entity {:?} in game {:?}", entity, game);
            commands.add(tile.activate(entity, position, direction, rotation));

            while sorted_tiles
                .peek()
                .is_some_and(|(_, _, _, _, _, in_game)| ***in_game == game)
            {
                let (entity, position, direction, rotation, tile, _) = sorted_tiles.next().unwrap();
                info!("Processing entity {:?} in game {:?}", entity, game);
                commands.add(tile.activate(entity, position, direction, rotation));
            }
        }
    }

    fn handle_hit_tiles(
        mut commands: Commands,
        mut collisions: EventReader<LaserHitEvent>,
        tiles: Query<(Entity, &Position, &T)>,
    ) {
        for LaserHitEvent {
            strength,
            consumer,
            shooter,
        } in collisions.read()
        {
            if let Ok((entity, _position, tile)) = tiles.get(*consumer) {
                if let Some(command) = tile.on_hit(entity, *strength, *shooter) {
                    commands.add(command);
                }
            }
        }
    }

    fn add_tile_material(
        mut tile_materials: Query<&mut Handle<ColorMaterial>, Added<T>>,
        mut assets: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        for mut material in &mut tile_materials {
            *material = assets.add(T::material(&asset_server));
        }
    }

    fn remove_tile_material(
        mut removed_tile: RemovedComponents<T>,
        mut tile_materials: Query<&mut Handle<ColorMaterial>>,
        empty_tile_material: Res<EmptyTileMaterial>,
    ) {
        for tile in removed_tile.read() {
            if let Ok(mut tile_material) = tile_materials.get_mut(tile) {
                *tile_material = empty_tile_material.clone();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum TileSystems {
    Add,
    Activate,
    OnHit,
    Remove,
}

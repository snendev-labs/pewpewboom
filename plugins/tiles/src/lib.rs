use std::{any::TypeId, marker::PhantomData};

use bevy::{
    ecs::world::Command,
    prelude::{
        info, Added, App, AssetServer, Assets, Changed, ColorMaterial, Commands, Component, Entity,
        Event, EventReader, Handle, IntoSystemConfigs, IntoSystemSetConfigs, Or, Plugin, Query,
        Res, ResMut, SystemSet, Update, With, World,
    },
};

use game_loop::{GamePhase, InGame};
pub use lasers;
use lasers::{Direction, LaserHitEvent, LaserPlugin, LaserSystems, Position, Rotation};
use tilemap::{EmptyTile, EmptyTileMaterial, Tilemap, TilemapEntities};

pub trait Tile {
    fn spawn(
        position: &Position,
        direction: &Direction,
        rotation: &Rotation,
        player: &Entity,
    ) -> impl Command;

    fn material(asset_server: &AssetServer) -> ColorMaterial;

    fn activate(
        &self,
        entity: Entity,
        position: &Position,
        direction: &Direction,
        rotation: &Rotation,
        shooter: &Entity,
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
        app.add_event::<TileSpawnEvent>().add_systems(
            Update,
            (
                Self::spawn_tiles.in_set(TileSystems::Spawn),
                Self::activate_tiles.in_set(TileSystems::Activate),
                Self::handle_hit_tiles.in_set(TileSystems::OnHit),
                Self::update_tile_material,
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
        games: Query<&GamePhase>,
        activated_tiles: Query<(
            Entity,
            &Position,
            &Direction,
            &Rotation,
            &Owner,
            &T,
            &InGame,
        )>,
    ) {
        let mut sorted_tiles = activated_tiles.iter().sort::<&InGame>().peekable();
        let total_active_games = games
            .iter()
            .filter(|game| matches!(game, GamePhase::Act))
            .collect::<Vec<_>>()
            .len();
        info!("Found {:?} active games", total_active_games);
        for (game, phase) in activated_games.iter().sort::<Entity>() {
            if !matches!(phase, GamePhase::Act) {
                continue;
            }
            info!("Found active game");

            let (entity, position, direction, rotation, owner, tile, _) = sorted_tiles
                .find(|(_, _, _, _, _, _, in_game)| ***in_game == game)
                .unwrap_or_else(|| {
                    panic!("failed to find tiles for game {:?}! invalid sort?", game);
                });
            info!("Found tiles for game {:?}", game);
            info!("Processing entity {:?} in game {:?}", entity, game);
            commands.add(tile.activate(entity, position, direction, rotation, &owner.inner()));

            while sorted_tiles
                .peek()
                .is_some_and(|(_, _, _, _, _, _, in_game)| ***in_game == game)
            {
                let (entity, position, direction, rotation, owner, tile, _) =
                    sorted_tiles.next().unwrap();
                info!("Processing entity {:?} in game {:?}", entity, game);
                commands.add(tile.activate(entity, position, direction, rotation, &owner.inner()));
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

    fn spawn_tiles(
        mut commands: Commands,
        mut tile_spawns: EventReader<TileSpawnEvent>,
        tilemaps: Query<&TilemapEntities, With<Tilemap>>,
    ) {
        let tile_spawns = tile_spawns
            .read()
            .filter_map(|tile_spawn| {
                if TypeId::of::<T>() == tile_spawn.tile_id {
                    Some(tile_spawn)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for tilemap_entities in tilemaps.iter() {
            for tile_spawn in &tile_spawns {
                for (hex, tile_entity) in tilemap_entities.iter() {
                    if *tile_entity == tile_spawn.on_tile {
                        commands.add(T::spawn(
                            &Position::from(*hex),
                            &Direction::default(),
                            &Rotation::default(),
                            &tile_spawn.player,
                        ));

                        commands.entity(*tile_entity).remove::<EmptyTile>();
                    }
                }
            }
        }
    }

    fn update_tile_material(
        added_tiles: Query<
            (&Position, Option<&T>, Option<&EmptyTile>),
            Or<(Added<T>, Added<EmptyTile>)>,
        >,
        tilemaps: Query<&TilemapEntities, With<Tilemap>>,
        mut materials: Query<&mut Handle<ColorMaterial>>,
        mut material_assets: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
        empty_tile_material: Res<EmptyTileMaterial>,
    ) {
        let Ok(tiles) = tilemaps.get_single() else {
            return;
        };

        for (position, tile, empty) in &added_tiles {
            info!("Tile added (or  removed)");
            let hex = **position;
            if let Some(mut material) = tiles
                .get(&hex)
                .and_then(|&entity| materials.get_mut(entity).ok())
            {
                info!("Material is recognized");
                if let Some(_) = tile {
                    info!("Change to tile material {:?}", T::material(&asset_server));
                    *material = material_assets.add(T::material(&asset_server));
                }

                if let Some(_) = empty {
                    info!("Change to empty tile material");
                    *material = empty_tile_material.clone_weak();
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum TileSystems {
    Spawn,
    Activate,
    OnHit,
}

#[derive(Event)]
pub struct TileSpawnEvent {
    pub tile_id: TypeId,
    pub on_tile: Entity,
    pub player: Entity,
}

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct Owner(Entity);

impl Owner {
    pub fn new(entity: Entity) -> Owner {
        Self(entity)
    }

    pub fn inner(&self) -> Entity {
        self.0
    }
}

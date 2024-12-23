use std::{any::TypeId, collections::HashSet, marker::PhantomData};

use bevy::{
    color::{Color, Mix},
    ecs::world::Command,
    prelude::{
        info, Added, App, AssetServer, Assets, Changed, ColorMaterial, Commands, Component, Deref,
        DerefMut, Entity, Event, EventReader, Handle, IntoSystemConfigs, IntoSystemSetConfigs, Or,
        Plugin, Query, Res, ResMut, SystemSet, Update, With, World,
    },
};

use game_loop::{GameLoopSystems, GamePhase, InGame, Player, PlayerColorAdjuster};
pub use lasers;
use lasers::{
    Amplification, Direction, LaserHitEvent, LaserPlugin, LaserSystems, Position, Rotation,
};
use tilemap::{EmptyTile, EmptyTileMaterial, Tilemap, TilemapEntities};

pub trait Tile {
    #[allow(unused_variables)]
    fn spawn(position: Position, player: Entity, game: Entity) -> impl Command;

    fn material(asset_server: &AssetServer) -> ColorMaterial;

    fn activate(
        &self,
        entity: Entity,
        parameters: TileParameters,
        shooter: Option<Entity>,
    ) -> impl Command;

    #[allow(unused_variables)]
    fn on_hit(&self, entity: Entity, strength: usize, shooter: Entity) -> Option<impl Command> {
        None as Option<fn(&mut World)>
    }
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LaserPlugin)
            .configure_sets(
                Update,
                (
                    TileSystems::Activate.after(GameLoopSystems),
                    LaserSystems,
                    TileSystems::OnHit,
                )
                    .chain(),
            )
            .add_systems(Update, Self::update_territories);
    }
}

impl TilesPlugin {
    // This could be better optimized in the future by only running conditional on a tile spawning or
    // being destroyed under a player's control and using a `Changed` query filter to track that
    // Owned tiles would need to be tracked by a marker component on the player entity or possibly
    // as children under the entity to implement this
    fn update_territories(
        mut commands: Commands,
        tilemaps: Query<&TilemapEntities>,
        tiles: Query<(&Position, &Owner)>,
        mut territories: Query<(Entity, Option<&mut Territory>), With<Player>>,
    ) {
        let Ok(tilemap) = tilemaps.get_single() else {
            info!("Found none or multiple tilemaps");
            return;
        };
        let mut sorted_tiles = tiles.iter().sort::<&Owner>().peekable();

        for (player, territory) in &mut territories.iter_mut().sort::<Entity>() {
            let mut updated_territory: HashSet<Entity> = HashSet::new();

            let Some((position, _)) = sorted_tiles.find(|(_, owner)| ***owner == player) else {
                info!(
                    "Could not find the relevant owned tiles for player with entity {}",
                    player
                );
                continue;
            };

            for coord in (**position).range(Territory::RANGE as u32) {
                if let Some(tile_entity) = tilemap.tiles.get(&coord) {
                    updated_territory.insert(*tile_entity);
                }
            }

            while sorted_tiles
                .peek()
                .is_some_and(|(_, owner)| ***owner == player)
            {
                let (position, _) = sorted_tiles.next().unwrap();
                for coord in (**position).range(Territory::RANGE as u32) {
                    if let Some(tile_entity) = tilemap.tiles.get(&coord) {
                        updated_territory.insert(*tile_entity);
                    }
                }
            }

            if let Some(mut territory) = territory {
                **territory = updated_territory;
            } else {
                commands.entity(player).insert(Territory(updated_territory));
            }
        }
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
        activated_tiles: Query<(
            Entity,
            &Position,
            Option<&Direction>,
            Option<&Rotation>,
            Option<&Amplification>,
            Option<&Owner>,
            &T,
            &InGame,
        )>,
    ) {
        let mut sorted_tiles = activated_tiles.iter().sort::<&InGame>().peekable();

        for (game, phase) in activated_games.iter().sort::<Entity>() {
            if !matches!(phase, GamePhase::Act) {
                continue;
            }

            let Some((entity, position, direction, rotation, amplification, owner, tile, _)) =
                sorted_tiles.find(|(_, _, _, _, _, _, _, in_game)| ***in_game == game)
            else {
                info!(
                    "failed to find tiles for game {:?}! None exist or invalid sort(?)",
                    game
                );
                continue;
            };
            let parameters = TileParameters::new(position, direction, rotation, amplification);
            commands.add(tile.activate(entity, parameters, owner.and_then(|owner| Some(owner.0))));

            while sorted_tiles
                .peek()
                .is_some_and(|(_, _, _, _, _, _, _, in_game)| ***in_game == game)
            {
                let (entity, position, direction, rotation, amplification, owner, tile, _) =
                    sorted_tiles.next().unwrap();

                let parameters = TileParameters::new(position, direction, rotation, amplification);
                commands.add(tile.activate(
                    entity,
                    parameters,
                    owner.and_then(|owner| Some(owner.0)),
                ));
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
                    info!("Tile hit command added to queue");
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

        for tilemap_entities in &tilemaps {
            for tile_spawn in &tile_spawns {
                for (hex, tile_entity) in &tilemap_entities.tiles {
                    if *tile_entity == tile_spawn.on_tile {
                        commands.add(T::spawn(
                            Position::from(*hex),
                            tile_spawn.owner,
                            tile_spawn.game,
                        ));

                        commands.entity(*tile_entity).remove::<EmptyTile>();
                    }
                }
            }
        }
    }

    fn update_tile_material(
        added_tiles: Query<
            (&Position, Option<&Owner>, Option<&T>, Option<&EmptyTile>),
            Or<(Added<T>, Added<EmptyTile>)>,
        >,
        tilemaps: Query<&TilemapEntities, With<Tilemap>>,
        mut materials: Query<&mut Handle<ColorMaterial>>,
        mut material_assets: ResMut<Assets<ColorMaterial>>,
        players: Query<&PlayerColorAdjuster>,
        asset_server: Res<AssetServer>,
        empty_tile_material: Res<EmptyTileMaterial>,
    ) {
        let Ok(tiles) = tilemaps.get_single() else {
            return;
        };

        for (position, owner, tile, empty) in &added_tiles {
            let hex = **position;
            if let Some(mut material) = tiles
                .get(&hex)
                .and_then(|&entity| materials.get_mut(entity).ok())
            {
                let darkening = match owner.and_then(|owner| players.get(**owner).ok()) {
                    Some(value) => value,
                    None => &PlayerColorAdjuster(0.),
                };

                if let Some(_) = tile {
                    *material = material_assets.add(
                        T::material(&asset_server)
                            .color
                            .mix(&Color::BLACK, **darkening),
                    );
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
    pub owner: Entity,
    pub game: Entity,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Component, Deref)]
pub struct Owner(Entity);

impl Owner {
    pub fn new(entity: Entity) -> Owner {
        Self(entity)
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[derive(Component)]
pub struct TileParameters {
    pub position: Position,
    pub direction: Option<Direction>,
    pub rotation: Option<Rotation>,
    pub amplification: Option<Amplification>,
}

impl TileParameters {
    pub fn new(
        position: &Position,
        direction: Option<&Direction>,
        rotation: Option<&Rotation>,
        amplification: Option<&Amplification>,
    ) -> TileParameters {
        let direction = direction.and_then(|direction| Some(*direction));
        let rotation = rotation.and_then(|rotation| Some(*rotation));
        let amplification = amplification.and_then(|amplification| Some(*amplification));
        Self {
            position: *position,
            direction,
            rotation,
            amplification,
        }
    }
    pub fn from_position(position: &Position) -> TileParameters {
        Self {
            position: *position,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut)]
pub struct Territory(HashSet<Entity>);

impl Territory {
    pub const RANGE: usize = 4;
}

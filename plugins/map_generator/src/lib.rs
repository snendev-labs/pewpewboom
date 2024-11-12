use std::{
    any::TypeId,
    cmp::{max, min},
    collections::{HashMap, HashSet},
    iter::zip,
};

use bevy::{
    color::palettes,
    prelude::{Color, *},
};
use hexx::{shapes, Hex, HexLayout};
use noise::{utils::*, Fbm, MultiFractal, Perlin};
use rand::{seq::IteratorRandom, Rng};

use entropy::EntropyBundle;
use game_loop::{GameInstance, GamePlayers, MapSize};
use hq::HQTile;
use mountain::MountainTile;
use resource_deposit::ResourceDepositTile;
use tilemap::{
    EmptyTile, EmptyTileMaterial, Tile, TileBundle, Tilemap, TilemapEntities, TilemapLayout,
    TilemapPlugin,
};
use tiles::{lasers::Position, TileSpawnEvent};

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::spawn_map, Self::spawn_hq, Self::spawn_terrain)
                .chain()
                .in_set(MapGeneratorSystems),
        );
    }
}

impl MapGeneratorPlugin {
    pub const TILE_CUTOFF: f64 = 0.4;
    pub const OBSTACLE_CUTOFF: f64 = 0.7;

    fn spawn_map(
        mut games: Query<
            (Entity, &MapSize, &mut EntropyBundle),
            (With<GameInstance>, Without<Tilemap>),
        >,
        mut commands: Commands,
        empty_tile_material: Res<EmptyTileMaterial>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (game, size, mut entropy) in &mut games {
            let layout = HexLayout {
                hex_size: TilemapPlugin::HEX_SIZE,
                ..Default::default()
            };
            let tile_mesh = meshes.add(Tile::mesh(&layout));

            let (world_half_width, world_half_height) = size.to_world_size(&layout);

            let fbm = Fbm::<Perlin>::new(entropy.entropy.gen())
                .set_octaves(1)
                .set_frequency(1.);

            let noise_map = PlaneMapBuilder::new(fbm)
                .set_size(2 * size.half_width + 1, 2 * size.half_height + 1)
                .set_x_bounds(-world_half_width, world_half_width)
                .set_y_bounds(-world_half_height, world_half_height)
                .build();

            let mut tiles: HashMap<Hex, Entity> = HashMap::new();
            let mut obstacles: HashSet<Tile> = HashSet::new();

            for coord in shapes::flat_rectangle([
                -(size.half_width as i32),
                size.half_width as i32,
                -(size.half_height as i32),
                size.half_height as i32,
            ]) {
                let (x_index, y_index) = size.rectangle_index(&coord);
                let noise = 0.5 * noise_map.get_value(x_index, y_index) + 0.5;
                if noise < Self::TILE_CUTOFF {
                    continue;
                }

                let position = layout.hex_to_world_pos(coord);
                let hex_entity = commands
                    .spawn((
                        TileBundle::new(
                            Tile::from(coord),
                            position,
                            10.,
                            tile_mesh.clone(),
                            empty_tile_material.clone_weak(),
                        ),
                        EmptyTile,
                    ))
                    .with_children(|b| {
                        b.spawn(Text2dBundle {
                            text: Text::from_section(
                                format!("{},{}", coord.x, coord.y),
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::Srgba(palettes::css::LIGHT_SLATE_GRAY),
                                    ..Default::default()
                                },
                            ),
                            transform: Transform::from_xyz(10.0, 35.0, 10.0),
                            ..Default::default()
                        });
                    })
                    .set_parent(game)
                    .id();
                tiles.insert(coord, hex_entity);

                if noise > Self::OBSTACLE_CUTOFF {
                    // Perlin noise values for the generated map seem to be clamped between [-1, 1] but still need to example parameters
                    // to get good distribution to figure out cutoff point
                    obstacles.insert(Tile::from(coord));
                }
            }

            let tilemap_data = TilemapEntities { tiles };
            commands.entity(game).insert((
                Tilemap::bundle(),
                TilemapLayout::new(layout),
                tilemap_data,
                ObstacleMap(obstacles),
            ));
        }
    }

    // Switch this to spawn first before map tiles to ensure there are valid tiles surrounding
    // player spawn position
    fn spawn_hq(
        mut games: Query<
            (
                Entity,
                &MapSize,
                &GamePlayers,
                &TilemapEntities,
                &mut EntropyBundle,
            ),
            Added<Tilemap>,
        >,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
    ) {
        for (game_entity, size, players, tilemap, mut entropy) in &mut games {
            let mut hq_positions: Vec<&Hex> = Vec::new();

            while hq_positions.is_empty() {
                let candidate_spawns = tilemap
                    .iter()
                    .map(|(hex, _)| hex)
                    .choose_multiple(&mut entropy.entropy, players.len());

                let mut minimum_distance = u32::MAX;
                for i in 0..candidate_spawns.len() {
                    for j in i + 1..candidate_spawns.len() {
                        let pair_distance =
                            candidate_spawns[i].unsigned_distance_to(*candidate_spawns[j]);
                        minimum_distance = min(minimum_distance, pair_distance)
                    }
                }

                if minimum_distance as f32 >= max(size.half_width, size.half_height) as f32 {
                    hq_positions = candidate_spawns;
                }
            }

            for (player, hq_position) in zip(&(**players), hq_positions) {
                let Some(tile) = tilemap.tiles.get(hq_position) else {
                    info!("Spawn point for player HQ not found in available tilemap hexes");
                    return;
                };
                info!(
                    "Spawning hq for player {:?} on hex {:?}",
                    player, hq_position
                );
                tile_spawns.send(TileSpawnEvent {
                    tile_id: TypeId::of::<HQTile>(),
                    on_tile: *tile,
                    owner: *player,
                    game: game_entity,
                });
            }
        }
    }

    fn spawn_terrain(
        mut obstacles: Query<
            (
                Entity,
                &mut ObstacleMap,
                &TilemapEntities,
                &mut EntropyBundle,
            ),
            Added<ObstacleMap>,
        >,
        hqs: Query<&Position, Added<HQTile>>,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
    ) {
        let hq_positions = hqs.iter().map(|position| **position).collect::<Vec<_>>();

        for (game, mut obstacle_map, tilemap, mut entropy) in &mut obstacles {
            let too_close_obstacles = obstacle_map
                .iter()
                .filter(|tile| {
                    hq_positions
                        .iter()
                        .any(|hex| hex.unsigned_distance_to(***tile) < 6)
                })
                .cloned()
                .collect::<Vec<_>>();

            for tile in too_close_obstacles {
                obstacle_map.remove(&tile);
            }

            for tile in obstacle_map.iter() {
                let sample: f32 = entropy.entropy.gen_range(0.0..=1.0);
                let Some(tile_entity) = tilemap.tiles.get(&(**tile)) else {
                    info!("Tile for spawned obstacle not found within the tilemap");
                    return;
                };
                tile_spawns.send(TileSpawnEvent {
                    tile_id: if sample > 0.25 {
                        TypeId::of::<MountainTile>()
                    } else {
                        TypeId::of::<ResourceDepositTile>()
                    },
                    on_tile: *tile_entity,
                    owner: game,
                    game: game,
                });
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct MapGeneratorSystems;

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct ObstacleMap(HashSet<Tile>);

#[cfg(test)]
mod tests {
    use bevy::prelude::App;

    use entropy::EntropyPlugin;

    use super::MapGeneratorPlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(EntropyPlugin::default());
        app.add_plugins(MapGeneratorPlugin);

        app
    }

    // #[test]
    // fn test_hit() {
    //     let mut app = test_app();

    //     if let Some(mut entropy) = app.world_mut().get_resource_mut::<GlobalEntropy>() {
    //         let mut entropy_bundle = EntropyBundle::new(&mut entropy);
    //         let seeded_map = ObstacleMap::generate(0..=1, 0..=1, 2, 2, &mut entropy_bundle);

    //         assert!(seeded_map.contains(&Tile::new(0, 0)))
    //     }
    // }
}

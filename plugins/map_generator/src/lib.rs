use std::{any::TypeId, collections::HashSet};

use bevy::prelude::*;
use hexx::Hex;
use hq::HQTile;
use noise::{utils::*, Fbm, Perlin};

use entropy::EntropyBundle;
use game_loop::{GameInstance, GamePlayers, GameRadius};
use mountain::MountainTile;
use rand::{
    seq::{index::sample, SliceRandom},
    Rng,
};
use resource_deposit::ResourceDepositTile;
use tilemap::{Tile, TilemapEntities};
use tiles::{lasers::Position, TileSpawnEvent};

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::spawn_hq, Self::spawn_obstacles, Self::spawn_mountains)
                .chain()
                .in_set(MapGeneratorSystems),
        );
    }
}

impl MapGeneratorPlugin {
    fn spawn_hq(
        mut games: Query<
            (Entity, &GameRadius, &GamePlayers, &mut EntropyBundle),
            Added<GamePlayers>,
        >,
        tiles: Query<&TilemapEntities>,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
    ) {
        let Ok(tilemap) = tiles.get_single() else {
            info!("Multiple tilemaps found");
            return;
        };

        for (game_entity, game_radius, players, mut entropy) in &mut games {
            let mut candidate_spawns = (0..3)
                .flat_map(|border_distance| {
                    Hex::new(0, 0)
                        .ring(**game_radius - border_distance)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            for player in &(**players) {
                let index = sample(&mut entropy.entropy, candidate_spawns.len(), 1);

                let hex = candidate_spawns.remove(index.iter().next().unwrap());
                let Some(tile) = tilemap.tiles.get(&hex) else {
                    info!("Spawn point for player HQ not found in available tilemap hexes");
                    return;
                };
                info!("Spawning hq for player {:?} on hex {:?}", player, hex);
                tile_spawns.send(TileSpawnEvent {
                    tile_id: TypeId::of::<HQTile>(),
                    on_tile: *tile,
                    owner: *player,
                    game: game_entity,
                });
            }
        }
    }

    fn spawn_mountains(
        tiles: Query<(Entity, &Tile)>,
        mut obstacles: Query<(Entity, &ObstacleMap, &mut EntropyBundle), Added<ObstacleMap>>,
        hqs: Query<&Position, With<HQTile>>,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
    ) {
        for (game, obstacle_map, mut entropy) in &mut obstacles {
            let hq_positions = hqs.iter().map(|position| **position).collect::<Vec<_>>();
            for (tile_entity, tile) in &tiles {
                if obstacle_map.contains(tile) {
                    if hq_positions
                        .iter()
                        .any(|position| position.unsigned_distance_to(**tile) < 6)
                    {
                        continue;
                    }

                    let sample: f32 = entropy.entropy.gen_range(0.0..=1.0);

                    tile_spawns.send(TileSpawnEvent {
                        tile_id: if sample > 0.25 {
                            TypeId::of::<MountainTile>()
                        } else {
                            TypeId::of::<ResourceDepositTile>()
                        },
                        on_tile: tile_entity,
                        owner: game,
                        game: game,
                    });
                }
            }
        }
    }

    fn spawn_obstacles(
        games: Query<(Entity, &GameRadius), (With<GameInstance>, Without<ObstacleMap>)>,
        mut commands: Commands,
        mut entropy: Query<&mut EntropyBundle>,
    ) {
        for (game, radius) in &games {
            let map_radius = **radius;
            if let Ok(mut entropy) = entropy.get_mut(game) {
                info!("Adding in obstacle map for game");
                commands.entity(game).insert(ObstacleMap::generate(
                    map_radius as usize,
                    map_radius as usize,
                    &mut entropy,
                ));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct MapGeneratorSystems;

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Component, Deref)]
pub struct ObstacleMap(HashSet<Tile>);

impl ObstacleMap {
    pub fn generate(
        horizontal: usize,
        vertical: usize,
        // horizontal_samples: u32, // Should be less than the length of the respective sampling interval
        // vertical_samples: u32, // Should be less than the length of the respective sampling interval
        entropy: &mut EntropyBundle,
    ) -> Self {
        let fbm = Fbm::<Perlin>::new(entropy.entropy.gen());

        let noise_map = PlaneMapBuilder::new(fbm)
            .set_size(2 * horizontal, 2 * vertical)
            .set_x_bounds(-(horizontal as f64), horizontal as f64)
            .set_y_bounds(-(vertical as f64), vertical as f64)
            .build();

        let mut hit_tiles: HashSet<Tile> = HashSet::new();

        for x in 0..2 * horizontal + 1 {
            for y in 0..2 * vertical + 1 {
                let noise = noise_map.get_value(x, y);

                if noise < -0.1 {
                    // Perlin noise values for the generated map seem to be clamped between [-1, 1] but still need to example parameters
                    // to get good distribution to figure out cutoff point
                    hit_tiles.insert(Tile::new(
                        x as i32 - horizontal as i32,
                        y as i32 - vertical as i32,
                    ));
                }
            }
        }

        ObstacleMap(hit_tiles)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::App;

    use entropy::{EntropyBundle, EntropyPlugin, GlobalEntropy};

    use super::{MapGeneratorPlugin, ObstacleMap, Tile};

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

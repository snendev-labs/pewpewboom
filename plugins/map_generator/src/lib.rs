use std::{any::TypeId, collections::HashSet, ops::RangeInclusive};

use bevy::prelude::*;
use game_loop::{GameInstance, GameRadius};
use itertools::Itertools;

use entropy::{Entropy, EntropyBundle};
use mountain::MountainTile;
use rand::Rng;
use resource_deposit::ResourceDepositTile;
use tilemap::Tile;
use tiles::TileSpawnEvent;

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::spawn_obstacles, Self::spawn_mountains).in_set(MapGeneratorSystems),
        );
    }
}

impl MapGeneratorPlugin {
    fn spawn_mountains(
        tiles: Query<(Entity, &Tile)>,
        mut obstacles: Query<(Entity, &ObstacleMap, &mut EntropyBundle), Added<ObstacleMap>>,
        mut tile_spawns: EventWriter<TileSpawnEvent>,
    ) {
        for (game, obstacle_map, mut entropy) in &mut obstacles {
            info!("Found spawned obstacle map {:?}", obstacle_map);
            for (tile_entity, tile) in &tiles {
                let sample: f32 = entropy.entropy.gen_range(0.0..=1.0);
                if obstacle_map.contains(tile) {
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
            let map_radius = **radius as i32;
            if let Ok(mut entropy) = entropy.get_mut(game) {
                info!("Adding in obstacle map for game");
                commands.entity(game).insert(ObstacleMap::generate(
                    -map_radius..=map_radius,
                    -map_radius..=map_radius,
                    3,
                    3,
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
        horizontal_range: RangeInclusive<i32>,
        vertical_range: RangeInclusive<i32>,
        horizontal_samples: u32, // Should be less than the length of the respective sampling interval
        vertical_samples: u32, // Should be less than the length of the respective sampling interval
        entropy: &mut EntropyBundle,
    ) -> Self {
        let mut horizontal = entropy.sample_from_range(horizontal_range, horizontal_samples);
        horizontal.sort();

        let horizontal = horizontal
            .chunks_exact(2)
            .filter_map(|chunk| {
                if chunk[0] != chunk[1] {
                    Some([chunk[0], chunk[1]])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut vertical = entropy.sample_from_range(vertical_range, vertical_samples);
        vertical.sort();

        let vertical = vertical
            .chunks_exact(2)
            .filter_map(|chunk| {
                if chunk[0] != chunk[1] {
                    Some([chunk[0], chunk[1]])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut hit_tiles: HashSet<Tile> = HashSet::new();

        for x_interval in horizontal {
            for y_interval in &vertical {
                let product = (x_interval[0]..=x_interval[1])
                    .cartesian_product(y_interval[0]..=y_interval[1])
                    .map(|(x, y)| Tile::new(x, y));
                hit_tiles.extend(product)
            }
        }

        ObstacleMap(hit_tiles)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::App;

    use entropy::{EntropyBundle, EntropyPlugin, GlobalEntropy};

    use super::{MapGeneratorPlugin, ObstacleMap, Tile};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(EntropyPlugin::default());
        app.add_plugins(MapGeneratorPlugin);

        app
    }

    #[test]
    fn test_hit() {
        let mut app = test_app();

        if let Some(mut entropy) = app.world_mut().get_resource_mut::<GlobalEntropy>() {
            let mut entropy_bundle = EntropyBundle::new(&mut entropy);
            let seeded_map = ObstacleMap::generate(0..=1, 0..=1, 2, 2, &mut entropy_bundle);

            assert!(seeded_map.contains(&Tile::new(0, 0)))
        }
    }
}

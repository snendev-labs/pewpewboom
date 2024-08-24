use std::{collections::HashSet, ops::RangeInclusive};

use bevy::prelude::*;
use itertools::Itertools;
use rand::thread_rng;

use entropy::{Entropy, EntropyBundle};
use game_loop::SpawnGame;
use mountain::MountainTile;
use tilemap::Tile;

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (Self::spawn_mountains).in_set(MapGeneratorSystems));

        app.observe(Self::observer);
    }
}

impl MapGeneratorPlugin {
    fn spawn_mountains(
        mut commands: Commands,
        tiles: Query<(Entity, &Tile)>,
        obstacles: Query<&ObstacleMap>,
    ) {
        let obstacle_map = obstacles.single();
        for (entity, tile) in &tiles {
            if obstacle_map.contains(tile) {
                commands.entity(entity).insert(MountainTile);
            }
        }
    }

    fn observer(trigger: Trigger<SpawnGame>, mut commands: Commands, entropy: Query<&Entropy>) {
        let game_instance = trigger.event().instance;
        let map_radius = trigger.event().radius as i32;

        if let Some(entropy) = entropy.get(game_instance) {
            commands.entity(game_instance).insert(ObstacleMap::generate(
                [-map_radius, map_radius],
                [-map_radius, map_radius],
                5,
                5,
                entropy,
            ));
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
        horizontal_samples: usize, // Should be less than the length of the respective sampling interval
        vertical_samples: usize, // Should be less than the length of the respective sampling interval
        entropy: EntropyBundle,
    ) -> Self {
        let mut rng = thread_rng();

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
                if (chunk[0] != chunk[1]) {
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

    use entropy::{EntropyPlugin, GlobalEntropy};

    use super::{MapGeneratorPlugin, ObstacleMap, Tile};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(EntropyPlugin::default());
        app.add_plugins(MapGeneratorPlugin);

        app
    }

    #[test]
    fn test_generating() {
        let test_map = ObstacleMap::generate([0, 1], [0, 1], 2, 2);

        let map: HashSet<Tile> = vec![(0, 0), (0, 1), (1, 0), (1, 1)]
            .into_iter()
            .map(|(x, y)| Tile::new(x, y))
            .collect();
        let map = ObstacleMap(map);
        assert_eq!(test_map, map)
    }

    #[test]
    fn test_hit() {
        let app = test_app();
        if let Some(entropy) = app.world().get_resource_mut::<GlobalEntropy>() {
            let entropy_bundle = EntropyBundle::new(entropy);
            let seeded_map = ObstacleMap::generate([0, 1], [0, 1], 2, 2, entropy_bundle);

            assert!(test_map.contains(&Tile::new(0, 0)))
        }
    }
}

use std::collections::HashSet;

use bevy::prelude::*;
use itertools::Itertools;
use rand::thread_rng;

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

    fn observer(trigger: Trigger<SpawnGame>, mut commands: Commands) {
        let game_instance = trigger.event().instance;
        let map_radius = trigger.event().radius as i32;
        commands.entity(game_instance).insert(ObstacleMap::generate(
            [-map_radius, map_radius],
            [-map_radius, map_radius],
            5,
            5,
        ));
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
        horizontal_range: [i32; 2],
        vertical_range: [i32; 2],
        horizontal_samples: usize, // Should be less than the length of the respective sampling interval
        vertical_samples: usize, // Should be less than the length of the respective sampling interval
    ) -> Self {
        let mut rng = thread_rng();

        let mut horizontal = rand::seq::index::sample(
            &mut rng,
            (horizontal_range[1] - horizontal_range[0] + 1) // inclusive of the interval limits
                .try_into()
                .unwrap(),
            horizontal_samples,
        )
        .iter()
        .map(|num| num as i32 + horizontal_range[0])
        .collect::<Vec<i32>>();
        horizontal.sort();

        let horizontal = horizontal
            .chunks_exact(2)
            .map(|chunk| [chunk[0], chunk[1]])
            .collect::<Vec<_>>();

        let mut vertical = rand::seq::index::sample(
            &mut rng,
            (vertical_range[1] - vertical_range[0] + 1)
                .try_into()
                .unwrap(),
            vertical_samples,
        )
        .iter()
        .map(|num| num as i32 + vertical_range[0])
        .collect::<Vec<i32>>();
        vertical.sort();

        let vertical = vertical
            .chunks_exact(2)
            .map(|chunk| [chunk[0], chunk[1]])
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

    use super::{ObstacleMap, Tile};

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
        let test_map = ObstacleMap::generate([0, 1], [0, 1], 2, 2);

        assert!(test_map.contains(&Tile::new(0, 0)))
    }
}

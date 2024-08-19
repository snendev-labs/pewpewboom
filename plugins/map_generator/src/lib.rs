use bevy::prelude::*;

use mountain::MountainTile;
use rand::thread_rng;

use tilemap::Tile;

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EntropyMap::generate([-10, 10], [-10, 10], 6, 6))
            .add_systems(Update, (Self::spawn_mountains).in_set(MapGeneratorSystems));
    }
}

impl MapGeneratorPlugin {
    fn spawn_mountains(
        mut commands: Commands,
        tiles: Query<(Entity, &Tile)>,
        entropy: Res<EntropyMap>,
    ) {
        for (entity, tile) in &tiles {
            if entropy.hit(*tile) {
                commands.entity(entity).insert(MountainTile);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct MapGeneratorSystems;

#[derive(Clone, Debug)]
#[derive(Resource)]
pub struct EntropyMap {
    horizontal: Vec<[i32; 2]>,
    vertical: Vec<[i32; 2]>,
}

impl EntropyMap {
    pub fn new(horizontal: Vec<[i32; 2]>, vertical: Vec<[i32; 2]>) -> Self {
        EntropyMap {
            horizontal,
            vertical,
        }
    }

    pub fn generate(
        horizontal_range: [i32; 2],
        vertical_range: [i32; 2],
        horizontal_samples: usize, // Should be less than the length of the respective sampling interval
        vertical_samples: usize, // Should be less than the length of the respective sampling interval
    ) -> Self {
        let mut rng = thread_rng();

        let mut horizontal = rand::seq::index::sample(
            &mut rng,
            (horizontal_range[1] - horizontal_range[0])
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
            (vertical_range[1] - vertical_range[0]).try_into().unwrap(),
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

        EntropyMap {
            horizontal,
            vertical,
        }
    }

    pub fn hit(&self, tile: Tile) -> bool {
        self.horizontal
            .iter()
            .any(|[lower, upper]| *lower <= tile.x && tile.x <= *upper)
            && self
                .vertical
                .iter()
                .any(|[lower, upper]| *lower <= tile.y && tile.y <= *upper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_entropy() {
        let entropy_map = EntropyMap::generate([0, 10], [0, 10], 5, 5);
        let horizontal = entropy_map
            .horizontal
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let vertical = entropy_map
            .vertical
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let mut sorted_horizontal = horizontal.clone();
        sorted_horizontal.sort();
        let mut sorted_vertical = vertical.clone();
        sorted_vertical.sort();
        assert!(horizontal == sorted_horizontal && vertical == sorted_vertical)
    }

    #[test]
    fn filled_entropy_map() {
        let entropy_map = EntropyMap::generate([0, 6], [0, 6], 6, 6);
        eprintln!("{:?}", entropy_map.horizontal);
        eprintln!("{:?}", entropy_map.vertical);
        assert_eq!(
            (entropy_map.horizontal, entropy_map.vertical),
            (vec![[0, 1], [2, 3], [4, 5]], vec![[0, 1], [2, 3], [4, 5]])
        )
    }

    #[test]
    fn hit_tile() {
        let horizontal = vec![[0, 2]];
        let vertical = vec![[0, 3]];
        let entropy_map = EntropyMap::new(horizontal, vertical);

        assert!(entropy_map.hit(Tile::new(1, 1)))
    }
}

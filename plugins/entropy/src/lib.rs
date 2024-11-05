use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::{EntropyPlugin as RandEntropyPlugin, *};
use rand::Rng;

pub use bevy_prng;
pub use bevy_rand;
pub use bevy_rand::prelude::ForkableRng;
pub use rand_core::RngCore;

#[derive(Clone, Copy, Default)]
pub struct EntropyPlugin {
    seed: [u8; 8],
}

impl Plugin for EntropyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RandEntropyPlugin::<WyRand>::with_seed(self.seed));
    }
}

pub type Entropy = EntropyComponent<WyRand>;
pub type GlobalEntropy = bevy_rand::prelude::GlobalEntropy<WyRand>;

#[derive(Clone, Debug, Default)]
#[derive(Component)]
pub struct EntropyBundle {
    pub entropy: Entropy,
}
// Should this be component or bundle with entropy component....

impl EntropyBundle {
    pub fn new(global: &mut GlobalEntropy) -> Self {
        Self {
            entropy: global.fork_rng(),
        }
    }

    pub fn sample_from_range(&mut self, range: RangeInclusive<i32>, samples: u32) -> Vec<i32> {
        (0..samples)
            .map(|_| self.entropy.gen_range(range.clone()))
            .collect()
    }
}

use bevy::{app::PluginGroupBuilder, prelude::*};

pub use game_loop;
pub use health;
pub use hq;
pub use laser_tower;
pub use merchandise;
pub use tilemap;
pub use tiles;

use game_loop::GameLoopPlugin;
use health::HealthPlugin;
use hq::HQPlugin;
use laser_tower::LaserTowerPlugin;
use merchandise::MerchPlugin;
use tilemap::TilemapPlugin;
use tiles::TilesPlugin;

pub struct PewPewBoomPlugins;

impl PluginGroup for PewPewBoomPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GameLoopPlugin)
            .add(TilesPlugin)
            .add(MerchPlugin)
            .add(HealthPlugin)
            .add(HQPlugin)
            .add(LaserTowerPlugin)
            .add(TilemapPlugin)
    }
}

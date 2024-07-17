use bevy::{app::PluginGroupBuilder, prelude::*};

pub use camera;
pub use game_loop;
pub use health;
pub use hq;
pub use laser_tower;
pub use merchandise;
pub use tilemap;
pub use tiles;

pub struct PewPewBoomPlugins;

impl PluginGroup for PewPewBoomPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(game_loop::GameLoopPlugin)
            .add(tiles::TilesPlugin)
            .add(merchandise::MerchPlugin)
            .add(health::HealthPlugin)
            .add(hq::HQPlugin)
            .add(laser_tower::LaserTowerPlugin)
            .add(tilemap::TilemapPlugin)
            .add(camera::CameraPlugin)
    }
}

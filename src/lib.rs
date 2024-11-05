use bevy::{app::PluginGroupBuilder, prelude::*};

pub use amplifier;
pub use camera;
pub use entropy;
pub use game_loop;
pub use health;
pub use hq;
pub use laser_tower;
pub use laser_visuals;
pub use map_generator;
pub use merchandise;
pub use refractor;
pub use tilemap;
pub use tiles;

pub struct PewPewBoomPlugins;

impl PluginGroup for PewPewBoomPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(entropy::EntropyPlugin::default())
            .add(game_loop::GameLoopPlugin)
            .add(tiles::TilesPlugin)
            .add(map_generator::MapGeneratorPlugin)
            .add(merchandise::MerchPlugin)
            .add(mountain::MountainPlugin)
            .add(resource_deposit::ResourceDepositPlugin)
            .add(health::HealthPlugin)
            .add(tilemap::TilemapPlugin)
            .add(camera::CameraPlugin)
            .add(shop::ShopPlugin)
            .add(laser_visuals::LaserVisualPlugin)
    }
}

pub struct PewPewBoomBuildingsPlugins;

impl PluginGroup for PewPewBoomBuildingsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(hq::HQPlugin)
            .add(laser_tower::LaserTowerPlugin)
            .add(amplifier::AmplifierPlugin)
            .add(refractor::RefractorPlugin)
            .add(reflector::ReflectorPlugin)
            .add(rotater::RotaterPlugin)
    }
}

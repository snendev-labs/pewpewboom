use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use tilemap::TargetedTile;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin);
        app.add_systems(
            Update,
            Self::change_targeted_tile_sound.run_if(resource_exists_and_changed::<TargetedTile>),
        )
        .observe(SingleSound::purchase_sound);
    }
}

impl SoundPlugin {
    fn change_targeted_tile_sound(targeted_tile: Res<TargetedTile>) {}

    fn purchase_sound(trigger: Trigger<SingleSound>) {}

    fn resource_hit_sound() {}

    fn mountain_hit_sound() {}

    fn building_hit_sound() {}

    fn tile_destroyed_sound() {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct SoundSystems;

#[derive(Event)]
pub enum SingleSound {
    SuccessfulPurchase,
    ResourceHit,
    MountainHit,
    BuildingHit,
    TileDestroyed,
}

impl SingleSound {
    fn purchase_sound(trigger: Trigger<SingleSound>) {}

    fn resource_hit_sound() {}

    fn mountain_hit_sound() {}

    fn building_hit_sound() {}

    fn tile_destroyed_sound() {}
}

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
        );
    }
}

impl SoundPlugin {
    fn change_targeted_tile_sound(targeted_tile: Res<TargetedTile>) {}

    fn purchase_sound() {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct SoundSystems;

use bevy::prelude::*;
use bevy_rts_camera::*;

use tilemap_3d::Tilemap;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::attach_camera_components,
                Self::remove_camera_components,
                Self::attach_ground_components,
            )
                .in_set(CameraSystems),
        );
    }
}

impl CameraPlugin {
    fn attach_camera_components(
        mut commands: Commands,
        new_cameras: Query<Entity, (With<PlayerCamera>, Without<RtsCamera>)>,
    ) {
        for camera in &new_cameras {
            commands.entity(camera).insert(PlayerCameraBundle::new());
        }
    }

    fn remove_camera_components(
        mut commands: Commands,
        mut removed_cameras: RemovedComponents<PlayerCamera>,
    ) {
        for camera in removed_cameras.read() {
            if let Some(mut entity) = commands.get_entity(camera) {
                entity.remove::<PlayerCameraBundle>();
            }
        }
    }

    fn attach_ground_components(
        mut commands: Commands,
        tilemaps: Query<Entity, (With<Tilemap>, Without<Ground>)>,
    ) {
        for tilemap_entity in &tilemaps {
            commands.entity(tilemap_entity).insert(Ground);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct CameraSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct PlayerCamera;

#[derive(Bundle)]
pub struct PlayerCameraBundle {
    rts: RtsCamera,
    controls: RtsCameraControls,
    camera: Camera3dBundle,
}

impl PlayerCameraBundle {
    fn new() -> Self {
        PlayerCameraBundle {
            rts: RtsCamera::default(),
            controls: RtsCameraControls::default(),
            camera: Camera3dBundle {
                transform: Transform::from_xyz(0., 20., 10.),
                ..Default::default()
            },
        }
    }
}

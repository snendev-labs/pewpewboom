use std::f32::consts::PI;

use bevy::{
    color::palettes,
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use game_loop::InGame;
use merchandise::{MerchAppExt, Merchandise, Money};
use shop::JustPurchased;
use tilemap::TilemapLayout;
use tiles::{
    lasers::{Position, Rotation},
    Owner, Tile, TileParameters, TilePlugin,
};

pub struct RotaterPlugin;

impl Plugin for RotaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin::<RotaterTile>::default())
            .add_systems(Update, Self::update_marker);
        app.define_merchandise::<RotaterTile>();
    }
}

impl RotaterPlugin {
    fn update_marker(
        tiles: Query<&Rotation, (With<RotaterTile>, Changed<Rotation>)>,
        mut markers: Query<(&Parent, &mut Transform, &mut Mesh2dHandle), With<RotaterMarker>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (parent, mut transform, mut mesh) in &mut markers {
            if let Ok(rotation) = tiles.get(**parent).and_then(|rotation| Ok(**rotation % 6)) {
                (*transform, *mesh) = match rotation {
                    0 => (
                        Transform::from_rotation(Quat::from_rotation_z(-PI / 3.)),
                        meshes.add(CircularSector::new(40., 0.)).into(),
                    ),
                    1 => (
                        Transform::from_rotation(Quat::from_rotation_z(-PI / 3.)),
                        meshes.add(CircularSector::new(40., PI / 6.)).into(),
                    ),
                    2 => (
                        Transform::from_rotation(Quat::from_rotation_z(-PI / 6.)),
                        meshes.add(CircularSector::new(40., PI / 3.)).into(),
                    ),
                    3 => (
                        Transform::IDENTITY,
                        meshes.add(CircularSector::new(40., PI / 2.)).into(),
                    ),
                    4 => (
                        Transform::from_rotation(Quat::from_rotation_z(PI / 6.)),
                        meshes.add(CircularSector::new(40., 2. * PI / 3.)).into(),
                    ),
                    5 => (
                        Transform::from_rotation(Quat::from_rotation_z(PI / 3.)),
                        meshes.add(CircularSector::new(40., 5. * PI / 6.)).into(),
                    ),
                    _ => unreachable!(
                        "All modulus cases for possible rotations should be covered already"
                    ),
                };
                info!("Changed rotation marker to match parameters");
            }
        }
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct RotaterTile;

impl Tile for RotaterTile {
    fn spawn(position: Position, player: Entity) -> impl Command {
        RotaterSpawn { position, player }
    }

    fn material(_asset_server: &AssetServer) -> ColorMaterial {
        Color::Srgba(palettes::css::CADET_BLUE).into()
    }

    fn activate(
        &self,
        _entity: Entity,
        parameters: TileParameters,
        _shooter: Option<Entity>,
    ) -> impl Command {
        RotaterActivate {
            position: parameters.position,
            rotation: parameters
                .rotation
                .unwrap_or_else(|| panic!("Rotator needs a rotation")),
        }
    }
}

impl Merchandise for RotaterTile {
    const PRICE: Money = Money::new(5);
    const NAME: &'static str = "Rotater Tower";

    fn material(asset_server: &AssetServer) -> ColorMaterial {
        let mut base = <Self as Tile>::material(asset_server);
        base.color.set_alpha(0.6);
        base
    }
}

pub struct RotaterSpawn {
    position: Position,
    player: Entity,
}

impl Command for RotaterSpawn {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<(
            Query<&TilemapLayout>,
            ResMut<Assets<Mesh>>,
            ResMut<Assets<ColorMaterial>>,
        )> = SystemState::new(world);

        let (layout, mut meshes, mut materials) = system_state.get_mut(world);

        let Ok(translation) = layout
            .get_single()
            .and_then(|layout| Ok(layout.hex_to_world_pos(*self.position).extend(11.)))
        else {
            info!("Did not get the single tilemap layout for the game");
            return;
        };

        let sector = meshes.add(CircularSector::new(40., PI / 6.));
        let white = materials.add(Color::WHITE);

        if let Some(game) = world.get::<InGame>(self.player) {
            world
                .spawn((
                    RotaterTile,
                    self.position,
                    Rotation::new(1),
                    Owner::new(self.player),
                    game.clone(),
                    Transform::from_translation(translation),
                    GlobalTransform::from_translation(translation),
                    JustPurchased,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        RotaterMarker,
                        MaterialMesh2dBundle {
                            mesh: sector.into(),
                            material: white,
                            transform: Transform::from_rotation(Quat::from_rotation_z(-PI / 3.)),
                            ..default()
                        },
                    ));
                });
        }
    }
}

pub struct RotaterActivate {
    position: Position,
    rotation: Rotation,
}

impl Command for RotaterActivate {
    fn apply(self, world: &mut World) {
        world.spawn((Rotation::new(self.rotation.get()), self.position));
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component)]
pub struct RotaterMarker;

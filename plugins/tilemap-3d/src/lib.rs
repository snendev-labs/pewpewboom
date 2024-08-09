use std::collections::HashMap;

use bevy::{
    color::Color,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        Assets, BuildChildren, Bundle, Camera, Commands, Component, Deref, DerefMut, Entity,
        GlobalTransform, Handle, IntoSystemConfigs, Mesh, Name, Plugin, Query, Reflect, Res,
        ResMut, Resource, SpatialBundle, Startup, SystemSet, Transform, Update, Vec2, Window, With,
        Without,
    },
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    window::PrimaryWindow,
};
use hexx::{shapes, ColumnMeshBuilder, Hex, HexLayout};

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, EmptyTileMaterial::startup_system)
            .add_systems(
                Update,
                (Self::spawn_tilemap, Self::handle_cursor_position).chain(),
            );
    }
}

impl TilemapPlugin {
    const HEX_SIZE: Vec2 = Vec2::splat(60.);

    fn spawn_tilemap(
        mut commands: Commands,
        tilemaps: Query<Entity, (With<Tilemap>, Without<TilemapEntities>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        empty_tile_material: Res<EmptyTileMaterial>,
    ) {
        for map_entity in &tilemaps {
            let layout = HexLayout {
                hex_size: TilemapPlugin::HEX_SIZE,
                ..Default::default()
            };
            let tile_mesh = meshes.add(Tile::mesh(&layout));

            let mut tiles = HashMap::default();
            for hex_coordinate in shapes::Hexagon::default().coords() {
                let position = layout.hex_to_world_pos(hex_coordinate);
                let hex_entity = commands
                    .spawn(TileBundle::new(
                        Tile(hex_coordinate),
                        position,
                        tile_mesh.clone(),
                        empty_tile_material.clone_weak(),
                    ))
                    .set_parent(map_entity)
                    .id();
                tiles.insert(hex_coordinate, hex_entity);
            }

            let tilemap_data = TilemapEntities { tiles };
            commands
                .entity(map_entity)
                .insert((TilemapLayout(layout), tilemap_data));
        }
    }

    fn handle_cursor_position(
        mut commands: Commands,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform)>,
        tilemaps: Query<(Entity, &TilemapLayout, &TilemapEntities)>,
        targeted_tile: Option<ResMut<TargetedTile>>,
    ) {
        let Ok(window) = windows.get_single() else {
            return;
        };
        let Ok((camera, camera_transform)) = cameras.get_single() else {
            return;
        };
        let Ok((tilemap, layout, tiles)) = tilemaps.get_single() else {
            return;
        };
        let Some(position) = window
            .cursor_position()
            .and_then(|position| camera.viewport_to_world_2d(camera_transform, position))
        else {
            return;
        };

        // convert to hex and back to "snap" to the hex border
        let coord: Hex = layout.world_pos_to_hex(position);
        if let Some(hovered_tile) = tiles.get(&coord).copied() {
            if let Some(mut targeted_tile) = targeted_tile {
                targeted_tile.tile = hovered_tile;
                targeted_tile.tilemap = tilemap;
            } else {
                commands.insert_resource(TargetedTile {
                    tile: hovered_tile,
                    tilemap,
                });
            }
        } else if targeted_tile.is_some() {
            commands.remove_resource::<TargetedTile>();
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[derive(SystemSet)]
pub struct TilemapSystems;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct Tilemap;

impl Tilemap {
    pub fn bundle() -> impl Bundle {
        (Tilemap, Name::new("Tilemap Root"), SpatialBundle::default())
    }
}

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TilemapLayout(HexLayout);

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TilemapCursor(Entity);

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TilemapEntities {
    tiles: HashMap<Hex, Entity>,
}

#[derive(Deref, Resource)]
pub struct EmptyTileMaterial(Handle<StandardMaterial>);

impl EmptyTileMaterial {
    fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        EmptyTileMaterial(materials.add(Color::WHITE))
    }

    fn startup_system(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
        commands.insert_resource(Self::new(materials.as_mut()));
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Tile(Hex);

impl Tile {
    const COLUMN_HEIGHT: f32 = 5.;

    fn mesh(hex_layout: &HexLayout) -> Mesh {
        let mesh_info = ColumnMeshBuilder::new(hex_layout, Self::COLUMN_HEIGHT)
            .without_bottom_face()
            .center_aligned()
            .build();

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_inserted_indices(Indices::U16(mesh_info.indices))
    }
}

#[derive(Bundle)]
struct TileBundle<T: Component> {
    tile: T,
    mesh: PbrBundle,
}

impl<T: Component> TileBundle<T> {
    fn new(
        tile: T,
        position: Vec2,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Self {
        Self {
            tile,
            mesh: PbrBundle {
                mesh: mesh.into(),
                material,
                transform: Transform::from_xyz(position.x, 0., position.y),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, PartialEq)]
#[derive(Resource)]
pub struct TargetedTile {
    pub tile: Entity,
    pub tilemap: Entity,
}

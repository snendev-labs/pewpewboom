use bevy::color::palettes;
use bevy::prelude::{
    resource_added, resource_exists_and_changed, resource_removed, App, Assets, BuildChildren,
    Bundle, Camera, Color, ColorMaterial, ColorMesh2dBundle, Commands, Component, Deref, DerefMut,
    DespawnRecursiveExt, Entity, GlobalTransform, Handle, IntoSystemConfigs, Mesh, Name, Plugin,
    Query, Reflect, Res, ResMut, Resource, SpatialBundle, SystemSet, Text, Text2dBundle, TextStyle,
    Transform, Update, Vec3Swizzles, Window, With, Without,
};
use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;

use hexx::{shapes, *};

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::spawn_tilemaps,
                Self::destroy_targeted_tile.run_if(resource_removed::<TargetedTile>()),
                Self::update_targeted_tile.run_if(resource_exists_and_changed::<TargetedTile>),
                Self::spawn_targeted_tile.run_if(resource_added::<TargetedTile>),
                Self::handle_cursor_position,
            )
                .chain()
                .in_set(TilemapSystems),
        );
    }
}

impl TilemapPlugin {
    /// World size of the hexagons (outer radius)
    const HEX_SIZE: Vec2 = Vec2::splat(30.0);

    fn spawn_tilemaps(
        mut commands: Commands,
        tilemaps: Query<Entity, (With<Tilemap>, Without<TilemapData>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for map_entity in &tilemaps {
            let layout = HexLayout {
                hex_size: TilemapPlugin::HEX_SIZE,
                ..Default::default()
            };
            let tile_mesh = meshes.add(Tile::mesh(&layout));
            let tile_material = materials.add(Tile::material());

            let mut tiles = HashMap::default();
            for coord in shapes::Hexagon::default().coords() {
                let position = layout.hex_to_world_pos(coord);
                let hex_entity = commands
                    .spawn(TileBundle::new(
                        Tile(coord),
                        position,
                        10.,
                        tile_mesh.clone(),
                        tile_material.clone_weak(),
                    ))
                    .with_children(|b| {
                        b.spawn(Text2dBundle {
                            text: Text::from_section(
                                format!("{},{}", coord.x, coord.y),
                                TextStyle {
                                    font_size: 7.0,
                                    color: Color::BLACK,
                                    ..Default::default()
                                },
                            ),
                            transform: Transform::from_xyz(0.0, 0.0, 10.0),
                            ..Default::default()
                        });
                    })
                    .set_parent(map_entity)
                    .id();
                tiles.insert(coord, hex_entity);
            }
            let tilemap_data = TilemapData {
                tile_material: tile_material.clone(),
                tiles,
            };
            commands
                .entity(map_entity)
                .insert((TilemapLayout(layout), tilemap_data));
        }
    }

    fn handle_cursor_position(
        mut commands: Commands,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform)>,
        tilemaps: Query<(Entity, &TilemapLayout, &TilemapData)>,
        targeted_tile: Option<ResMut<TargetedTile>>,
    ) {
        let window = windows.single();
        let (camera, camera_transform) = cameras.single();
        let Ok((tilemap, layout, data)) = tilemaps.get_single() else {
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
        if let Some(hovered_tile) = data.tiles.get(&coord).copied() {
            if let Some(mut targeted_tile) = targeted_tile {
                targeted_tile.tile = hovered_tile;
                targeted_tile.tilemap = tilemap;
            } else {
                commands.insert_resource(TargetedTile {
                    tile: hovered_tile,
                    tilemap,
                });
            }
        } else {
            if targeted_tile.is_some() {
                commands.remove_resource::<TargetedTile>();
            }
        }
    }

    fn spawn_targeted_tile(
        mut commands: Commands,
        tilemaps: Query<(Entity, &TilemapLayout), With<Tilemap>>,
        tiles: Query<&Transform, With<Tile>>,
        targeted_tile: Res<TargetedTile>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        let Ok((entity, layout)) = tilemaps.get(targeted_tile.tilemap) else {
            return;
        };
        let Ok(position) = tiles.get(targeted_tile.tile) else {
            return;
        };
        let cursor_mesh = meshes.add(CursorHex::mesh(&layout));
        let cursor_material = materials.add(CursorHex::material());
        let cursor = commands
            .spawn(TileBundle::new(
                CursorHex,
                position.translation.xy(),
                20.,
                cursor_mesh,
                cursor_material,
            ))
            .id();
        commands.entity(entity).insert(TilemapCursor(cursor));
    }

    fn update_targeted_tile(
        mut commands: Commands,
        mut cursors: Query<(Entity, &mut Transform), With<CursorHex>>,
        tilemaps: Query<
            (Entity, &TilemapCursor),
            (With<Tilemap>, Without<Tile>, Without<CursorHex>),
        >,
        tiles: Query<&Transform, (With<Tile>, Without<Tilemap>, Without<CursorHex>)>,
        targeted_tile: Res<TargetedTile>,
    ) {
        // first, check that no old cursors exist for previously-targeted tilemaps
        for (tilemap, cursor) in &tilemaps {
            if tilemap != targeted_tile.tilemap {
                if let Ok((cursor, _)) = cursors.get(**cursor) {
                    commands.entity(cursor).despawn_recursive();
                }
                commands.entity(tilemap).remove::<TilemapCursor>();
            }
        }
        // get the cursor_hex transform
        let Ok((_, cursor)) = tilemaps.get(targeted_tile.tilemap) else {
            return;
        };
        let Ok((_, mut cursor)) = cursors.get_mut(**cursor) else {
            return;
        };
        // get the targeted tile
        let Ok(tile) = tiles.get(targeted_tile.tile) else {
            return;
        };
        // snap the cursor_hex to the targeted tile
        cursor.translation.x = tile.translation.x;
        cursor.translation.y = tile.translation.y;
    }

    fn destroy_targeted_tile(
        mut commands: Commands,
        tilemaps: Query<(Entity, &TilemapCursor), With<Tilemap>>,
    ) {
        for (tilemap, cursor) in &tilemaps {
            commands.entity(tilemap).remove::<TilemapCursor>();
            commands.entity(**cursor).despawn_recursive();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(Component, Reflect)]
pub struct TilemapData {
    tiles: HashMap<Hex, Entity>,
    tile_material: Handle<ColorMaterial>,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Tile(Hex);

impl Tile {
    fn mesh(hex_layout: &HexLayout) -> Mesh {
        let mesh_info = PlaneMeshBuilder::new(hex_layout)
            .facing(Vec3::Z)
            .with_scale(Vec3::splat(0.98))
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

    pub fn material() -> impl Into<ColorMaterial> {
        Color::WHITE
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CursorHex;

impl CursorHex {
    fn mesh(hex_layout: &HexLayout) -> Mesh {
        let mesh_info = PlaneMeshBuilder::new(hex_layout)
            .facing(Vec3::Z)
            .with_inset_options(InsetOptions {
                keep_inner_face: false,
                scale: 0.2,
                ..Default::default()
            })
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

    fn material() -> impl Into<ColorMaterial> {
        Color::Srgba(palettes::css::RED)
    }
}

#[derive(Bundle)]
struct TileBundle<T: Component> {
    tile: T,
    mesh: ColorMesh2dBundle,
}

impl<T: Component> TileBundle<T> {
    fn new(
        tile: T,
        position: Vec2,
        z: f32,
        mesh: Handle<Mesh>,
        material: Handle<ColorMaterial>,
    ) -> Self {
        Self {
            tile,
            mesh: ColorMesh2dBundle {
                mesh: mesh.into(),
                material,
                transform: Transform::from_xyz(position.x, position.y, z),
                ..Default::default()
            },
        }
    }
}

#[derive(Resource)]
pub struct TargetedTile {
    pub tile: Entity,
    pub tilemap: Entity,
}

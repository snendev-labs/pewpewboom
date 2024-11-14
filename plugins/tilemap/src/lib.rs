use std::collections::HashMap;
use std::f32::consts::PI;

use bevy::color::palettes;
use bevy::log::info;
use bevy::prelude::{
    resource_added, resource_exists_and_changed, resource_removed, App, Assets, BuildChildren,
    Bundle, Camera, Color, ColorMaterial, ColorMesh2dBundle, Commands, Component, Deref, DerefMut,
    DespawnRecursiveExt, Entity, GlobalTransform, Handle, IntoSystemConfigs, Mesh, Name, Plugin,
    Query, Reflect, Res, ResMut, Resource, SpatialBundle, Startup, SystemSet, Text, Text2dBundle,
    TextStyle, Transform, Update, Vec3Swizzles, Window, With, Without,
};
use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

use bevy::window::PrimaryWindow;

use hexx::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                EmptyTileMaterial::startup_system,
                TerritoryTileMaterial::startup_system,
            ),
        )
        .add_systems(
            Update,
            (
                Self::destroy_targeted_tile.run_if(resource_removed::<TargetedTile>()),
                Self::update_targeted_tile.run_if(resource_exists_and_changed::<TargetedTile>),
                Self::spawn_targeted_tile.run_if(resource_added::<TargetedTile>),
                Self::handle_cursor_position,
                Self::update_cursor_directions,
            )
                .chain()
                .in_set(TilemapSystems),
        );
    }
}

impl TilemapPlugin {
    /// World size of the hexagons (outer radius)
    pub const HEX_SIZE: Vec2 = Vec2::splat(60.0);

    fn handle_cursor_position(
        mut commands: Commands,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&Camera, &GlobalTransform)>,
        tilemaps: Query<(Entity, &TilemapLayout, &TilemapEntities)>,
        targeted_tile: Option<ResMut<TargetedTile>>,
        cursor_position: Option<ResMut<CursorWorldPosition>>,
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

        if let Some(mut cursor_position) = cursor_position {
            **cursor_position = position
        } else {
            commands.insert_resource(CursorWorldPosition(position))
        }

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

    fn update_cursor_directions(
        mut commands: Commands,
        mut tiles: Query<(Entity, &Tile, Option<&mut CursorDirection>)>,
        tilemaps: Query<&TilemapLayout, With<Tilemap>>,
        cameras: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window, With<PrimaryWindow>>,
    ) {
        let Ok(window) = windows.get_single() else {
            return;
        };
        let Ok((camera, camera_transform)) = cameras.get_single() else {
            return;
        };
        let Ok(layout) = tilemaps.get_single() else {
            return;
        };
        let Some(position) = window
            .cursor_position()
            .and_then(|position| camera.viewport_to_world_2d(camera_transform, position))
        else {
            return;
        };

        for (tile_entity, tile, cursor_direction) in &mut tiles {
            let tile_position = layout.hex_to_world_pos(**tile);
            let current_direction = match (position - tile_position).to_angle() {
                theta if theta < PI / 3. && theta >= 0. => {
                    CursorDirection(EdgeDirection::FLAT_NORTH_EAST)
                }
                theta if theta >= PI / 3. && theta < 2. * PI / 3. => {
                    CursorDirection(EdgeDirection::FLAT_NORTH)
                }
                theta if theta >= 2. * PI / 3. && theta <= PI => {
                    CursorDirection(EdgeDirection::FLAT_NORTH_WEST)
                }
                theta if theta >= -PI && theta < -2. * PI / 3. => {
                    CursorDirection(EdgeDirection::FLAT_SOUTH_WEST)
                }
                theta if theta >= -2. * PI / 3. && theta < -PI / 3. => {
                    CursorDirection(EdgeDirection::FLAT_SOUTH)
                }
                _ => CursorDirection(EdgeDirection::FLAT_SOUTH_EAST),
            };
            if let Some(mut direction) = cursor_direction {
                *direction = current_direction
            } else {
                commands.entity(tile_entity).insert(current_direction);
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
        let cursor_mesh = meshes.add(CursorHex::mesh(layout));
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

    #[allow(clippy::type_complexity)]
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

impl TilemapLayout {
    pub fn new(layout: HexLayout) -> TilemapLayout {
        Self(layout)
    }
}

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TilemapCursor(Entity);

#[derive(Clone, Debug)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct TilemapEntities {
    pub tiles: HashMap<Hex, Entity>,
}

#[derive(Clone, Copy, Debug)]
#[derive(Component)]
pub struct EmptyTile;

#[derive(Deref, Resource)]
pub struct EmptyTileMaterial(Handle<ColorMaterial>);

impl EmptyTileMaterial {
    fn new(materials: &mut Assets<ColorMaterial>) -> Self {
        EmptyTileMaterial(materials.add(Color::WHITE))
    }

    fn startup_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands.insert_resource(Self::new(materials.as_mut()));
    }
}

#[derive(Deref, Resource)]
pub struct TerritoryTileMaterial(Handle<ColorMaterial>);

impl TerritoryTileMaterial {
    fn new(materials: &mut Assets<ColorMaterial>) -> TerritoryTileMaterial {
        Self(materials.add(Color::from(bevy::color::palettes::basic::SILVER)))
    }

    fn startup_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands.insert_resource(Self::new(materials.as_mut()));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Tile(Hex);

impl Tile {
    pub fn new(x: i32, y: i32) -> Tile {
        Self(Hex { x, y })
    }

    pub fn mesh(hex_layout: &HexLayout) -> Mesh {
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
}

impl From<Hex> for Tile {
    fn from(value: Hex) -> Tile {
        Self(value)
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
pub struct TileBundle<T: Component> {
    tile: T,
    mesh: ColorMesh2dBundle,
}

impl<T: Component> TileBundle<T> {
    pub fn new(
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

#[derive(Clone, PartialEq)]
#[derive(Resource)]
pub struct TargetedTile {
    pub tile: Entity,
    pub tilemap: Entity,
}

#[derive(Clone, Copy, Debug)]
#[derive(Resource, Deref, DerefMut)]
pub struct CursorWorldPosition(Vec2);

#[derive(Clone, Copy, Debug)]
#[derive(Component, Deref, DerefMut)]
pub struct CursorDirection(EdgeDirection);

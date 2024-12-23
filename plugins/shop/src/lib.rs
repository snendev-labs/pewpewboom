use std::f32::consts::PI;

use bevy::{
    color::palettes,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use sickle_ui::{
    prelude::{
        LabelConfig, RadioGroup, UiBuilderExt, UiColumnExt, UiContainerExt, UiLabelExt,
        UiRadioGroupExt,
    },
    ui_style::generated::{SetFlexDirectionExt, SetMaxHeightExt, SetOverflowExt},
    SickleUiPlugin,
};

use game_loop::{GamePhase, GamePlayers, Player, Ready};
use merchandise::{Merch, MerchMaterials, MerchRegistry, Purchase};
use tilemap::{
    CursorDirection, CursorWorldPosition, EmptyTile, EmptyTileMaterial, TargetedTile,
    TerritoryTileMaterial, Tile,
};
use tiles::{
    lasers::{Direction, Position, Rotation},
    Territory,
};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.init_resource::<CursorCapture>();
        app.add_systems(
            Update,
            (
                Self::setup_ui,
                Self::handle_shop_selection,
                Self::handle_ready,
                Self::handle_player_control,
                Self::capture_cursor.run_if(resource_exists::<CursorCapture>),
                Self::render_territories.run_if(
                    resource_exists::<TerritoryTileMaterial>
                        .and_then(resource_exists::<EmptyTileMaterial>)
                        .and_then(resource_exists::<ControllingPlayer>),
                ),
                Self::update_tile_material.run_if(
                    resource_exists_and_changed::<SelectedMerch>
                        .or_else(resource_removed::<SelectedMerch>())
                        .or_else(resource_exists_and_changed::<TargetedTile>)
                        .or_else(resource_removed::<TargetedTile>()),
                ),
                Self::make_purchase.run_if(
                    resource_exists::<SelectedMerch>
                        .and_then(resource_exists::<TargetedTile>)
                        .and_then(resource_exists::<ControllingPlayer>),
                ),
                Self::clear_shop,
                Self::spawn_drag_markers,
                Self::update_old_purchases,
                Self::update_tile_parameters,
                Self::start_drag.run_if(resource_exists::<CursorWorldPosition>),
                Self::handle_drag,
                Self::stop_drag,
            )
                .chain()
                .in_set(ShopSystems),
        );
    }
}

impl ShopPlugin {
    fn setup_ui(
        mut commands: Commands,
        games: Query<(&GamePhase, &GamePlayers), Or<(Changed<GamePhase>, Added<GamePlayers>)>>,
        merch_registry: Res<MerchRegistry>,
    ) {
        if let Some((_, players)) = games
            .get_single()
            .ok()
            .filter(|(phase, _)| matches!(phase, GamePhase::Choose))
        {
            info!("Running setup ui");
            let root = commands.spawn(ShopUIRoot::bundle()).id();
            commands
                .ui_builder(root)
                .column(|column| {
                    column.label(LabelConfig::from("L. MARTY's LASER MART"));
                    let merch = merch_registry
                        .sorted()
                        .into_iter()
                        .map(|(_, merch)| merch.name())
                        .collect();
                    column
                        .radio_group(merch, None, false)
                        .insert(ShopMerchOption)
                        .style()
                        .max_height(Val::Percent(100.))
                        .overflow(Overflow::clip_y())
                        .flex_direction(FlexDirection::Column);
                    let player_labels = (0..players.len())
                        .map(|index| format!("Player {}", index + 1).to_string())
                        .collect::<Vec<_>>();
                    column
                        .radio_group(player_labels, 0, false)
                        .insert(ShopPlayerSwitch)
                        .style()
                        .max_height(Val::Percent(100.))
                        .overflow(Overflow::clip_y())
                        .flex_direction(FlexDirection::Row);
                    column
                        .container(
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(30.),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::Srgba(palettes::css::BLUE).into(),
                                ..default()
                            },
                            |container| {
                                container.label(LabelConfig::from("Ready!"));
                            },
                        )
                        .insert(ReadyButton);
                })
                .style()
                .max_height(Val::Percent(100.));
        }
    }

    fn clear_shop(
        mut commands: Commands,
        games: Query<&GamePhase>, // Not working with Changed filter for some reason...
        shop: Query<Entity, With<ShopUIRoot>>,
    ) {
        if !games
            .get_single()
            .is_ok_and(|phase| matches!(phase, GamePhase::Choose))
        {
            for shop_entity in &shop {
                commands.entity(shop_entity).despawn_recursive();
            }

            commands.remove_resource::<SelectedMerch>();
        };
    }

    fn handle_shop_selection(
        mut commands: Commands,
        selected_merch: Option<ResMut<SelectedMerch>>,
        merch_registry: Res<MerchRegistry>,
        shop_radio_group: Query<&RadioGroup, (With<ShopMerchOption>, Changed<RadioGroup>)>,
    ) {
        let Ok(shop_radio_group) = shop_radio_group.get_single() else {
            return;
        };
        let shop_inventory = merch_registry.sorted();
        if let Some(selection) = shop_radio_group.selected() {
            let Some((_, merch)) = shop_inventory.get(selection) else {
                commands.remove_resource::<SelectedMerch>();
                return;
            };
            if let Some(mut selected_merch) = selected_merch {
                selected_merch.0 = (*merch).clone();
            } else {
                commands.insert_resource(SelectedMerch((*merch).clone()));
            }
        } else if selected_merch.is_some() {
            commands.remove_resource::<SelectedMerch>();
        }
    }

    fn handle_player_control(
        mut commands: Commands,
        controlling_player: Option<ResMut<ControllingPlayer>>,
        players: Query<&GamePlayers>,
        player_switch: Query<&RadioGroup, (With<ShopPlayerSwitch>, Changed<RadioGroup>)>,
    ) {
        let Ok(player_switch) = player_switch.get_single() else {
            return;
        };
        let Ok(game_players) = players
            .get_single()
            .and_then(|players| Ok((**players).clone()))
        else {
            return;
        };

        if let Some(selected_player) = player_switch
            .selected()
            .and_then(|index| Some(game_players[index]))
        {
            if let Some(mut controlling_player) = controlling_player {
                controlling_player.0 = selected_player;
                info!("Setting controlling player to {}", selected_player);
            } else {
                info!("Inserting controlling player to {}", selected_player);
                commands.insert_resource(ControllingPlayer(selected_player))
            }
        }
    }

    fn handle_ready(
        mut commands: Commands,
        mut interactions: Query<
            (&mut BackgroundColor, &Interaction),
            (Changed<Interaction>, With<ReadyButton>),
        >,
        players: Query<Entity, With<Player>>,
    ) {
        for (mut color, interaction) in &mut interactions {
            match interaction {
                Interaction::Pressed => {
                    *color = Color::Srgba(palettes::css::DARK_BLUE).into();
                    for player in &players {
                        commands.entity(player).insert(Ready);
                    }
                    info!("Players are ready");
                }
                Interaction::Hovered => {
                    *color = Color::Srgba(palettes::css::LIGHT_BLUE).into();
                }
                Interaction::None => {
                    *color = Color::Srgba(palettes::css::BLUE).into();
                }
            }
        }
    }

    fn render_territories(
        controlling_player: Option<Res<ControllingPlayer>>,
        territories: Query<&Territory>,
        mut tile_materials: Query<(Entity, &mut Handle<ColorMaterial>), With<EmptyTile>>,
        territory_tile_material: Res<TerritoryTileMaterial>,
        empty_tile_material: Res<EmptyTileMaterial>,
        mut last_controlling_player: Local<Option<ControllingPlayer>>,
    ) {
        if let Some(territory) = controlling_player
            .as_ref()
            .and_then(|player| territories.get(***player).ok())
        {
            for (_, mut material) in tile_materials
                .iter_mut()
                .filter(|(tile, _)| territory.contains(tile))
            {
                *material = territory_tile_material.clone();
            }
        }

        if controlling_player.as_deref().cloned() != *last_controlling_player {
            if let Some(territory) = last_controlling_player
                .as_ref()
                .and_then(|player| territories.get(**player).ok())
            {
                for (_, mut material) in tile_materials
                    .iter_mut()
                    .filter(|(tile, _)| territory.contains(tile))
                {
                    *material = empty_tile_material.clone();
                }
            }
        }

        *last_controlling_player = controlling_player.as_deref().cloned();
    }

    fn update_tile_material(
        mut tile_materials: Query<&mut Handle<ColorMaterial>>,
        merch_materials: Res<MerchMaterials>,
        selected_merch: Option<Res<SelectedMerch>>,
        targeted_tile: Option<Res<TargetedTile>>,
        empty_tiles: Query<&EmptyTile>,
        empty_tile_material: Res<EmptyTileMaterial>,
        mut last_target: Local<Option<TargetedTile>>,
        mut last_merch: Local<Option<SelectedMerch>>,
    ) {
        if let Some(targeted_tile) = targeted_tile
            .as_deref()
            .filter(|targeted_tile| empty_tiles.contains(targeted_tile.tile))
        {
            if let Ok(mut tile_material) = tile_materials.get_mut(targeted_tile.tile) {
                if let Some(merch_material) = selected_merch
                    .as_deref()
                    .and_then(|merch| merch_materials.get(&merch.id()))
                {
                    *tile_material = merch_material.clone();
                }
            }
        }
        // todo: check that tile isn't occupied
        // Only checking for empty tiles and returning them to empty material, but filled tiles are
        // changed permanently - this is the bug
        // Remove resource run condition doesn't seem to be working - not running in final tick after
        // `TargetedTile` is removed
        if targeted_tile.as_deref().cloned() != *last_target {
            if let Some(target) = last_target
                .as_ref()
                .filter(|last_target| empty_tiles.contains(last_target.tile))
            {
                if let Ok(mut tile_material) = tile_materials.get_mut(target.tile) {
                    *tile_material = empty_tile_material.clone();
                }
            }
        }
        if let Some(targeted_tile) = targeted_tile
            .as_deref()
            .filter(|targeted_tile| empty_tiles.contains(targeted_tile.tile))
        {
            let selected_merch = selected_merch.as_deref();
            if selected_merch != last_merch.as_ref() {
                if let Ok(mut tile_material) = tile_materials.get_mut(targeted_tile.tile) {
                    *tile_material = selected_merch
                        .and_then(|selected_merch| merch_materials.get(&selected_merch.id()))
                        .cloned()
                        .unwrap_or_else(|| empty_tile_material.clone());
                }
            }
        }
        *last_target = targeted_tile.as_deref().cloned();
        *last_merch = selected_merch.as_deref().cloned();
    }

    fn capture_cursor(
        mut capture: ResMut<CursorCapture>,
        windows: Query<&Window, With<PrimaryWindow>>,
        nodes: Query<(&Node, &GlobalTransform)>,
        dragging: Query<&Dragging>,
    ) {
        let Ok(window) = windows.get_single() else {
            return;
        };

        let Some(cursor) = window.cursor_position() else {
            return;
        };

        let on_shop = nodes.iter().any(|(node, transform)| {
            let node_position = transform.translation().xy();
            let half_size = 0.5 * node.size();
            let min = node_position - half_size;
            let max = node_position + half_size;
            (min.x..max.x).contains(&cursor.x) && (min.y..max.y).contains(&cursor.y)
        });

        capture.0 = !dragging.is_empty() || on_shop;
    }

    fn make_purchase(
        mut commands: Commands,
        mut purchases: EventWriter<Purchase>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        selected_merch: Option<Res<SelectedMerch>>,
        targeted_tile: Option<Res<TargetedTile>>,
        mut purchase_tile: Option<ResMut<PurchaseOnTile>>,
        controlling_player: Res<ControllingPlayer>,
        capture: Option<Res<CursorCapture>>,
    ) {
        if capture.is_some_and(|capture| capture.0) {
            return;
        }

        if let Some(targeted_tile) = targeted_tile.as_deref() {
            if let Some(merch) = selected_merch.as_deref() {
                if mouse_input.just_released(MouseButton::Left) {
                    info!(
                        "Sent purchase for merch {:?} on tile {:?}",
                        (**merch).clone(),
                        targeted_tile.tile
                    );
                    purchases.send(Purchase::new(
                        **controlling_player,
                        (**merch).clone(),
                        targeted_tile.tile,
                    ));

                    if let Some(tile) = purchase_tile.as_deref_mut() {
                        **tile = targeted_tile.tile;
                    } else {
                        commands.insert_resource(PurchaseOnTile(targeted_tile.tile));
                    }
                }
            }
        }
    }

    fn spawn_drag_markers(
        mut commands: Commands,
        purchases: Query<(Entity, &Children), With<JustPurchased>>,
        tile_adjusters: Query<&TileAdjuster>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for (entity, _) in purchases
            .iter()
            .filter(|(_, children)| !children.iter().any(|child| tile_adjusters.contains(*child)))
        {
            info!("Spawning tile adjuster");
            let marker = commands
                .spawn((
                    TileAdjuster,
                    TileAdjuster::spawn(&mut meshes, &mut materials),
                ))
                .id();

            commands.entity(entity).add_child(marker);
        }
    }

    fn update_old_purchases(
        mut commands: Commands,
        games: Query<&GamePhase, Changed<GamePhase>>,
        markers: Query<Entity, With<TileAdjuster>>,
        purchases: Query<Entity, With<JustPurchased>>,
    ) {
        if let Some(_) = games
            .get_single()
            .ok()
            .filter(|game_phase| !matches!(game_phase, GamePhase::Choose))
        {
            for marker in &markers {
                commands.entity(marker).despawn();
            }

            for purchase in &purchases {
                commands.entity(purchase).remove::<JustPurchased>();
            }
        }
    }

    fn update_tile_parameters(
        tile_adjusters: Query<(&Parent, &Transform), (Changed<Transform>, With<TileAdjuster>)>,
        mut tiles: Query<(Option<&mut Direction>, Option<&mut Rotation>), With<Position>>,
    ) {
        for (parent, transform) in &tile_adjusters {
            if let Ok((direction, rotation)) = tiles.get_mut(**parent) {
                if let Some(mut direction) = direction {
                    *direction = TileAdjuster::to_direction(transform.translation);
                }

                if let Some(mut rotation) = rotation {
                    *rotation = TileAdjuster::to_rotation(transform.translation);
                }
            }
        }
    }

    fn start_drag(
        mut commands: Commands,
        mouse_input: Res<ButtonInput<MouseButton>>,
        cursor_position: Res<CursorWorldPosition>,
        markers: Query<(Entity, &GlobalTransform), With<TileAdjuster>>,
    ) {
        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }

        if let Some((marker_entity, _)) = markers
            .iter()
            .filter(|(_, transform)| {
                transform.translation().xy().distance(**cursor_position) <= TileAdjuster::RADIUS
            })
            .next()
        {
            info!("Dragging inserted in marker at current cursor position");
            commands.entity(marker_entity).insert(Dragging);
        }
    }

    fn handle_drag(
        mut markers: Query<(&Parent, &mut Transform), (With<TileAdjuster>, With<Dragging>)>,
        game_tiles: Query<(Entity, &Position)>,
        tiles: Query<(&Tile, &CursorDirection)>,
    ) {
        let Ok((parent, mut transform)) = markers.get_single_mut() else {
            return;
        };

        let Ok((_, position)) = game_tiles.get(**parent) else {
            info!("No existing game tiles are the parent to the dragged marker");
            return;
        };

        if let Some((_, cursor_direction)) = tiles.iter().find(|(&tile, _)| *tile == **position) {
            let cursor_direction: Direction = (**cursor_direction).into();
            let angle = match cursor_direction {
                Direction::North => 0.,
                Direction::Northwest => PI / 3.,
                Direction::Southwest => 2. * PI / 3.,
                Direction::South => PI,
                Direction::Southeast => 4. * PI / 3.,
                Direction::Northeast => 5. * PI / 3.,
            };

            let rotation = Quat::from_rotation_z(angle);
            transform.translation = rotation.mul_vec3(TileAdjuster::OFFSET);
        }
    }

    fn stop_drag(
        mut commands: Commands,
        mouse_input: Res<ButtonInput<MouseButton>>,
        markers: Query<Entity, (With<TileAdjuster>, With<Dragging>)>,
    ) {
        if mouse_input.just_released(MouseButton::Left) {
            for marker in &markers {
                commands.entity(marker).remove::<Dragging>();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct ShopSystems;

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct ShopUIRoot;

impl ShopUIRoot {
    fn bundle() -> impl Bundle {
        (
            Self,
            Name::new("Shop UI Root"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                border_color: Color::Srgba(palettes::css::DARK_SLATE_GRAY).into(),
                background_color: Color::Srgba(palettes::css::SLATE_BLUE).into(),
                ..Default::default()
            },
        )
    }
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct ShopMerchOption;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deref, DerefMut, Resource, Reflect)]
pub struct SelectedMerch(Merch);

#[derive(Debug)]
#[derive(Component)]
pub struct ReadyButton;

#[derive(Debug, Default)]
#[derive(Resource)]
pub struct CursorCapture(pub bool);

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct ShopPlayerSwitch;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deref, DerefMut, Resource, Reflect)]
pub struct ControllingPlayer(Entity);

#[derive(Clone, Debug)]
#[derive(Deref, DerefMut, Resource, Reflect)]
pub struct PurchaseOnTile(Entity);

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct JustPurchased;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct TileAdjuster;

impl TileAdjuster {
    pub const OFFSET: Vec3 = Vec3::new(0., 70., 0.);
    pub const RADIUS: f32 = 10.;

    pub fn spawn(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> MaterialMesh2dBundle<ColorMaterial> {
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: Self::RADIUS,
            })),
            material: materials.add(Color::BLACK),
            transform: Transform::from_translation(Self::OFFSET),
            ..default()
        }
    }

    pub fn to_direction(position: Vec3) -> Direction {
        match position.angle_between(Vec3::X) {
            theta if theta < PI / 3. && theta >= 0. && position.y >= 0. => Direction::Northeast,
            theta if theta >= PI / 3. && theta < 2. * PI / 3. && position.y >= 0. => {
                Direction::North
            }
            theta if theta >= 2. * PI / 3. && theta <= PI && position.y >= 0. => {
                Direction::Northwest
            }
            theta if theta >= 2. * PI / 3. && theta <= PI && position.y < 0. => {
                Direction::Southwest
            }
            theta if theta >= PI / 3. && theta < 2. * PI / 3. && position.y < 0. => {
                Direction::South
            }
            _ => Direction::Southeast,
        }
    }

    pub fn to_rotation(position: Vec3) -> Rotation {
        match position.angle_between(Vec3::X) {
            theta if theta < PI / 3. && theta >= 0. && position.y >= 0. => Rotation::new(1),
            theta if theta >= PI / 3. && theta < 2. * PI / 3. && position.y >= 0. => {
                Rotation::new(2)
            }
            theta if theta >= 2. * PI / 3. && theta <= PI && position.y >= 0. => Rotation::new(3),
            theta if theta >= 2. * PI / 3. && theta <= PI && position.y < 0. => Rotation::new(4),
            theta if theta >= PI / 3. && theta < 2. * PI / 3. && position.y < 0. => {
                Rotation::new(5)
            }
            _ => Rotation::new(6),
        }
    }
}

#[derive(Clone, Copy)]
#[derive(Component)]
pub struct Dragging;

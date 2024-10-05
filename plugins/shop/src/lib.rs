use bevy::{color::palettes, prelude::*};
use sickle_ui::{
    prelude::{LabelConfig, RadioGroup, UiBuilderExt, UiColumnExt, UiLabelExt, UiRadioGroupExt},
    ui_style::generated::{SetFlexDirectionExt, SetMaxHeightExt, SetOverflowExt},
    SickleUiPlugin,
};

use game_loop::{GamePhase, Player};
use merchandise::{Merch, MerchMaterials, MerchRegistry, Purchase};
use tilemap::{EmptyTile, EmptyTileMaterial, TargetedTile};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.add_systems(
            Update,
            (
                Self::setup_ui,
                Self::handle_shop_selection,
                Self::update_tile_material.run_if(
                    resource_exists_and_changed::<SelectedMerch>
                        .or_else(resource_removed::<SelectedMerch>())
                        .or_else(resource_exists_and_changed::<TargetedTile>)
                        .or_else(resource_removed::<TargetedTile>()),
                ),
                Self::make_purchase.run_if(
                    resource_exists::<SelectedMerch>.and_then(resource_exists::<TargetedTile>),
                ),
            )
                .chain()
                .in_set(ShopSystems),
        );
    }
}

impl ShopPlugin {
    fn setup_ui(
        mut commands: Commands,
        games: Query<&GamePhase, Changed<GamePhase>>,
        merch_registry: Res<MerchRegistry>,
    ) {
        if !games
            .get_single()
            .is_ok_and(|phase| matches!(phase, GamePhase::Choose))
        {
            return;
        };
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
            })
            .style()
            .max_height(Val::Percent(100.));
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
                } else {
                    *tile_material = empty_tile_material.clone();
                }
            }
        }
        // todo: check that tile isn't occupied

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

    fn make_purchase(
        mut purchases: EventWriter<Purchase>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        selected_merch: Option<Res<SelectedMerch>>,
        targeted_tile: Option<Res<TargetedTile>>,
        players: Query<Entity, With<Player>>,
    ) {
        if let Some(targeted_tile) = targeted_tile.as_deref() {
            if let Some(merch) = selected_merch.as_deref() {
                if mouse_input.just_released(MouseButton::Left) {
                    info!(
                        "Sent purchase for merch {:?} on tile {:?}",
                        (**merch).clone(),
                        targeted_tile.tile
                    );
                    purchases.send(Purchase::new(
                        players.iter().next().expect("No players added to game"),
                        (**merch).clone(),
                        targeted_tile.tile,
                    ));
                }
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

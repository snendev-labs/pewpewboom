use bevy::{color::palettes, prelude::*};
use sickle_ui::{
    prelude::{LabelConfig, RadioGroup, UiBuilderExt, UiColumnExt, UiLabelExt, UiRadioGroupExt},
    ui_style::generated::{SetFlexDirectionExt, SetMaxHeightExt, SetOverflowExt},
    SickleUiPlugin,
};

use game_loop::GamePhase;
use merchandise::{Merch, MerchRegistry};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.add_systems(
            Update,
            (Self::setup_ui, Self::handle_shop_selection).in_set(ShopSystems),
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

#[derive(Debug)]
#[derive(Resource, Reflect)]
pub struct SelectedMerch(Merch);

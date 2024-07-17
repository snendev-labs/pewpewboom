use bevy::{color::palettes, prelude::*};
use game_loop::GamePhase;
use sickle_ui::{
    prelude::{LabelConfig, UiBuilderExt, UiColumnExt, UiLabelExt},
    SickleUiPlugin,
};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);
        app.add_systems(Update, Self::setup_ui.in_set(ShopSystems));
    }
}

impl ShopPlugin {
    fn setup_ui(mut commands: Commands, games: Query<&GamePhase, Changed<GamePhase>>) {
        if games
            .get_single()
            .is_ok_and(|phase| !matches!(phase, GamePhase::Choose))
        {
            return;
        };
        info!("Spawning shop...");
        let root = commands.spawn(ShopUIRoot::bundle()).id();
        commands.ui_builder(root).column(|column| {
            column.label(LabelConfig::from("HELLO WORLD!"));
        });
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

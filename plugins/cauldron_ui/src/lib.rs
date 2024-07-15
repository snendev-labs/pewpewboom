use bevy::prelude::*;

use cauldron::{Cauldron, CauldronOption, CauldronSystems};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct CauldronUISystems;

pub struct CauldronUIPlugin;

impl Plugin for CauldronUIPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            CauldronUISystems
                .run_if(any_with_component::<Camera2d>)
                .after(CauldronSystems),
        )
        .add_systems(Update, Self::spawn_ui.in_set(CauldronUISystems));
    }
}

impl CauldronUIPlugin {
    fn spawn_ui(
        mut commands: Commands,
        new_cauldrons: Query<Entity, (With<Cauldron>, With<CauldronOption>, Without<CauldronUI>)>,
    ) {
        for entity in &new_cauldrons {
            let root = commands.spawn(CauldronUIRoot::bundle()).id();
            commands.entity(entity).insert(CauldronUI(root));
        }
    }

    fn _handle_ui(mut _commands: Commands, _ui_query: Query<Entity, With<CauldronUI>>) {}

    fn _despawn_ui(mut _commands: Commands, _ui_query: Query<Entity, With<CauldronUI>>) {}
}

#[derive(Component, Reflect)]
pub struct CauldronUI(Entity);

#[derive(Component, Reflect)]
pub struct CauldronUIRoot;

impl CauldronUIRoot {
    fn bundle() -> impl Bundle {
        (
            Self,
            Name::new("Cauldron UI Root"),
            NodeBundle {
                style: Style {
                    ..Default::default()
                },
                background_color: Color::ANTIQUE_WHITE.into(),
                border_color: Color::DARK_GRAY.into(),
                z_index: ZIndex::Local(2),
                ..Default::default()
            },
        )
    }
}

use bevy::prelude::*;

use encounter::{Encounter, EncounterSystems};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct EncounterUISystems;

pub struct EncounterUIPlugin;

impl Plugin for EncounterUIPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            EncounterUISystems
                .run_if(any_with_component::<Camera2d>)
                .after(EncounterSystems),
        )
        .add_systems(Update, Self::spawn_ui.in_set(EncounterUISystems));
    }
}

impl EncounterUIPlugin {
    fn spawn_ui(
        mut _commands: Commands,
        _new_encounters: Query<Entity, (With<Encounter>, Without<EncounterUI>)>,
    ) {
    }

    fn _handle_ui(mut _commands: Commands, _ui_query: Query<Entity, With<EncounterUI>>) {}

    fn _despawn_ui(mut _commands: Commands, _ui_query: Query<Entity, With<EncounterUI>>) {}
}

#[derive(Component, Reflect)]
pub struct EncounterUI(Entity);

#[derive(Component, Reflect)]
pub struct EncounterUIRoot;

impl EncounterUIRoot {
    fn bundle() -> impl Bundle {
        (
            Self,
            Name::new("Encounter UI Root"),
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

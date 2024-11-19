use bevy::{
    app::{Plugin, Update},
    color::{palettes, Color},
    log::info,
    prelude::{
        Commands, Component, Entity, Event, IntoSystemConfigs, Query, Res, SystemSet, Text,
        Text2dBundle, TextStyle, Timer, TimerMode, Transform, Trigger, With,
    },
    time::Time,
    utils::Duration,
};
use tilemap::Tile;

#[derive(Clone, Copy, Debug)]
pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (Self::tick_popups, Self::despawn_popups).in_set(PopupSystems),
        );
        app.observe(Popup::spawn_popup);
    }
}

impl PopupPlugin {
    fn tick_popups(time: Res<Time>, mut popups: Query<&mut PopupBundle>) {
        for mut popup in &mut popups {
            popup.timer.tick(time.delta());
        }
    }

    fn despawn_popups(mut commands: Commands, popups: Query<(Entity, &mut PopupBundle)>) {
        for (popup_entity, popup) in &popups {
            if popup.timer.finished() {
                commands.entity(popup_entity).despawn();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct PopupSystems;

#[derive(Event)]
pub struct PopupEvent {
    pub text: String,
}

impl Popup {
    fn spawn_popup(
        trigger: Trigger<PopupEvent>,
        mut commands: Commands,
        transforms: Query<&Transform, With<Tile>>,
    ) {
        let entity = trigger.entity();
        info!("Popup spawn triggered");

        if let Ok(transform) = transforms.get(entity) {
            info!("Spawning popup at {:?}", transform);
            commands.spawn((
                PopupBundle {
                    popup: Popup,
                    timer: Timer::new(Duration::from_millis(1500), TimerMode::Once),
                },
                Text2dBundle {
                    text: Text::from_section(
                        trigger.event().text.clone(),
                        TextStyle {
                            font_size: 16.0,
                            color: Color::Srgba(palettes::css::LIGHT_SLATE_GRAY),
                            ..Default::default()
                        },
                    ),
                    transform: *transform,
                    ..Default::default()
                },
            ));
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component)]
pub struct Popup;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct PopupBundle {
    pub popup: Popup,
    pub timer: Timer,
}

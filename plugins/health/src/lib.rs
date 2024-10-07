use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>()
            .add_systems(Update, Self::despawn_dead_entities.in_set(HealthSystems));
    }
}

impl HealthPlugin {
    fn despawn_dead_entities(
        mut commands: Commands,
        query: Query<(Entity, &Health), Changed<Health>>,
    ) {
        for (entity, health) in &query {
            if **health <= 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct HealthSystems;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Health(usize);

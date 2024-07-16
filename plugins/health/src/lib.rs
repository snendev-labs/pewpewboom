use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Deref, DerefMut)]
pub struct Health(usize);

use actions::ActionSet;
use bevy::prelude::*;

use game_instance::{HasTurn, Hero, Villain};
use monsters::Monster;

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::handle_villain_turn,).chain().in_set(TurnSystems),
        );
    }
}

impl TurnPlugin {
    fn handle_villain_turn(
        // TODO: link villain to currently deployed monsters
        mut commands: Commands,
        villain: Query<Entity, (With<Villain>, With<HasTurn>)>,
        monsters: Query<&ActionSet, With<Monster>>,
    ) {
        if villain.is_empty() {
            return;
        }
        for actions in &monsters {
            let action = actions.rand();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct TurnSystems;

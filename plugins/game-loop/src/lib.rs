use bevy::prelude::*;

pub struct GameLoopPlugin;

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionsCompleteEvent>().add_systems(
            Update,
            (
                Self::spawn_players,
                Self::complete_choose_phase,
                Self::complete_act_phase,
            )
                .chain()
                .in_set(GameLoopSystems),
        );
    }
}

impl GameLoopPlugin {
    fn spawn_players(
        mut commands: Commands,
        new_games: Query<Entity, (With<GameInstance>, Without<GamePlayers>)>,
    ) {
        for new_game in &new_games {
            let players = (0..2)
                .map(|_| commands.spawn((Player, InGame(new_game))).id())
                .collect();

            commands.entity(new_game).insert(GamePlayers(players));
        }
    }

    fn complete_choose_phase(
        mut games: Query<(Entity, &mut GamePhase, &GamePlayers)>,
        players: Query<Option<&Ready>, With<Player>>,
    ) {
        for (_, mut phase, game_players) in &mut games {
            if let GamePhase::Choose = phase.as_ref() {
                let all_ready = game_players
                    .iter()
                    .map(|entity| players.get(*entity).ok().flatten())
                    .all(|ready| ready.is_some());
                if all_ready {
                    *phase = GamePhase::Act;
                }
            }
        }
    }

    fn complete_act_phase(
        mut games: Query<&mut GamePhase>,
        mut events: EventReader<ActionsCompleteEvent>,
    ) {
        for ActionsCompleteEvent { game } in events.read() {
            if let Ok(mut phase) = games.get_mut(*game) {
                *phase = GamePhase::Choose;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct GameLoopSystems;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct GameInstance;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub enum GamePhase {
    #[default]
    Choose,
    Act,
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Deref, Reflect)]
pub struct GamePlayers(Vec<Entity>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Component, Deref, Reflect)]
pub struct InGame(Entity);

#[derive(Debug, Default)]
#[derive(Bundle)]
pub struct GameInstanceBundle {
    instance: GameInstance,
    phase: GamePhase,
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Player;

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Ready;

#[derive(Event)]
pub struct ActionsCompleteEvent {
    game: Entity,
}

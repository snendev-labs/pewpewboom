use bevy::prelude::*;

pub struct GameLoopPlugin;

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionCompleteEvent>()
            .add_event::<DrawingCompleteEvent>()
            .add_systems(
                Update,
                (
                    Self::spawn_players,
                    Self::complete_choose_phase,
                    Self::complete_action_phase,
                    Self::complete_drawing_phase,
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
                .map(|index| {
                    commands
                        .spawn((
                            Player,
                            InGame(new_game),
                            PlayerColorAdjuster((index as f32 / 2.) * 0.5),
                        ))
                        .id()
                })
                .collect();

            commands.entity(new_game).insert(GamePlayers(players));
        }
    }

    fn complete_choose_phase(
        mut commands: Commands,
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
                    for player in &game_players.0 {
                        commands.entity(*player).remove::<Ready>();
                    }
                    info!("Game phase changed to act");
                }
            }
        }
    }

    fn complete_action_phase(
        mut games: Query<&mut GamePhase>,
        mut events: EventReader<ActionCompleteEvent>,
    ) {
        for ActionCompleteEvent { game } in events.read() {
            if let Ok(mut phase) = games.get_mut(*game) {
                *phase = GamePhase::Draw;
                info!("Game phase changed to draw")
            }
        }
    }

    fn complete_drawing_phase(
        mut games: Query<&mut GamePhase>,
        mut events: EventReader<DrawingCompleteEvent>,
    ) {
        for DrawingCompleteEvent { game } in events.read() {
            if let Ok(mut phase) = games.get_mut(*game) {
                *phase = GamePhase::Choose;
                info!("Game phase changed to choose")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct GameLoopSystems;

#[derive(Debug)]
#[derive(Event)]
pub struct SpawnGame {
    pub instance: Entity,
    pub radius: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub struct GameInstance;

#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Reflect)]
pub enum GamePhase {
    #[default]
    Choose,
    Act,
    Draw,
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Deref, Reflect)]
pub struct GamePlayers(Vec<Entity>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Component, Deref, Reflect)]
pub struct InGame(Entity);

impl InGame {
    pub fn new(entity: Entity) -> InGame {
        Self(entity)
    }

    pub fn inner(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Default)]
#[derive(Bundle)]
pub struct GameInstanceBundle {
    instance: GameInstance,
    phase: GamePhase,
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Player;

// Probably move this somewhere more appropriate later...
#[derive(Clone, Copy, Debug, Default)]
#[derive(Component, Deref, Reflect)]
pub struct PlayerColorAdjuster(pub f32);

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Ready;

#[derive(Event)]
pub struct ActionCompleteEvent {
    pub game: Entity,
}

#[derive(Event)]
pub struct DrawingCompleteEvent {
    pub game: Entity,
}

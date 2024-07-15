use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct GameInstanceSystems;

pub struct GameInstancePlugin;

impl Plugin for GameInstancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::start_game.in_set(GameInstanceSystems));
    }
}

impl GameInstancePlugin {
    fn start_game(mut commands: Commands, new_games: Query<Entity, Added<GameInstance>>) {
        for entity in &new_games {
            commands
                .entity(entity)
                .insert((Name::new("Game Instance"), Floor::default()));
            let hero = commands
                .spawn((
                    Name::new("Player one (Hero)"),
                    InGame(entity),
                    Player::One,
                    Hero,
                    Health(100),
                    HasTurn,
                ))
                .id();
            let villain = commands
                .spawn((
                    Name::new("Player two (Villain)"),
                    InGame(entity),
                    Player::Two,
                    Villain,
                ))
                .id();
            commands
                .entity(entity)
                .insert((GameHero(hero), GameVillain(villain)));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct GameInstance;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct GameHero(Entity);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct GameVillain(Entity);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct HasTurn;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Floor(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Deref, Reflect)]
pub struct InGame(Entity);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Health(usize);

impl From<usize> for Health {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct BonusHealth(usize);

impl From<usize> for BonusHealth {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub enum Player {
    One,
    Two,
}

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Hero;

#[derive(Debug)]
#[derive(Component, Reflect)]
pub struct Villain;

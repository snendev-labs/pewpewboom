use bevy::prelude::*;
use bevy_anytoasts::{AnyToastsExt, Result, ResultVec};

use game_instance::{Floor, GameInstance, InGame, Villain};
use monsters::Monster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct CauldronSystems;

pub struct CauldronPlugin;

impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::spawn_options.anyhow(), Self::select_option.anyhow())
                .chain()
                .in_set(CauldronSystems),
        );
    }
}

impl CauldronPlugin {
    fn spawn_options(
        mut commands: Commands,
        cauldrons: Query<(Entity, &InGame), (With<Cauldron>, Without<CauldronOptions>)>,
        games: Query<&Floor, With<GameInstance>>,
    ) -> ResultVec<()> {
        let mut errors = vec![];
        for (entity, in_game) in &cauldrons {
            let floor = match games.get(**in_game) {
                Ok(floor) => floor,
                Err(error) => {
                    errors.push(anyhow::Error::new(error));
                    continue;
                }
            };
            let mut options = vec![];
            for option in Cauldron::options(**floor) {
                let option = commands.spawn((option, in_game.clone())).id();
                options.push(option);
            }
            commands.entity(entity).insert(CauldronOptions(options));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn select_option(
        mut commands: Commands,
        mut villain: Query<(&mut MonstersOwned, &InGame), With<Villain>>,
        cauldrons: Query<(Entity, &CauldronPurchase, &InGame), With<Cauldron>>,
        options: Query<&CauldronOption>,
    ) -> Result<()> {
        let (mut monsters, in_game) = villain.get_single_mut()?;
        for (entity, selection, in_game) in &cauldrons {
            unimplemented!();
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct Cauldron;

impl Cauldron {
    pub fn options(floor: usize) -> Vec<CauldronOption> {
        match floor {
            0 => {
                vec![
                    (Monster::Slime, 4).into(),
                    (Monster::Slime, 5).into(),
                    (Monster::Goblin, 6).into(),
                    (Monster::Skeleton, 7).into(),
                ]
            }
            _ => {
                todo!()
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub struct CauldronOption {
    monster: Monster,
    price: usize,
}

impl From<(Monster, usize)> for CauldronOption {
    fn from((monster, price): (Monster, usize)) -> Self {
        CauldronOption { monster, price }
    }
}

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct CauldronOptions(Vec<Entity>);

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct MonstersOwned(Vec<Entity>);

#[derive(Clone, Debug)]
#[derive(Component, Reflect)]
pub struct CauldronPurchase(Entity);

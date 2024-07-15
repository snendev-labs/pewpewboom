use bevy::prelude::*;
use bevy_anytoasts::{AnyToastsExt, Result, ResultVec};

use game_instance::{BonusHealth, GameInstance, Health, Hero};

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::setup_actions,
                Self::reset_actions,
                Self::run_actions.anyhow(),
            )
                .chain()
                .in_set(ActionsSystems),
        );
    }
}

impl ActionsPlugin {
    fn run_actions(
        mut commands: Commands,
        query: Query<&ActionList, With<GameInstance>>,
        attack_casters: Query<&AttackDamage>,
        block_casters: Query<&BlockDamage>,
        mut targets: Query<(&mut Health, Option<&mut BonusHealth>, Option<&mut Poison>)>,
    ) -> ResultVec<()> {
        let mut errors = vec![];
        for actions in &query {
            for action in actions.iter() {
                if let Err(error) = Self::run_action(
                    action,
                    &mut commands,
                    &attack_casters,
                    &block_casters,
                    &mut targets,
                ) {
                    errors.push(error);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn run_action(
        action: &Action,
        commands: &mut Commands,
        attack_casters: &Query<&AttackDamage>,
        block_casters: &Query<&BlockDamage>,
        targets: &mut Query<(&mut Health, Option<&mut BonusHealth>, Option<&mut Poison>)>,
    ) -> Result<()> {
        let (mut hp, bonus_hp, poison) = targets.get_mut(action.target)?;
        match action.ability {
            Ability::Attack => {
                let damage = attack_casters.get(action.caster)?;
                if let Some(mut bonus_hp) = bonus_hp {
                    if **damage >= **bonus_hp {
                        commands.entity(action.target).remove::<BonusHealth>();
                        **hp = hp.saturating_sub(**damage - **bonus_hp);
                    } else {
                        **bonus_hp = bonus_hp.saturating_sub(**damage);
                    }
                } else {
                    **hp = hp.saturating_sub(**damage);
                }
            }
            Ability::Block | Ability::Shield => {
                let damage = block_casters.get(action.caster)?;
                if let Some(mut bonus_hp) = bonus_hp {
                    **bonus_hp += **damage;
                } else {
                    commands
                        .entity(action.target)
                        .insert(BonusHealth::from(**damage));
                }
            }
            Ability::Taunt => {
                commands.entity(action.target).try_insert(Taunt);
            }
            Ability::Poison => {
                if let Some(mut poison) = poison {
                    **poison += 1;
                } else {
                    commands.entity(action.target).insert(Poison(1));
                }
            }
        }
        Ok(())
    }

    fn setup_actions(
        mut commands: Commands,
        heroes_missing_actions: Query<Entity, (With<Hero>, Without<ActionSet>)>,
    ) {
        for entity in &heroes_missing_actions {
            let mut actions = ActionSet::default();
            actions.insert(Ability::Attack);
            actions.insert(Ability::Block);
            commands
                .entity(entity)
                .insert((actions, AttackDamage(5), BlockDamage(5)));
        }
    }

    fn reset_actions(mut commands: Commands, taunters: Query<Entity, With<Taunt>>) {
        for entity in &taunters {
            commands.entity(entity).remove::<Taunt>();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct ActionsSystems;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct AttackDamage(usize);

impl From<usize> for AttackDamage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct BlockDamage(usize);

impl From<usize> for BlockDamage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub enum Ability {
    Attack,
    Block,
    Taunt,
    Shield,
    Poison,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct Action {
    ability: Ability,
    caster: Entity,
    target: Entity,
}

impl Action {
    pub fn new(ability: Ability, caster: Entity, target: Entity) -> Self {
        Self {
            ability,
            caster,
            target,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct ActionList(pub Vec<Action>);

#[derive(Clone, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct ActionSet(bevy::utils::HashSet<Ability>);

impl ActionSet {
    pub fn rand(&self) -> Option<Action> {
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct Poison(usize);

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct Taunt;

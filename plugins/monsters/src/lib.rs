use bevy::prelude::*;

use actions::{Ability, ActionSet, AttackDamage, BlockDamage};
use game_instance::Health;

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum Monster {
    Slime,
    Goblin,
    Skeleton,
    ZombiePriest,
    Witch,
    Minotaur,
}

impl Monster {
    pub fn health(&self) -> Health {
        match self {
            Monster::Slime => 4,
            Monster::Goblin => 7,
            Monster::Skeleton => 8,
            Monster::ZombiePriest => 6,
            Monster::Witch => 5,
            Monster::Minotaur => 15,
        }
        .into()
    }

    pub fn action_set(&self) -> ActionSet {
        let mut actions = ActionSet::default();
        match self {
            Monster::Slime => {
                actions.insert(Ability::Attack);
                actions.insert(Ability::Taunt);
            }
            Monster::Goblin => {
                actions.insert(Ability::Attack);
            }
            Monster::Skeleton => {
                actions.insert(Ability::Attack);
                actions.insert(Ability::Block);
            }
            Monster::ZombiePriest => {
                actions.insert(Ability::Shield);
            }
            Monster::Witch => {
                actions.insert(Ability::Poison);
            }
            Monster::Minotaur => {
                actions.insert(Ability::Attack);
                actions.insert(Ability::Block);
            }
        }
        actions
    }

    pub fn attack_damage(&self) -> Option<AttackDamage> {
        match self {
            Monster::Slime => Some(2.into()),
            Monster::Goblin => Some(5.into()),
            Monster::Skeleton => Some(3.into()),
            Monster::ZombiePriest => None,
            Monster::Witch => None,
            Monster::Minotaur => Some(10.into()),
        }
    }

    pub fn block_damage(&self) -> Option<BlockDamage> {
        match self {
            Monster::Slime => None,
            Monster::Goblin => None,
            Monster::Skeleton => Some(2.into()),
            Monster::ZombiePriest => Some(3.into()),
            Monster::Witch => None,
            Monster::Minotaur => Some(4.into()),
        }
    }
}

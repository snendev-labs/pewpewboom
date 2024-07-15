use bevy::{prelude::*, utils::HashSet};

use game_instance::{Health, InGame};
use monsters::Monster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct EncounterSystems;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                Self::despawn_dead_monsters,
                Self::spawn_monsters,
                Self::despawn_completed_encounters,
            )
                .chain()
                .in_set(EncounterSystems),
        );
    }
}

impl EncounterPlugin {
    fn spawn_monsters(
        mut commands: Commands,
        new_encounters: Query<(Entity, &Encounter, &InGame), Without<EncounterMonsters>>,
    ) {
        for (encounter_entity, encounter, in_game) in &new_encounters {
            let mut monster_entities = HashSet::default();
            for monster in &encounter.monsters {
                let mut monster_builder = commands.spawn((
                    *monster,
                    monster.health(),
                    monster.action_set(),
                    InEncounter(encounter_entity),
                    *in_game,
                ));
                if let Some(attack) = monster.attack_damage() {
                    monster_builder.insert(attack);
                }
                if let Some(block) = monster.block_damage() {
                    monster_builder.insert(block);
                }
                monster_entities.insert(monster_builder.id());
            }
            commands
                .entity(encounter_entity)
                .insert(EncounterMonsters(monster_entities));
        }
    }

    fn despawn_dead_monsters(
        mut commands: Commands,
        mut encounters: Query<&mut EncounterMonsters>,
        monsters: Query<(Entity, &Health, &InEncounter), With<Monster>>,
    ) {
        for (monster, hp, in_encounter) in &monsters {
            if **hp == 0 {
                let mut encounter_monsters = encounters
                    .get_mut(**in_encounter)
                    .expect("encounter should be valid");
                encounter_monsters.remove(&monster);
                commands.entity(monster).despawn();
            }
        }
    }

    fn despawn_completed_encounters(
        mut commands: Commands,
        encounters: Query<(Entity, &EncounterMonsters), With<Encounter>>,
    ) {
        for (entity, encounter_monsters) in &encounters {
            if encounter_monsters.is_empty() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct Encounter {
    monsters: Vec<Monster>,
}

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct EncounterMonsters(HashSet<Entity>);

#[derive(Component, Deref, Reflect)]
pub struct InEncounter(Entity);

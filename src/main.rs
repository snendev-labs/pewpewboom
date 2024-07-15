use bevy::prelude::*;

use actions::{Ability, Action, ActionList, ActionSet, ActionsPlugin, AttackDamage, BlockDamage};
use cauldron::CauldronPlugin;
use encounter::EncounterPlugin;
use game_instance::{GameInstance, GameInstancePlugin, Health, Hero};
use turns::TurnPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins((
        ActionsPlugin,
        GameInstancePlugin,
        EncounterPlugin,
        CauldronPlugin,
        TurnPlugin,
    ));

    let instance = app.world.spawn(GameInstance).id();
    app.update();

    let mut command = String::new();
    loop {
        print!(">: ");
        let _ = std::io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        match command.trim() {
            "attack" => {
                let mut query = app
                    .world
                    .query_filtered::<(Entity, &ActionSet), With<AttackDamage>>();
                let (entity, actions) = query
                    .get_single_mut(&mut app.world)
                    .expect("attacking caster to have action set and attack damage");
                if actions.contains(&Ability::Attack) {
                    app.world
                        .get_entity_mut(instance)
                        .unwrap()
                        .insert(ActionList(vec![Action::new(
                            Ability::Attack,
                            entity,
                            todo!(),
                        )]));
                }
            }
            "block" => {
                let mut query = app
                    .world
                    .query_filtered::<(Entity, &ActionSet), With<BlockDamage>>();
                let (entity, actions) = query
                    .get_single_mut(&mut app.world)
                    .expect("attacking caster to have action set and block damage");
                if actions.contains(&Ability::Attack) {
                    app.world
                        .get_entity_mut(entity)
                        .unwrap()
                        .insert(ActionList(vec![Action::new(
                            Ability::Block,
                            entity,
                            entity,
                        )]));
                }
            }
            "quit" => {
                println!("Quiting");
                break;
            }
            _ => {
                println!("Unrecognized commmand, breaking: {:?}", command);
                break;
            }
        }
        app.update();
        let mut query = app.world.query_filtered::<&Health, With<Hero>>();
        let hp = query
            .get_single(&app.world)
            .expect("hero should have health");
        println!("Hero health is {}.", **hp);
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::query::QueryData;
    use game_instance::{Hero, Player, Villain};

    use super::*;

    #[test]
    fn test_game_instance() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins((ActionsPlugin, GameInstancePlugin));

        app.world.spawn(GameInstance);

        app.update();

        fn count<T: QueryData>(app: &mut App, expected_count: usize) {
            let mut query = app.world.query::<T>();
            let iter = query.iter(&app.world);
            assert_eq!(expected_count, iter.count());
        }

        count::<&Player>(&mut app, 2);
        count::<(&Player, &Hero)>(&mut app, 1);
        count::<(&Player, &Villain)>(&mut app, 1);
        count::<(&Player, &Hero, &Villain)>(&mut app, 0);
    }
}

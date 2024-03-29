use super::{gamelog, raws, Combatant, Monster, OnBattle, Party, RunState, WantsToEncounter};
use specs::prelude::*;

pub fn invoke_battle(ecs: &mut World) {
    if is_encounter(ecs) {
        raws::spawn_named_fighter(&raws::RAWS.lock().unwrap(), ecs, "Red Lime");
        raws::spawn_named_fighter(&raws::RAWS.lock().unwrap(), ecs, "Red Lime");
    }

    {
        let entities = ecs.entities();
        let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
        let mut on_battles = ecs.write_storage::<OnBattle>();
        let monster = ecs.write_storage::<Monster>();
        let combatant = ecs.write_storage::<Combatant>();

        // 最初のwants_encounterだけ処理する
        for wants_encounter in (&wants_encounter).join().take(1) {
            let player_entity = ecs.fetch::<Entity>();
            let parties = ecs.read_storage::<Party>();
            let party = parties.get(*player_entity).unwrap();

            // god modeのときはエンカウントしない
            if party.god_mode {
                entities.delete(wants_encounter.monster).expect("Unable to delete");
                return;
            }

            // main process
            let mut runstate = ecs.write_resource::<RunState>();
            *runstate = RunState::BattleEncounter;

            let mut combat_monsters = vec![];
            for (entity, _monster, _combatant) in (&entities, &monster, &combatant).join() {
                combat_monsters.push(entity);
            }

            on_battles.clear();

            // battleを作成し、player entityに追加する
            on_battles
                .insert(
                    *player_entity,
                    OnBattle {
                        monster: wants_encounter.monster,
                        monsters: combat_monsters,
                        xp: 0,
                        gold: 0.0,
                        drops: Vec::new(),
                    },
                )
                .expect("Unable to insert encounter");

            gamelog::clear_log(&crate::gamelog::BATTLE_LOG);
            gamelog::Logger::new()
                .append("Monster appearing!")
                .log(&crate::gamelog::LogKind::Battle);
        }
        wants_encounter.clear();
    }
}

fn is_encounter(ecs: &World) -> bool {
    let mut is_encounter = false;

    let wants_encounter = ecs.write_storage::<WantsToEncounter>();
    for _wants_encounter in (&wants_encounter).join().take(1) {
        is_encounter = true;
    }
    return is_encounter;
}

use super::{gamelog, Combatant, Monster, OnBattle, Party, Player, RunState, WantsToEncounter};
use specs::prelude::*;

pub fn is_encounter(ecs: &mut World) -> bool {
    let mut is_encounter = false;

    let wants_encounter = ecs.write_storage::<WantsToEncounter>();
    for _wants_encounter in (&wants_encounter).join().take(1) {
        is_encounter = true;
    }
    return is_encounter;
}

pub fn invoke_battle(ecs: &mut World) {
    let entities = ecs.entities();
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battle = ecs.write_storage::<OnBattle>();
    let monster = ecs.write_storage::<Monster>();
    let mut combatant = ecs.write_storage::<Combatant>();
    let player = ecs.read_storage::<Player>();

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

        // battleを作成し、player entityに追加する
        battle
            .insert(
                *player_entity,
                OnBattle { monster: wants_encounter.monster, monsters: combat_monsters },
            )
            .expect("Unable to insert encounter");

        gamelog::clear_log(&crate::gamelog::BATTLE_LOG);
        gamelog::Logger::new().append("Monster appearing!").log(&crate::gamelog::LogKind::Battle);
    }
    wants_encounter.clear();
}

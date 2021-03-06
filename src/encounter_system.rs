use super::{
    gamelog::BattleLog, spawner, Battle, Combatant, Monster, Player, Pools, RunState,
    WantsToEncounter,
};
use specs::prelude::*;

pub fn is_encounter(ecs: &mut World) -> bool {
    let mut is_encounter = false;

    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    for wants_encounter in (&wants_encounter).join().take(1) {
        is_encounter = true;
    }
    return is_encounter;
}

pub fn invoke_battle(ecs: &mut World) {
    let entities = ecs.entities();
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();
    let mut battle = ecs.write_storage::<Battle>();
    let mut monster = ecs.write_storage::<Monster>();
    let mut combatant = ecs.write_storage::<Combatant>();
    let player = ecs.read_storage::<Player>();

    // 最初のwants_encounterだけ処理する
    for wants_encounter in (&wants_encounter).join().take(1) {
        // god mode
        let player_entity = ecs.fetch::<Entity>();
        let pools = ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.god_mode {
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

        // battleを作成する
        battle
            .insert(wants_encounter.monster, Battle { monsters: combat_monsters })
            .expect("Unable to insert encounter");

        // playerを戦闘中にする
        for (entity, _player) in (&entities, &player).join() {
            combatant.insert(entity, Combatant {}).expect("Unable to insert combatant");
        }

        battlelog.entries = vec![];
        battlelog.entries.push(format!("Monster appearing!"));
    }
    wants_encounter.clear();
}

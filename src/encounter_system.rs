use super::{gamelog::BattleLog, Battle, Combatant, RunState, WantsToEncounter, Player};
use specs::prelude::*;

pub fn invoke_battle(ecs: &mut World) {
    let entities = ecs.entities();
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();
    let mut battle = ecs.write_storage::<Battle>();
    let mut combatant = ecs.write_storage::<Combatant>();
    let mut player = ecs.read_storage::<Player>();

    for wants_encounter in (&wants_encounter).join().take(1) {
        // 最初のwants_encounterだけ処理する
        let mut runstate = ecs.write_resource::<RunState>();
        *runstate = RunState::BattleEncounter;

        // battleを作成する
        battle
            .insert(wants_encounter.monster, Battle { monster: wants_encounter.monster })
            .expect("Unable to insert encounter");

        // monsterを戦闘中にする
        combatant
            .insert(wants_encounter.monster, Combatant {})
            .expect("Unable to insert combatant");

        // playerを戦闘中にする
        for (entity, _player) in (&entities, &player).join() {
            combatant.insert(entity, Combatant {})
            .expect("Unable to insert combatant");
        }

        battlelog.entries = vec![];
        battlelog.entries.push(format!("Monster appearing!"));
    }
    wants_encounter.clear();
}

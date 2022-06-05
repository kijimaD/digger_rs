use super::{gamelog::BattleLog, Battle, RunState, WantsToEncounter};
use specs::prelude::*;

pub fn invoke_battle(ecs: &mut World) {
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();
    let mut battle = ecs.write_storage::<Battle>();

    for wants_encounter in (&wants_encounter).join().take(1) {
        // 最初のwants_encounterだけ処理する
        let mut runstate = ecs.write_resource::<RunState>();
        *runstate = RunState::BattleEncounter;
        battle
            .insert(wants_encounter.monster, Battle { monster: wants_encounter.monster })
            .expect("Unable to insert encounter");

        battlelog.entries = vec![];
        battlelog.entries.push(format!("Monster appearing!"));
    }
    wants_encounter.clear();
}

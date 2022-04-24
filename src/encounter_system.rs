use super::{
    WantsToEncounter, gamelog::BattleLog, Battle, RunState, spawner
};
use specs::prelude::*;

pub struct EncounterSystem {}

impl<'a> System<'a> for EncounterSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, WantsToEncounter>,
        WriteExpect<'a, BattleLog>,
        WriteStorage<'a, Battle>,
        WriteExpect<'a, RunState>,
    );

    // この関数は実行されてはいるが、stateが切り替わらない。
    fn run(&mut self, data: Self::SystemData) {
        let(
            mut wants_encounter,
            mut log,
            mut battle,
            mut runstate,
        ) = data;

        for wants_encounter in (&wants_encounter).join().take(1) {
            *runstate = RunState::BattleEncounter;
            battle
                .insert(
                    wants_encounter.monster,
                    Battle {
                        monster: wants_encounter.monster,
                    },
                )
                .expect("Unable to insert encounter");

            log.entries = vec![];
            log.entries.push(format!("Monster appearing"));
        }
        wants_encounter.clear();
    }
}

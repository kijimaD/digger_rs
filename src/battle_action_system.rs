// 戦闘用entityごとに、wants_to_meleeを生成する
use super::{CombatStats, Monster, WantsToMelee};
use specs::prelude::*;

pub struct BattleActionSystem {}

// 敵entityから、味方へのwants_to_meleeを発生させる

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, mut wants_to_melee, combat_stats, monster) = data;

        // monster -> player
        for (entity, _stats, _monster) in (&entities, &combat_stats, &monster).join() {
            wants_to_melee
                .insert(
                    entity,
                    WantsToMelee {
                        target: *player_entity,
                    },
                )
                .expect("Unable to insert WantsToMelee");
        }
    }
}

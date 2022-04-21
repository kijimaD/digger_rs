// 戦闘用entityごとに、wants_to_meleeを生成する
use super::{CombatStats, Monster, Player, WantsToMelee};
use specs::prelude::*;

pub struct BattleActionSystem {}

// 敵entityから、味方へのwants_to_meleeを発生させる

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, _player_entity, mut wants_to_melee, combat_stats, player, monster) = data;

        // monster -> player
        for (m_entity, _stats, _monster) in (&entities, &combat_stats, &monster).join() {
            for (p_entity, _combat_stats, _player) in
                (&entities, &combat_stats, &player).join().take(1)
            {
                wants_to_melee
                    .insert(
                        m_entity,
                        WantsToMelee {
                            target: p_entity, // player
                        },
                    )
                    .expect("Unable to insert WantsToMelee");
            }
        }
    }
}

// 戦闘用entityごとに、wants_to_meleeを生成する
use super::{Monster, Player, Pools, WantsToMelee};
use specs::prelude::*;

pub struct BattleActionSystem {}

// 敵entityから、味方へのwants_to_meleeを発生させる

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, _player_entity, mut wants_to_melee, pools, player, monster) = data;

        // monster -> player
        for (m_entity, _pools, _monster) in (&entities, &pools, &monster).join() {
            for (p_entity, _pools, _player) in (&entities, &pools, &player).join().take(1) {
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

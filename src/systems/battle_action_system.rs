/// 戦闘用entityごとに、wants_to_meleeを生成する
use crate::{Combatant, Monster, Player, Pools, WantsToMelee};
use specs::prelude::*;

pub struct BattleActionSystem {}

/// 敵entityから、味方へのwants_to_meleeを発生させる。味方の場合はコマンド指定するため、このシステムでは対象にしない

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Combatant>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, _player_entity, mut wants_to_melee, pools, player, monster, combatant) =
            data;

        // monster -> player
        for (m_entity, _pools, _monster, _combatant) in
            (&entities, &pools, &monster, &combatant).join()
        {
            for (player_entity, _pools, _player) in (&entities, &pools, &player).join().take(1) {
                wants_to_melee
                    .insert(m_entity, WantsToMelee { target: player_entity, way: None })
                    .expect("Unable to insert WantsToMelee");
            }
        }
    }
}

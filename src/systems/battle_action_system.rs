/// 戦闘用entityごとに、wants_to_meleeを生成する
use crate::{Combatant, Monster, NaturalAttackDefense, Player, Pools, WantsToMelee};
use rand::seq::SliceRandom;
use specs::prelude::*;

pub struct BattleActionSystem {}

/// 敵entityから、味方へのwants_to_meleeを発生させる。味方の場合はコマンド指定するため、このシステムでは対象にしない
/// 実際のところNPCの戦闘コマンドを決定するので、名前を変えたほうがよいかもしれない。たとえば体力が少なく回復手段を持つモンスターは30％の可能性で回復アイテムを使う、など

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Combatant>,
        ReadStorage<'a, NaturalAttackDefense>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, _player_entity, mut wants_to_melee, pools, player, monster, combatant, nad) =
            data;
        let mut rng = rand::thread_rng();

        // monster -> player
        for (m_entity, _pools, _monster, _combatant, nad) in
            (&entities, &pools, &monster, &combatant, &nad).join()
        {
            for (p_entity, _pools, _player) in (&entities, &pools, &player).join().take(1) {
                wants_to_melee
                    .insert(
                        m_entity,
                        WantsToMelee {
                            target: p_entity,                                        // player
                            way: nad.attacks.choose(&mut rng).unwrap().name.clone(), // TODO: 名前と攻撃の実態が一致してないので、一致させる。wayには攻撃方法そのものを入れないといけない
                        },
                    )
                    .expect("Unable to insert WantsToMelee");
            }
        }
    }
}

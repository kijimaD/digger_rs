// battle_entityごとに、wants_to_meleeを生成する
use specs::prelude::*;
use super::{BattleEntity, Player, WantsToMelee};

pub struct BattleActionSystem {}

// 敵entityから、味方へのwants_to_meleeを発生させる

impl<'a> System<'a> for BattleActionSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, BattleEntity>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, WantsToMelee>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, player_entity, battle_entity, players, mut wants_to_melee) = data;

        // monster -> player
        for (entity, battle_entity) in (&entities, &battle_entity).join() {
            wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity });
        }
    }
}

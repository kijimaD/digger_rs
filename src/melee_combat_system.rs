use super::{
    gamelog::BattleLog, spawner, Battle, CombatStats, DefenseBonus, Equipped, MeleePowerBonus,
    Name, RunState, SufferDamage, WantsToEncounter, WantsToMelee, particle_system::ParticleBuilder, Position
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

// battle state用のsystem
// コマンドを入力するたびに実行して敵とプレイヤーの双方の攻撃を行う
// 回すたびにwants_to_meleeを一つ消してenter待ちstateにすれば一つずつ攻撃ができそう

// 戦闘用実装のメモ
// 1. 接触したときwants_to_encounterを生成してstateを切り替え
// 2. wants_to_encounterを削除
// 3. battle_entityを生成
// 4. battle_entityそれぞれでプレイヤーコマンド or AIによってwants_to_meleeを生成＋処理でダメージを発生させる。これで1ターンとする。プレイヤーのwantsはコマンドで生成し、AIのwantsはbattle_entityから生成するか。
// 5. 敵のbattle_entityが残っていれば再度コマンド選択に戻る
// 6. 敵のbattle_entityが残っていなければbattle_resultに移動して戦闘を終了する

// TODO: entityが複数の攻撃手段を持つようにする。player entityの場合はコマンドで選択肢、モンスターの場合はAI選択。
// <wants_to_melee method, from, to>

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, BattleLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            combat_stats,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut inflict_damage,
            mut particle_builder,
            positions
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in
                        (&entities, &defense_bonuses, &equipped).join()
                    {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    // TODO: 戦闘モードに対応させる
                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }

                    let mut rng = RandomNumberGenerator::new();
                    let damage = i32::max(
                        0,
                        (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus),
                    ) + rng.range(1, 5);

                    if damage == 0 {
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {}, for {} hp",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}

// TODO: systemにしたいが、state切り替えがうまくいかず諦めた
pub fn invoke_battle(ecs: &mut World) {
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();
    let mut battle = ecs.write_storage::<Battle>();

    for wants_encounter in (&wants_encounter).join().take(1) {
        // 最初のwants_encounterだけ処理する
        let mut runstate = ecs.write_resource::<RunState>();
        *runstate = RunState::BattleEncounter;
        battle
            .insert(
                wants_encounter.monster,
                Battle {
                    monster: wants_encounter.monster,
                },
            )
            .expect("Unable to insert encounter");

        battlelog.entries = vec![];
        battlelog.entries.push(format!("Monster appearing"));
    }
    wants_encounter.clear();
}

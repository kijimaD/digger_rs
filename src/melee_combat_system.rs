use specs::prelude::*;
use super::{CombatStats, WantsToMelee, WantsToEncounter, Name, gamelog::{GameLog, BattleLog}, MeleePowerBonus, DefenseBonus, Equipped, RunState, BattleEntity};

pub struct MeleeCombatSystem {}

// battle state用のsystem
// コマンドを入力するたびに実行して敵とプレイヤーの双方の攻撃を行う
// 回すたびにwants_to_meleeを一つ消してenter待ちstateにすれば一つずつ攻撃ができそう

// 戦闘用実装のメモ
// 1. 接触したときwants_to_encounterを生成してstateを切り替え
// 2. wants_to_encounterを削除
// 3. battle_entityを生成(entityはwants_to_encounterからcopy)
// 4. battle_entityそれぞれでプレイヤーコマンド or AIによってwants_to_meleeを生成＋処理でダメージを発生させる。これで1ターンとする
// 5. 敵のbattle_entityが残っていれば再度コマンド選択に戻る
// 6. 敵のbattle_entityが残っていなければbattle_resultに移動して戦闘を終了する

// TODO: entityが複数の攻撃手段を持つようにする。player entityの場合はコマンドで選択肢、モンスターの場合はAI選択。
// <wants_to_melee method, from, to>

// 未実装
impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        ReadStorage<'a, MeleePowerBonus>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, wants_melee, names, combat_stats, melee_power_bonuses, defense_bonuses, equipped) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in (&entities, &melee_power_bonuses, &equipped).join() {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in (&entities, &defense_bonuses, &equipped).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus));

                    if damage == 0 {
                        log.entries.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                        // SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        // wants_melee.clear();
    }
}

// TODO: 書いてる箇所が現状と合ってないので直す
pub fn delete_combat_event(ecs : &mut World) {
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();
    let mut battle_entity = ecs.write_storage::<BattleEntity>();

    for wants_encounter in (&wants_encounter).join() {
        let mut runstate = ecs.write_resource::<RunState>();
        *runstate = RunState::BattleCommand;
        battlelog.entries.push(format!("Enter Battle"));
        // 戦闘用entityを生成
        battle_entity.insert(wants_encounter.monster, BattleEntity{ monster: wants_encounter.monster }).expect("Unable to insert attack");
    }
    // エンカウント用entityは削除
    wants_encounter.clear();
}

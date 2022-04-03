use specs::prelude::*;
use super::{CombatStats, WantsToMelee, WantsToEncounter, Name, SufferDamage, gamelog::{GameLog, BattleLog}, MeleePowerBonus, DefenseBonus, Equipped, RunState};

pub struct MeleeCombatSystem {}

// battle state用のsystem
// コマンドを入力するたびに実行して敵とプレイヤーの双方の攻撃を行う
// 回すたびにwants_to_meleeを一つ消してenter待ちstateにすれば一つずつ攻撃ができそう

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, MeleePowerBonus>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, wants_melee, names, combat_stats, mut inflict_damage, melee_power_bonuses, defense_bonuses, equipped) = data;

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

pub fn delete_combat_event(ecs : &mut World) {
    let mut wants_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut battlelog = ecs.write_resource::<BattleLog>();

    for _wants_encounter in (&wants_encounter).join() {
        let mut runstate = ecs.write_resource::<RunState>();
        *runstate = RunState::BattleCommand;
        // 現在は、戦闘に入ったentityを知ることができない
        battlelog.entries.push(format!("Enter Battle"));

        // wants_to_encounterはエンカウントに使っている

        // wants_to_meleeは戦闘内で使うことにする。
        // 実際の攻撃を発生させるタイミングがない。
        // 戦闘に参加しているエンティティからwant_to_meleeを追加させて、wants_to_meleeをまとめて処理して戦闘にする
    }

    wants_encounter.clear();
    // すぐに消す方式だと1回だけstateを切り替えられるが、モンスターの情報を取ることができない
    // TODO: エンカウント用と戦闘用のentityを分ける
    // エンカウントを元に、戦闘用entityを生成して、戦闘用entityでforして、それぞれ攻撃させる。空っぽにすれば勝ち
}

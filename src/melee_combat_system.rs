use super::{
    gamelog::BattleLog, particle_system::ParticleBuilder, Attributes, DefenseBonus, Equipped,
    HungerClock, HungerState, MeleePowerBonus, Name, Pools, Position, SufferDamage, WantsToMelee,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

// battle state用のsystem
// 敵とプレイヤーのwants_to_meleeを処理して、双方の攻撃を行う
// 回すたびにwants_to_meleeを一つ消してenter待ちstateにすれば一つずつ攻撃ができそう

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
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            attributes,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut inflict_damage,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
        ) = data;

        for (entity, wants_melee, name, attacker_pools, attacker_attributes) in
            (&entities, &wants_melee, &names, &pools, &attributes).join()
        {
            if attacker_pools.hit_points.current > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let hc = hunger_clock.get(entity);
                if let Some(hc) = hc {
                    if hc.state == HungerState::WellFed {
                        offensive_bonus += 1;
                    }
                }

                let target_pools = pools.get(wants_melee.target).unwrap();
                let target_attributes = attributes.get(wants_melee.target).unwrap();
                if target_pools.hit_points.current > 0 {
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
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }

                    let mut rng = RandomNumberGenerator::new();
                    let damage = i32::max(
                        0,
                        (attacker_attributes.might.bonus + offensive_bonus)
                            - (target_attributes.might.bonus + defensive_bonus),
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

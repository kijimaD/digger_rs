use crate::systems::ParticleBuilder;
use crate::{
    effects::*, skill_bonus, Attributes, Equipped, HungerClock, HungerState, MeleeWeapon, Name,
    NaturalAttackDefense, Pools, Position, Skill, Skills, WantsToMelee, WeaponAttribute, Wearable,
};
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
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefense>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            attributes,
            skills,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
            mut rng,
            equipped_items,
            meleeweapons,
            wearables,
            natural,
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in
            (&entities, &wants_melee, &names, &attributes, &skills, &pools).join()
        {
            // Are the attacker and defender alive? Only attack if they are
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            let target_name = names.get(wants_melee.target).unwrap();

            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let mut attack_name = "punch".to_string(); // initialize
                let mut weapon_info = MeleeWeapon {
                    attribute: WeaponAttribute::Might,
                    hit_bonus: 0,
                    damage_n_dice: 1,
                    damage_die_type: 4,
                    damage_bonus: 0,
                };

                if let Some(nat) = natural.get(entity) {
                    if !nat.attacks.is_empty() {
                        let attack_index = if nat.attacks.len() == 1 {
                            0
                        } else {
                            rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1
                        };
                        weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                        weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                        weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                        weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                        attack_name = nat.attacks[attack_index].name.clone();
                    }
                }

                // wayが入るのはプレイヤーキャラでコマンド指定したとき
                if let Some(way) = wants_melee.way {
                    weapon_info = meleeweapons.get(way).unwrap().clone();
                    attack_name = names.get(way).unwrap().name.clone();
                }

                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = attacker_attributes.might.bonus;
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = 0; // TODO: Once weapons support this
                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(entity) {
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }
                let modified_hit_roll = natural_roll
                    + attribute_hit_bonus
                    + skill_hit_bonus
                    + weapon_hit_bonus
                    + status_hit_bonus;

                let base_armor_class = match natural.get(wants_melee.target) {
                    None => 10,
                    Some(nat) => nat.armor_class.unwrap_or(10),
                };
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);

                let mut armor_item_bonus_f = 0.0;
                for (wielded, armor) in (&equipped_items, &wearables).join() {
                    if wielded.owner == wants_melee.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                }
                let armor_item_bonus = armor_item_bonus_f as i32;

                let armor_class =
                    base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    // Target hit! Until we support weapons, we're going with 1d4
                    let base_damage =
                        rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(
                        0,
                        base_damage
                            + attr_damage_bonus
                            + skill_hit_bonus
                            + skill_damage_bonus
                            + weapon_damage_bonus,
                    );
                    add_effect(
                        Some(entity),
                        EffectType::Damage { amount: damage },
                        Targets::Single { target: wants_melee.target },
                    );
                    crate::gamelog::Logger::new()
                        .npc_name(&name.name)
                        .append(&attack_name)
                        .append("->")
                        .npc_name(&target_name.name)
                        .append("for")
                        .damage(damage)
                        .append("hp.")
                        .log(&crate::gamelog::LogKind::Battle);
                    if let Some(pos) = positions.get(wants_melee.target) {
                        // TODO: 戦闘モードに対応させる
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                } else if natural_roll == 1 {
                    // Natural 1 miss
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("considers attacking")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but misjudges the timing!")
                        .log(&crate::gamelog::LogKind::Battle);
                } else {
                    // Miss
                    crate::gamelog::Logger::new()
                        .color(rltk::CYAN)
                        .append(&name.name)
                        .color(rltk::WHITE)
                        .append("attacks")
                        .color(rltk::CYAN)
                        .append(&target_name.name)
                        .color(rltk::WHITE)
                        .append("but can't connect.")
                        .log(&crate::gamelog::LogKind::Battle);
                }
            }
        }

        wants_melee.clear();
    }
}

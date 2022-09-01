use super::*;
use crate::components::{Attributes, OnBattle, Party, Player, Pools};
use crate::gamelog;
use crate::gamesystem::{player_hp_at_level, sp_at_level};
use crate::map::Map;
use specs::prelude::*;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    let parties = ecs.read_storage::<Party>();
    let players = ecs.read_storage::<Player>();
    let player_entity = ecs.fetch::<Entity>();

    if let Some(pool) = pools.get_mut(target) {
        if let Some(party) = parties.get(*player_entity) {
            if !party.god_mode {
                // 攻撃主と攻撃対象が同じである場合はダメージを与えない
                if let Some(creator) = damage.creator {
                    if creator == target {
                        return;
                    }
                }
                if let EffectType::Damage { amount } = damage.effect_type {
                    pool.hit_points.current -= amount;
                    // TODO: particle系を適切にする
                    add_effect(None, EffectType::Bloodstain, Targets::Single { target });
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: rltk::to_cp437('‼'),
                            fg: rltk::RGB::named(rltk::ORANGE),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single { target },
                    );

                    if players.get(target).is_some() {
                        crate::gamelog::record_event("Damage Taken", amount);
                    }

                    if let Some(creator) = damage.creator {
                        if players.get(creator).is_some() {
                            crate::gamelog::record_event("Damage Inflicted", amount);
                        }
                    }

                    if pool.hit_points.current < 1 {
                        add_effect(
                            damage.creator,
                            EffectType::EntityDeath,
                            Targets::Single { target },
                        );
                    }
                }
            }
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}

pub fn death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    let mut pools = ecs.write_storage::<Pools>();
    let mut parties = ecs.write_storage::<Party>();
    let mut on_battles = ecs.write_storage::<OnBattle>();
    let attributes = ecs.read_storage::<Attributes>();
    let player_entity = ecs.fetch::<Entity>();

    {
        if let Some(pos) = entity_position(ecs, target) {
            crate::spatial::remove_entity(target, pos as usize);
        }
    }

    if let Some(source) = effect.creator {
        if ecs.read_storage::<Player>().get(source).is_some() {
            if let Some(stats) = pools.get(target) {
                xp_gain += stats.level * 100;
                gold_gain += stats.gold;
            }

            // 戦闘終了後に一気にまとめてxpやgoldの獲得をさせるため、OnBattleに格納するのみ
            if xp_gain != 0 || gold_gain != 0.0 {
                let mut player_stats = pools.get_mut(source).unwrap();
                let mut party = parties.get_mut(*player_entity).unwrap();
                let player_attributes = attributes.get(source).unwrap();
                let mut on_battle = on_battles.get_mut(*player_entity).unwrap();

                on_battle.xp += xp_gain;
                on_battle.gold += gold_gain;
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing { amount } = heal.effect_type {
            pool.hit_points.current =
                i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: rltk::to_cp437('‼'),
                    fg: rltk::RGB::named(rltk::GREEN),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

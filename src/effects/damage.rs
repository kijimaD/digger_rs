use super::*;
use crate::components::{Attributes, Player, Pools};
use crate::gamelog;
use crate::gamesystem::{player_hp_at_level, sp_at_level};
use crate::map::Map;
use specs::prelude::*;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    let player_entity = ecs.fetch::<Entity>();
    if let Some(pool) = pools.get_mut(target) {
        if !pool.god_mode {
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

                if target == *player_entity {
                    crate::gamelog::record_event("Damage Taken", amount);
                }

                if let Some(creator) = damage.creator {
                    if creator == *player_entity {
                        crate::gamelog::record_event("Damage Inflicted", amount);
                    }
                }

                if pool.hit_points.current < 1 {
                    add_effect(damage.creator, EffectType::EntityDeath, Targets::Single { target });
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
    let attributes = ecs.read_storage::<Attributes>();

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

            if xp_gain != 0 || gold_gain != 0.0 {
                let mut player_stats = pools.get_mut(source).unwrap();
                let player_attributes = attributes.get(source).unwrap();
                player_stats.xp += xp_gain;
                player_stats.gold += gold_gain;
                if player_stats.xp >= player_stats.level * 1000 {
                    // level up
                    player_stats.level += 1;
                    gamelog::Logger::new()
                        .append(format!("Congratulations, you are now level{}", player_stats.level))
                        .color(rltk::MAGENTA)
                        .append("Congratulations, you are now level")
                        .append(format!("{}", player_stats.level))
                        .log(&crate::gamelog::LogKind::Field);
                    player_stats.hit_points.max = player_hp_at_level(
                        player_attributes.fitness.base + player_attributes.fitness.modifiers,
                        player_stats.level,
                    );
                    player_stats.hit_points.current = player_stats.hit_points.max;
                    player_stats.sp.max = sp_at_level(
                        player_attributes.intelligence.base
                            + player_attributes.intelligence.modifiers,
                        player_stats.level,
                    );
                    player_stats.sp.current = player_stats.sp.max;

                    let player_pos = ecs.fetch::<rltk::Point>();
                    let map = ecs.fetch::<Map>();
                    for i in 0..10 {
                        if player_pos.y - i > 1 {
                            add_effect(
                                None,
                                EffectType::Particle {
                                    glyph: rltk::to_cp437('░'),
                                    fg: rltk::RGB::named(rltk::GOLD),
                                    bg: rltk::RGB::named(rltk::BLACK),
                                    lifespan: 400.0,
                                },
                                Targets::Tile {
                                    tile_idx: map.xy_idx(player_pos.x, player_pos.y - i) as i32,
                                },
                            );
                        }
                    }
                }
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

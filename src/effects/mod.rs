use specs::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;

use crate::map::Map;
mod damage;
mod targeting;
pub use targeting::*;
mod hunger;
mod movement;
mod particles;
mod triggers;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage { amount: i32 },
    Bloodstain,
    Particle { glyph: rltk::FontCharType, fg: rltk::RGB, bg: rltk::RGB, lifespan: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    WellFed,
    Healing { amount: i32 },
    TriggerFire { trigger: Entity },
    TeleportTo { x: i32, y: i32, depth: i32, player_only: bool },
}

#[derive(Clone, Debug)]
pub enum Targets {
    Party,
    Single { target: Entity },
    TargetList { targets: Vec<Entity> },
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> },
}

pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner { creator, effect_type, targets });
}

pub fn run_effects_queue(ecs: &mut World) {
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, &effect);
        } else {
            break;
        }
    }
}

/// effectの対象を元に分岐して、異なる関数を適用する
/// EffectType::ItemUseの場合、triggerが呼ばれ、triggerが各種effectを生成する。つまり間接的にeffectを生成する
fn target_applicator(ecs: &mut World, effect: &EffectSpawner) {
    if let EffectType::ItemUse { item } = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
    } else if let EffectType::TriggerFire { trigger } = effect.effect_type {
        triggers::trigger(effect.creator, trigger, &effect.targets, ecs);
    } else {
        match &effect.targets {
            Targets::Tile { tile_idx } => affect_tile(ecs, effect, *tile_idx),
            Targets::Tiles { tiles } => {
                tiles.iter().for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx))
            }
            Targets::Single { target } => affect_entity(ecs, effect, *target),
            Targets::Party {} => {
                let field_entity = *ecs.fetch::<Entity>();
                affect_entity(ecs, effect, field_entity)
            }
            Targets::TargetList { targets } => {
                targets.iter().for_each(|entity| affect_entity(ecs, effect, *entity))
            }
        }
    }
}

/// タイルへの効果がタイル上のentityにも及ぶeffect
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage { .. } => true,
        EffectType::WellFed => true,
        EffectType::Healing { .. } => true,
        EffectType::TeleportTo { .. } => true,
        _ => false,
    }
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let content = crate::spatial::get_tile_content_clone(tile_idx as usize);
        content.iter().for_each(|entity| affect_entity(ecs, effect, *entity));
    }

    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        EffectType::Particle { .. } => particles::particle_to_tile(ecs, tile_idx, &effect),
        _ => {}
    }
}

fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target),
        EffectType::Bloodstain { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                damage::bloodstain(ecs, pos)
            }
        }
        EffectType::Particle { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                particles::particle_to_tile(ecs, pos, &effect)
            }
        }
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing { .. } => damage::heal_damage(ecs, effect, target),
        EffectType::TeleportTo { .. } => movement::apply_teleport(ecs, effect, target),
        _ => {}
    }
}

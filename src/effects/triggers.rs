use super::*;
use crate::components::*;
use crate::gamelog::GameLog;
use crate::gamesystem::{mana_at_level, player_hp_at_level};
use crate::RunState;
use specs::prelude::*;

pub fn item_trigger(creator: Option<Entity>, item: Entity, targets: &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(creator, item, targets, ecs);

    // if it was a consumable, then it gets deleted
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}

fn event_trigger(creator: Option<Entity>, entity: Entity, targets: &Targets, ecs: &mut World) {
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    // food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog.entries.push(format!("You eat the {}.", names.get(entity).unwrap().name));
    }

    // town portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog.entries.push("You are already in town!".to_string());
        } else {
            gamelog.entries.push("You are teleported back to town!".to_string());
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
        }
    }

    // healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(creator, EffectType::Healing { amount: heal.heal_amount }, targets.clone());
    }

    // damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(creator, EffectType::Damage { amount: damage.damage }, targets.clone());
    }

    // teleport
    if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
        add_effect(
            creator,
            EffectType::TeleportTo {
                x: teleport.x,
                y: teleport.y,
                depth: teleport.depth,
                player_only: teleport.player_only,
            },
            targets.clone(),
        );
    }
}

pub fn trigger(creator: Option<Entity>, trigger: Entity, targets: &Targets, ecs: &mut World) {
    event_trigger(creator, trigger, targets, ecs);

    if ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Delete failed");
    }
}

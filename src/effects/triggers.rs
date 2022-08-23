use super::*;
use crate::components::*;
use crate::gamelog;
use crate::gamesystem::{player_hp_at_level, sp_at_level};
use crate::RunState;
use specs::prelude::*;

pub fn item_trigger(creator: Option<Entity>, item: Entity, targets: &Targets, ecs: &mut World) {
    // Use the item via the generic system
    let did_something = event_trigger(creator, item, targets, ecs);

    // if it was a consumable, then it gets deleted
    if did_something && ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}

fn event_trigger(
    creator: Option<Entity>,
    entity: Entity,
    targets: &Targets,
    ecs: &mut World,
) -> bool {
    let mut did_something = false;

    // food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog::Logger::new()
            .append(format!("You eat the {}.", names.get(entity).unwrap().name))
            .log(&crate::gamelog::LogKind::Field);
        did_something = true;
    }

    // town portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog::Logger::new()
                .append("You are already in town!")
                .log(&crate::gamelog::LogKind::Field);
        } else {
            gamelog::Logger::new()
                .append("You are teleported back to town!")
                .log(&crate::gamelog::LogKind::Field);
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
            did_something = true;
        }
    }

    // healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(creator, EffectType::Healing { amount: heal.heal_amount }, targets.clone());
        did_something = true;
    }

    // damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(creator, EffectType::Damage { amount: damage.damage }, targets.clone());
        did_something = true;
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
        did_something = true;
    }

    did_something
}

pub fn trigger(creator: Option<Entity>, trigger: Entity, targets: &Targets, ecs: &mut World) {
    let did_something = event_trigger(creator, trigger, targets, ecs);

    if did_something && ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Delete failed");
    }
}

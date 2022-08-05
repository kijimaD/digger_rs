use super::*;
use crate::components::{Attributes, Consumable, Name, Player, Pools, ProvidesFood};
use crate::gamelog::GameLog;
use crate::gamesystem::{mana_at_level, player_hp_at_level};
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

    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog.entries.push(format!("You eat the {}.", names.get(entity).unwrap().name));
    }
}

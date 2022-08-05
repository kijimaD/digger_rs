use super::{
    gamelog::GameLog, Consumable, EquipmentChanged, Equippable, Equipped, HungerClock, HungerState,
    InBackpack, InflictsDamage, Map, Name, Pools, ProvidesFood, ProvidesHealing, RunState,
    TownPortal, WantsToUseItem,
};
use crate::effects::*;
use specs::prelude::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Equippable>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, TownPortal>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            mut gamelog,
            mut runstate,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            _inflict_damage,
            mut pools,
            equippable,
            provides_food,
            mut hunger_clocks,
            mut equipped,
            mut backpack,
            mut dirty,
            town_portal,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged {}).expect("Unable to insert");

            add_effect(
                Some(entity),
                EffectType::ItemUse { item: useitem.item },
                Targets::Single { target: *player_entity },
            )
        }

        wants_use.clear();
    }
}

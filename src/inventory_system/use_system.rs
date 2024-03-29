use super::{Consumable, EquipmentChanged, InflictsDamage, WantsToUseItem};
use crate::{components, effects::*};
use specs::prelude::*;

pub struct ItemUseSystem {}

/// consumable itemを処理するsystem
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, EquipmentChanged>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (_player_entity, entities, mut wants_use, _inflict_damage, consumables, mut dirty) =
            data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged {}).expect("Unable to insert");

            let consumable = consumables.get(useitem.item).unwrap();
            add_effect(
                Some(entity),
                EffectType::ItemUse { item: useitem.item },
                match consumable.target {
                    components::ItemTarget::Field => Targets::Party,
                    components::ItemTarget::Battle => Targets::Single { target: useitem.target },
                },
            )
        }

        wants_use.clear();
    }
}

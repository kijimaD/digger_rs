use super::{EquipmentChanged, InflictsDamage, WantsToUseItem};
use crate::effects::*;
use specs::prelude::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, EquipmentChanged>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, entities, mut wants_use, _inflict_damage, mut dirty) = data;

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

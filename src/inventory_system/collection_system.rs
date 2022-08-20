use super::{gamelog, EquipmentChanged, InBackpack, Name, Position, WantsToPickupItem};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut wants_pickup, mut positions, names, mut backpack, mut dirty) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(pickup.item, InBackpack { owner: pickup.collected_by })
                .expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged {}).expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                gamelog::Logger::new()
                    .append(format!("You pick up the {}.", names.get(pickup.item).unwrap().name))
                    .log(&crate::gamelog::LogKind::Field);
            }
        }

        wants_pickup.clear();
    }
}

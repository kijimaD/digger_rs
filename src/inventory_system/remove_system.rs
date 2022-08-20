use super::{gamelog, Equipped, InBackpack, Name, WantsToRemoveItem};
use specs::prelude::*;

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack, names) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
            gamelog::Logger::new()
                .append(format!("You unequip {}.", names.get(to_remove.item).unwrap().name))
                .log(&crate::gamelog::LogKind::Field);
        }

        wants_remove.clear();
    }
}

use super::{gamelog::GameLog, Equipped, InBackpack, Name, WantsToRemoveItem};
use specs::prelude::*;

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack, mut gamelog, names) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
            gamelog
                .entries
                .push(format!("You unequip {}.", names.get(to_remove.item).unwrap().name));
        }

        wants_remove.clear();
    }
}

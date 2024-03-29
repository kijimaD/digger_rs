use super::{gamelog, EquipmentChanged, InBackpack, Name, Position, WantsToDropItem};
use specs::prelude::*;

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
            mut dirty,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y })
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged {}).expect("Unable to insert");

            if entity == *player_entity {
                gamelog::Logger::new()
                    .append(format!("You drop the {}.", names.get(to_drop.item).unwrap().name))
                    .log(&crate::gamelog::LogKind::Field);
            }
        }

        wants_drop.clear();
    }
}

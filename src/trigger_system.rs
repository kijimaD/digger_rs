use super::{
    effects::*, gamelog, ApplyTeleport, EntityMoved, EntryTrigger, Map, Name, Position,
    SingleActivation, TeleportTo,
};
use specs::prelude::*;

pub struct TriggerSystem {}

/// タイルにセットされたコンポーネントを元に、イベントを生成するsystem
impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        ReadStorage<'a, SingleActivation>,
        ReadStorage<'a, TeleportTo>,
        WriteStorage<'a, ApplyTeleport>,
        ReadExpect<'a, Entity>,
    );

    /// エンティティが動いたとき、その位置にトリガーエンティティがないか確認する。トリガーがあった場合、effectを生成する。effectにはtrigger entity自体を渡しているので、効果を特定できる
    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            names,
            entities,
            single_activation,
            teleporters,
            mut apply_teleport,
            player_entity,
        ) = data;

        // Iterate the entities that moved and their final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id {
                    // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(entity_id);
                            if let Some(name) = name {
                                gamelog::Logger::new()
                                    .append(format!("{} triggers!", &name.name))
                                    .log();
                            }

                            // Call the effects system
                            add_effect(
                                Some(entity),
                                EffectType::TriggerFire { trigger: entity_id },
                                Targets::Tile { tile_idx: idx as i32 },
                            );
                        }
                    }
                }
            });
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}

use super::{
    gamelog::GameLog, Consumable, EquipmentChanged, Equippable, Equipped, HungerClock, HungerState,
    InBackpack, InflictsDamage, Map, Name, Pools, ProvidesFood, ProvidesHealing, RunState,
    TownPortal, WantsToUseItem,
};
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
            let mut used_item = true;

            // Targeting
            let mut party_targets: Vec<Entity> = Vec::new();
            party_targets.push(*player_entity);

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = useitem.target;

                    // Remove any items the target has in the item's slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            gamelog.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                    // FIXME: スロットが占有されているときに装備すると、前の装備が消える
                    // 装備を外す -> バックパックにしまう
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: *player_entity })
                            .expect("Unable to insert backpack entry");
                    }

                    // Wield the item
                    equipped
                        .insert(useitem.item, Equipped { owner: target, slot: target_slot })
                        .expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    gamelog
                        .entries
                        .push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                }
            }

            // If it heals, apply the healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    used_item = false;
                    for _target in party_targets.iter() {
                        let pools = pools.get_mut(useitem.target);
                        if let Some(pools) = pools {
                            pools.hit_points.current = i32::min(
                                pools.hit_points.max,
                                pools.hit_points.current + healer.heal_amount,
                            );
                            gamelog.entries.push(format!(
                                "You use the {}, healing {} hp.",
                                names.get(useitem.item).unwrap().name,
                                healer.heal_amount
                            ));
                            used_item = true;
                        }
                    }
                }
            }

            // 食料品
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    used_item = true;
                    let target = party_targets[0];
                    let hc = hunger_clocks.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!(
                            "You eat the {}.",
                            names.get(useitem.item).unwrap().name
                        ));
                    }
                }
            }

            // town portal
            if let Some(_townportal) = town_portal.get(useitem.item) {
                if map.depth == 1 {
                    gamelog
                        .entries
                        .push("You are a already in town, so the sand does nothing.".to_string());
                } else {
                    used_item = true;
                    gamelog.entries.push("You are teleported back to town!".to_string());
                    *runstate = RunState::TownPortal;
                }
            }

            // If its a consumable, we delete it on use
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }

        wants_use.clear();
    }
}

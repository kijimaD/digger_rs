use crate::{
    gamelog, gamesystem::attr_bonus, AttributeBonus, Attributes, EquipmentChanged, Equipped,
    InBackpack, Item, Party,
};
use specs::prelude::*;
use std::collections::HashMap;

pub struct EncumbranceSystem {}

impl<'a> System<'a> for EncumbranceSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Party>,
        WriteStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, AttributeBonus>,
    );

    /// 重さとペナルティを管理する。アイテムのdirty flagが立つたびに再計算する
    fn run(&mut self, data: Self::SystemData) {
        let (
            mut equip_dirty,
            entities,
            items,
            backpacks,
            wielded,
            mut parties,
            mut attributes,
            player,
            attrbonus,
        ) = data;

        if equip_dirty.is_empty() {
            return;
        }

        struct ItemUpdate {
            weight: f32,
            initiative: f32,
            might: i32,
            fitness: i32,
            quickness: i32,
            intelligence: i32,
        }

        let mut to_update: HashMap<Entity, ItemUpdate> = HashMap::new();
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(
                entity,
                ItemUpdate {
                    weight: 0.0,
                    initiative: 0.0,
                    might: 0,
                    fitness: 0,
                    quickness: 0,
                    intelligence: 0,
                },
            );
        }

        // Remove all dirty statements
        equip_dirty.clear();

        // Total up equipped items
        for (item, equipped, entity) in (&items, &wielded, &entities).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                totals.weight += item.weight_kg;
                totals.initiative += item.initiative_penalty;
                if let Some(attr) = attrbonus.get(entity) {
                    totals.might += attr.might.unwrap_or(0);
                    totals.fitness += attr.fitness.unwrap_or(0);
                    totals.quickness += attr.quickness.unwrap_or(0);
                    totals.intelligence += attr.intelligence.unwrap_or(0);
                }
            }
        }

        // Total up carried items
        for (item, carried, entity) in (&items, &backpacks, &entities).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                totals.weight += item.weight_kg;
                totals.initiative += item.initiative_penalty;
            }
        }

        // Apply the data to Party
        for (entity, item) in to_update.iter() {
            if let Some(party) = parties.get_mut(*entity) {
                party.total_weight = item.weight;
                party.total_initiative_penalty = item.initiative;

                if let Some(attr) = attributes.get_mut(*entity) {
                    attr.might.modifiers = item.might;
                    attr.fitness.modifiers = item.fitness;
                    attr.quickness.modifiers = item.quickness;
                    attr.intelligence.modifiers = item.intelligence;
                    attr.might.bonus = attr_bonus(attr.might.base + attr.might.modifiers);
                    attr.fitness.bonus = attr_bonus(attr.fitness.base + attr.fitness.modifiers);
                    attr.quickness.bonus =
                        attr_bonus(attr.quickness.base + attr.quickness.modifiers);
                    attr.intelligence.bonus =
                        attr_bonus(attr.intelligence.base + attr.intelligence.modifiers);

                    // MEMO: 戦闘用エンティティ分割で計算できなくなったので一時的に固定
                    // let carry_capacity_kg = (attr.might.base + attr.might.modifiers) * 15;
                    let carry_capacity_kg = 100;

                    if party.total_weight as i32 > carry_capacity_kg {
                        // Overburdened
                        party.total_initiative_penalty += 4.0;
                        if *entity == *player {
                            gamelog::Logger::new()
                                .color(rltk::ORANGE)
                                .append(format!(
                                    "You are overburdened, and suffering an initiative penalty."
                                ))
                                .log(&crate::gamelog::LogKind::Field);
                        }
                    }
                }
            }
        }
    }
}

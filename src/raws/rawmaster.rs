use super::{Raws, Reaction};
use crate::components::*;
use crate::random_table::{MasterTable, RandomTable};
use crate::{attr_bonus, npc_hp, sp_at_level};
use regex::Regex;
use rltk::prelude::*;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::{HashMap, HashSet};

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    fighter_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    loot_index: HashMap<String, usize>,
    faction_index: HashMap<String, HashMap<String, Reaction>>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                fighters: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            fighter_index: HashMap::new(),
            prop_index: HashMap::new(),
            loot_index: HashMap::new(),
            faction_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate item name in raws [{}]",
                    item.name
                ));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!("WARNING - duplicate mob name in raws [{}]", mob.name));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, fighter) in self.raws.fighters.iter().enumerate() {
            if used_names.contains(&fighter.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate fighter name in raws [{}]",
                    fighter.name
                ));
            }
            self.fighter_index.insert(fighter.name.clone(), i);
            used_names.insert(fighter.name.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate prop name in raws [{}]",
                    prop.name
                ));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING! - Spawn tables references unspecified entity {}",
                    spawn.name
                ));
            }
        }

        for (i, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), i);
        }

        for faction in self.raws.faction_table.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for other in faction.responses.iter() {
                reactions.insert(
                    other.0.clone(),
                    match other.1.as_str() {
                        "ignore" => Reaction::Ignore,
                        "flee" => Reaction::Flee,
                        _ => Reaction::Attack,
                    },
                );
            }
            self.faction_index.insert(faction.name.clone(), reactions);
        }
    }
}

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(_wpn) = &item.weapon {
        return EquipmentSlot::Melee;
    } else if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

macro_rules! apply_effects {
    ( $effects:expr, $eb:expr) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => {
                    $eb =
                        $eb.with(ProvidesHealing { heal_amount: effect.1.parse::<i32>().unwrap() })
                }
                "damage" => {
                    $eb = $eb.with(InflictsDamage { damage: effect.1.parse::<i32>().unwrap() })
                }
                "town_portal" => $eb = $eb.with(TownPortal {}),
                "food" => $eb = $eb.with(ProvidesFood {}),
                "single_activation" => $eb = $eb.with(SingleActivation {}),
                _ => rltk::console::log(format!(
                    "Warning: consumable effect {} not implemented.",
                    effect_name
                )),
            }
        }
    };
}

pub fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        eb = spawn_position(pos, eb, key, raws);

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name { name: item_template.name.clone() });

        eb = eb.with(crate::components::Item {
            initiative_penalty: item_template.initiative_penalty.unwrap_or(0.0),
            weight_kg: item_template.weight_kg.unwrap_or(0.0),
            base_value: item_template.base_value.unwrap_or(0.0),
        });

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(crate::components::Consumable {});
            apply_effects!(consumable.effects, eb);
        }

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable { slot: EquipmentSlot::Melee });
            let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
            let mut wpn = MeleeWeapon {
                attribute: WeaponAttribute::Might,
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: weapon.hit_bonus,
            };
            match weapon.attribute.as_str() {
                "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
                _ => wpn.attribute = WeaponAttribute::Might,
            }
            eb = eb.with(wpn);
        }

        if let Some(wearable) = &item_template.wearable {
            let slot = string_to_slot(&wearable.slot);
            eb = eb.with(Equippable { slot });
            eb = eb.with(Wearable { slot, armor_class: wearable.armor_class });
        }

        if let Some(ab) = &item_template.attributes {
            eb = eb.with(AttributeBonus {
                might: ab.might,
                fitness: ab.fitness,
                quickness: ab.quickness,
                intelligence: ab.intelligence,
            });
        }

        return Some(eb.build());
    }
    None
}

pub fn get_vendor_items(categories: &[String], raws: &RawMaster) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(cat) = &item.vendor_category {
            if categories.contains(cat) && item.base_value.is_some() {
                result.push((item.name.clone(), item.base_value.unwrap()));
            }
        }
    }

    result
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        eb = spawn_position(pos, eb, key, raws);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips { available: quips.clone() });
        }

        if let Some(vendor) = &mob_template.vendor {
            eb = eb.with(Vendor { categories: vendor.clone() });
        }

        eb = eb.with(Name { name: mob_template.name.clone() });
        eb = eb.with(Initiative { current: 2 });

        match mob_template.movement.as_ref() {
            "random" => {
                eb = eb.with(MoveMode { mode: Movement::Random });
            }
            "random_waypoint" => {
                eb = eb.with(MoveMode { mode: Movement::RandomWaypoint { path: None } });
            }
            _ => {
                eb = eb.with(MoveMode { mode: Movement::Static });
            }
        }

        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }
        eb = eb.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        // loot item
        if let Some(loot) = &mob_template.loot_table {
            eb = eb.with(LootTable { table: loot.clone() });
        }

        if let Some(light) = &mob_template.light {
            eb = eb.with(LightSource {
                range: light.range,
                color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
            });
        }

        if let Some(faction) = &mob_template.faction {
            eb = eb.with(Faction { name: faction.clone() });
        } else {
            eb = eb.with(Faction { name: "Townsfolk".to_string() })
        }

        let new_mob = eb.build();

        // Are they wielding anything?
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, SpawnType::Equipped { by: new_mob });
            }
        }

        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = spawn_position(pos, eb, key, raws);

        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(Name { name: prop_template.name.clone() });

        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile {
                eb = eb.with(BlocksTile {})
            };
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility {
                eb = eb.with(BlocksVisibility {})
            };
        }
        if let Some(door_open) = prop_template.door_open {
            eb = eb.with(Door { open: door_open });
        }

        if let Some(light) = &prop_template.light {
            eb = eb.with(LightSource {
                range: light.range,
                color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
            });
            eb = eb.with(Viewshed { range: light.range, dirty: true, visible_tiles: Vec::new() });
        }

        if let Some(entry_trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger {});
            apply_effects!(entry_trigger.effects, eb);
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_fighter(raws: &RawMaster, ecs: &mut World, key: &str) -> Option<Entity> {
    if raws.fighter_index.contains_key(key) {
        let fighter_template = &raws.raws.fighters[raws.fighter_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = eb.with(Name { name: fighter_template.name.clone() });
        eb = eb.with(Monster {}); // TODO: 味方と敵で共通化したい
        eb = eb.with(Combatant {});
        eb = eb.with(EquipmentChanged {});

        // attr
        let mut fighter_fitness = 11;
        let mut fighter_int = 11;
        let mut attr = Attributes {
            might: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            fitness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            quickness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
        };
        if let Some(might) = fighter_template.attributes.might {
            attr.might = Attribute { base: might, modifiers: 0, bonus: attr_bonus(might) };
        }
        if let Some(fitness) = fighter_template.attributes.fitness {
            attr.fitness = Attribute { base: fitness, modifiers: 0, bonus: attr_bonus(fitness) };
            fighter_fitness = fitness;
        }
        if let Some(quickness) = fighter_template.attributes.quickness {
            attr.quickness =
                Attribute { base: quickness, modifiers: 0, bonus: attr_bonus(quickness) };
        }
        if let Some(intelligence) = fighter_template.attributes.intelligence {
            attr.intelligence =
                Attribute { base: intelligence, modifiers: 0, bonus: attr_bonus(intelligence) };
            fighter_int = intelligence;
        }
        eb = eb.with(attr);

        // pool
        let fighter_level =
            if fighter_template.level.is_some() { fighter_template.level.unwrap() } else { 1 };
        let fighter_hp = npc_hp(fighter_fitness, fighter_level);
        let fighter_sp = sp_at_level(fighter_int, fighter_level);
        let pools = Pools {
            level: fighter_level,
            xp: 0,
            hit_points: Pool { current: fighter_hp, max: fighter_hp },
            sp: Pool { current: fighter_sp, max: fighter_sp },
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: if let Some(gold) = &fighter_template.gold {
                let (n, d, b) = parse_dice_string(&gold);
                let mut rng = RandomNumberGenerator::new();
                (rng.roll_dice(n, d) + b) as f32
            } else {
                0.0
            },
        };
        eb = eb.with(pools);

        let party = Party { god_mode: false };
        eb = eb.with(party);

        // natural attack
        if let Some(na) = &fighter_template.natural {
            let mut nature =
                NaturalAttackDefense { armor_class: na.armor_class, attacks: Vec::new() };
            if let Some(attacks) = &na.attacks {
                for nattack in attacks.iter() {
                    let (n, d, b) = parse_dice_string(&nattack.damage);
                    let attack = NaturalAttack {
                        name: nattack.name.clone(),
                        hit_bonus: nattack.hit_bonus,
                        damage_n_dice: n,
                        damage_die_type: d,
                        damage_bonus: b,
                    };
                    nature.attacks.push(attack);
                }
            }
            eb = eb.with(nature);
        }

        // skill
        let mut skills = Skills { skills: HashMap::new() };
        skills.skills.insert(Skill::Melee, 1);
        skills.skills.insert(Skill::Defense, 1);
        if let Some(mobskills) = &fighter_template.skills {
            for sk in mobskills.iter() {
                match sk.0.as_str() {
                    "Melee" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    "Defense" => {
                        skills.skills.insert(Skill::Defense, *sk.1);
                    }
                    _ => {
                        rltk::console::log(format!("Unknown skill referenced: [{}]", sk.0));
                    }
                }
            }
        }
        eb = eb.with(skills);

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }

    None
}

fn spawn_position<'a>(
    pos: SpawnType,
    new_entity: EntityBuilder<'a>,
    tag: &str,
    raws: &RawMaster,
) -> EntityBuilder<'a> {
    let eb = new_entity;

    match pos {
        SpawnType::AtPosition { x, y } => eb.with(Position { x, y }),
        SpawnType::Carried { by } => eb.with(InBackpack { owner: by }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb.with(Equipped { owner: by, slot })
        }
    }
}

fn get_renderable_component(
    renderable: &super::item_structs::Renderable,
) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

/// 階層によるモンスター生成テーブルを決定する
pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> MasterTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = MasterTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt.add(e.name.clone(), weight, raws);
    }

    rt
}

pub fn string_to_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Shield" => EquipmentSlot::Shield,
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso,
        "Legs" => EquipmentSlot::Legs,
        "Feet" => EquipmentSlot::Feet,
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => {
            rltk::console::log(format!("Warning: unknown equipment slot type [{}]", slot));
            EquipmentSlot::Melee
        }
    }
}

pub fn get_item_drop(
    raws: &RawMaster,
    rng: &mut rltk::RandomNumberGenerator,
    table: &str,
) -> Option<String> {
    if raws.loot_index.contains_key(table) {
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.loot_tables[raws.loot_index[table]];
        for item in available_options.drops.iter() {
            rt.add(item.name.clone(), item.weight);
        }
        let result = rt.roll(rng);
        return Some(result);
    }

    None
}

pub fn faction_reaction(my_faction: &str, their_faction: &str, raws: &RawMaster) -> Reaction {
    if raws.faction_index.contains_key(my_faction) {
        let mf = &raws.faction_index[my_faction];
        if mf.contains_key(their_faction) {
            return mf[their_faction];
        } else if mf.contains_key("Default") {
            return mf["Default"];
        } else {
            return Reaction::Ignore;
        }
    }
    Reaction::Ignore
}

pub enum SpawnTableType {
    Item,
    Mob,
    Prop,
}

pub fn spawn_type_by_name(raws: &RawMaster, key: &str) -> SpawnTableType {
    if raws.item_index.contains_key(key) {
        SpawnTableType::Item
    } else if raws.mob_index.contains_key(key) {
        SpawnTableType::Mob
    } else {
        SpawnTableType::Prop
    }
}

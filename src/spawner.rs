use super::{
    random_table::MasterTable, raws::*, Attribute, Attributes, Combatant, EntryTrigger,
    EquipmentChanged, Faction, HungerClock, HungerState, Initiative, LightSource, Map,
    MasterDungeonMap, Name, OtherLevelPosition, Party, Player, Pool, Pools, Position, Rect,
    Renderable, SerializeMe, SingleActivation, Skill, Skills, TeleportTo, TileType, Viewshed,
};
use crate::{attr_bonus, player_hp_at_level, sp_at_level};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// フィールド用エンティティ。"@"
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let player = ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Attributes {
            might: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            fitness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            quickness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
        }) // TODO: battle entityに移動する
        .with(Party {
            god_mode: false,
            gold: 50.0,
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
        })
        .with(EquipmentChanged {})
        .with(Player {})
        .with(Faction { name: "Player".to_string() })
        .with(Initiative { current: 0 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(HungerClock { state: HungerState::WellFed, duration: 20 })
        .with(LightSource { color: rltk::RGB::from_hex("#EDF122").expect("Bad color"), range: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Starting equipment
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Rusty Longsword",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Dried Sausage",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Health Potion",
        SpawnType::Carried { by: player },
    );
    spawn_named_entity(&RAWS.lock().unwrap(), ecs, "Beer", SpawnType::Carried { by: player });
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Stained Tunic",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Torn Trousers",
        SpawnType::Equipped { by: player },
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs,
        "Town Portal Stone",
        SpawnType::Carried { by: player },
    );

    player
}

// TODO: rawで生成する
pub fn battle_player(ecs: &mut World) -> Entity {
    let mut skills = Skills { skills: HashMap::new() };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);

    let player = ecs
        .create_entity()
        .with(Player {})
        .with(Combatant {})
        .with(Name { name: "Player".to_string() })
        .with(Attributes {
            might: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            fitness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            quickness: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
            intelligence: Attribute { base: 11, modifiers: 0, bonus: attr_bonus(11) },
        })
        .with(Pools {
            hit_points: Pool {
                current: player_hp_at_level(1, 1) - 1,
                max: player_hp_at_level(1, 1),
            },
            sp: Pool { current: sp_at_level(11, 1) - 1, max: sp_at_level(11, 1) },
            xp: 0,
            level: 1,
            gold: 0.0,
        })
        .with(skills)
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    player
}

const MAX_MONSTERS: i32 = 2;

fn room_table(map_depth: i32) -> MasterTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

/// Fills a region with stuff!
pub fn spawn_region(
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS));
        if num_spawns == 0 {
            return;
        }

        for _i in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };

            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result =
        spawn_named_entity(&RAWS.lock().unwrap(), ecs, &spawn.1, SpawnType::AtPosition { x, y });

    if spawn_result.is_some() {
        // we succeeded in spawning something from the data
        return;
    }

    rltk::console::log(format!("WARNING: We dont't know how to spawn [{}]", spawn.1));
}

pub fn spawn_town_portal(ecs: &mut World) {
    // Get current position & depth
    let map = ecs.fetch::<Map>();
    let player_depth = map.depth;
    let player_pos = ecs.fetch::<rltk::Point>();
    let player_x = player_pos.x;
    let player_y = player_pos.y;
    std::mem::drop(player_pos);
    std::mem::drop(map);

    // Find part of the town for the portal
    let dm = ecs.fetch::<MasterDungeonMap>();
    let town_map = dm.get_map(1).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
        }
    }
    let portal_x = (stairs_idx as i32 % town_map.width) - 2;
    let portal_y = stairs_idx as i32 / town_map.width;

    std::mem::drop(dm);

    // Spawn the portal itself
    ecs.create_entity()
        .with(OtherLevelPosition { x: portal_x, y: portal_y, depth: 1 })
        .with(Renderable {
            glyph: rltk::to_cp437('♥'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(EntryTrigger {})
        .with(TeleportTo { x: player_x, y: player_y, depth: player_depth, player_only: true })
        .with(Name { name: "Town Portal".to_string() })
        .with(SingleActivation {})
        .build();
}

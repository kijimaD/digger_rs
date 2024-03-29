use super::{
    gamelog, BlocksTile, BlocksVisibility, Combatant, Door, EntityMoved, Faction, HungerClock,
    HungerState, Item, Map, Monster, Name, Player, Pools, Position, Renderable, RunState, State,
    TileType, Vendor, VendorMode, Viewshed, WantsToEncounter, WantsToPickupItem,
};
use crate::raws::Reaction;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let monsters = ecs.read_storage::<Monster>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();
    let mut wants_to_encounter = ecs.write_storage::<WantsToEncounter>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let vendors = ecs.read_storage::<Vendor>();
    let factions = ecs.read_storage::<Faction>();
    let mut result = RunState::AwaitingInput;

    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return RunState::AwaitingInput;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_gamemode(
            destination_idx,
            |potential_target| {
                if let Some(_vendor) = vendors.get(potential_target) {
                    return Some(RunState::ShowVendor {
                        vendor: potential_target,
                        mode: VendorMode::Sell,
                    });
                }

                let mut hostile = true;
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap(),
                    );
                    if reaction != Reaction::Attack {
                        hostile = false;
                    }
                }
                if !hostile {
                    swap_entities.push((potential_target, pos.x, pos.y));

                    pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                    pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                    entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");

                    viewshed.dirty = true;
                    let mut ppos = ecs.write_resource::<Point>();
                    ppos.x = pos.x;
                    ppos.y = pos.y;
                    return Some(RunState::Ticking);
                }
                let door = doors.get_mut(potential_target);
                if let Some(door) = door {
                    door.open = true;
                    blocks_visibility.remove(potential_target);
                    blocks_movement.remove(potential_target);
                    let glyph = renderables.get_mut(potential_target).unwrap();
                    glyph.glyph = rltk::to_cp437('/');
                    viewshed.dirty = true;
                    return Some(RunState::Ticking);
                }
                None
            },
        );

        if !crate::spatial::is_blocked(destination_idx) {
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
            let new_idx = map.xy_idx(pos.x, pos.y);
            entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");
            crate::spatial::move_entity(entity, old_idx, new_idx);

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            result = RunState::Ticking;
            match map.tiles[destination_idx] {
                TileType::DownStairs => result = RunState::NextLevel,
                TileType::UpStairs => result = RunState::PreviousLevel,
                _ => {}
            }
        }
    }

    // エンカウント
    {
        let player_pos = ecs.read_resource::<Point>();
        for (entity, _viewshed, _monster, pos) in
            (&entities, &mut viewsheds, &monsters, &mut positions).join()
        {
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                wants_to_encounter
                    .insert(entity, WantsToEncounter { monster: entity })
                    .expect("Unable to insert encounter");
            }
        }
    }

    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = m.1;
            their_pos.y = m.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            crate::spatial::move_entity(m.0, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }

    // 足元のアイテムを表示
    let player_pos = ecs.fetch::<Point>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let names = ecs.read_storage::<Name>();

    let mut target: Option<Entity> = None;
    for (entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target = Some(entity);
        }
    }

    match target {
        None => {}
        Some(target) => match target {
            _item => {
                gamelog::Logger::new()
                    .item_name(&names.get(target).unwrap().name)
                    .append(format!("is there.[G]"))
                    .log(&crate::gamelog::LogKind::Field);
            }
        },
    }

    // 足元の地形を表示
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        gamelog::Logger::new().append(format!("downstairs.")).log(&crate::gamelog::LogKind::Field);
    }

    result
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        gamelog::Logger::new()
            .append(format!("There is no way down from here."))
            .log(&crate::gamelog::LogKind::Field);
        false
    }
}

pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStairs {
        true
    } else {
        gamelog::Logger::new()
            .append(format!("There is no way up from here."))
            .log(&crate::gamelog::LogKind::Field);
        false
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog::Logger::new()
            .append(format!("There is nothing here to pick up."))
            .log(&crate::gamelog::LogKind::Field),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup");
        }
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        crate::spatial::for_each_tile_content(idx, |entity_id| {
            let faction = factions.get(entity_id);
            match faction {
                None => {}
                Some(faction) => {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap(),
                    );
                    if reaction == Reaction::Attack {
                        can_heal = false;
                    }
                }
            }
        });
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hc = hunger_clocks.get(*player_entity);
    if let Some(hc) = hc {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    let entities = ecs.entities();
    let player = ecs.read_storage::<Player>();
    let mut pools = ecs.write_storage::<Pools>();

    if can_heal {
        for (_entity, _player, pools) in (&entities, &player, &mut pools).join() {
            pools.hit_points.current = i32::min(pools.hit_points.current + 1, pools.hit_points.max);
        }
    }

    RunState::Ticking
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Hotkeys
    // MEMO: On EXWM environment, can't test shift key down...
    if ctx.shift && ctx.key.is_some() {
        let key: Option<i32> = match ctx.key.unwrap() {
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None,
        };
        if let Some(key) = key {
            return use_consumable_hotkey(gs, key - 1);
        }
    }

    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::A => {
                return try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::D => {
                return try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::W => {
                return try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::S => {
                return try_move_player(0, 1, &mut gs.ecs)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::E => {
                return try_move_player(1, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Q => {
                return try_move_player(-1, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::X => {
                return try_move_player(1, 1, &mut gs.ecs)
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::Z => {
                return try_move_player(-1, 1, &mut gs.ecs)
            }

            // Skip Turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Picking up items
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::U => return RunState::ShowUseItem,
            VirtualKeyCode::T => return RunState::ShowDropItem,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            VirtualKeyCode::I => {
                let players = gs.ecs.read_storage::<Player>();
                let combatants = gs.ecs.read_storage::<Combatant>();
                let entities = gs.ecs.entities();

                // 最初の1人を選択する
                for (entity, _combatant, _player) in
                    (&entities, &combatants, &players).join().nth(0)
                {
                    return RunState::ShowEquipItem { entity, index: 0 };
                }
            }

            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,

            // Cheating!
            VirtualKeyCode::Backslash | VirtualKeyCode::Slash => return RunState::ShowCheatMenu,

            // Level up
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            // Level down
            VirtualKeyCode::Comma => {
                if try_previous_level(&mut gs.ecs) {
                    return RunState::PreviousLevel;
                }
            }

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::Ticking
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack};

    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        return RunState::ItemTargeting { item: carried_consumables[key as usize] };
    }
    RunState::Ticking
}

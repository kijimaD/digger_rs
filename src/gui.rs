use super::{
    camera, gamelog::BattleLog, gamelog::GameLog, Attribute, Attributes, Battle, Combatant,
    Consumable, Equipped, HungerClock, HungerState, InBackpack, Map, Monster, Name, Player, Pools,
    Position, RunState, State,
};
use rltk::{Point, RandomNumberGenerator, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;
    let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    // separators
    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray, black); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray, black); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray, black); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 8, box_gray, black); // Top-right panel

    // connectors
    ctx.set(0, 45, box_gray, black, to_cp437('├'));
    ctx.set(49, 8, box_gray, black, to_cp437('├'));
    ctx.set(49, 0, box_gray, black, to_cp437('┬'));
    ctx.set(49, 45, box_gray, black, to_cp437('┴'));
    ctx.set(79, 8, box_gray, black, to_cp437('┤'));
    ctx.set(79, 45, box_gray, black, to_cp437('┤'));

    // Draw the town name
    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 1;
    let x_pos = (22 - (name_length / 2)) as i32;
    ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
    ctx.set(x_pos + name_length as i32, 0, box_gray, black, to_cp437('├'));
    ctx.print_color(x_pos + 1, 0, white, black, &map.name);
    std::mem::drop(map);

    // Draw stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health =
        format!("Health:{}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("Mana:{}/{}", player_pools.mana.current, player_pools.mana.max);
    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.draw_bar_horizontal(
        64,
        1,
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        RGB::named(rltk::RED),
        RGB::named(rltk::BLACK),
    );
    ctx.draw_bar_horizontal(
        64,
        2,
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        RGB::named(rltk::BLUE),
        RGB::named(rltk::BLACK),
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);

    // Equipped
    let mut y = 9;
    let equipped = ecs.read_storage::<Equipped>();
    let name = ecs.read_storage::<Name>();
    for (equipped_by, item_name) in (&equipped, &name).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(50, y, white, black, &item_name.name);
            y += 1;
        }
    }
}

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }

    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_gray, black, name);
    let color: RGB = if attribute.modifiers < 0 {
        RGB::from_f32(1.0, 0.0, 0.0)
    } else if attribute.modifiers == 0 {
        RGB::named(rltk::WHITE)
    } else {
        RGB::from_f32(0.0, 1.0, 0.0)
    };
    ctx.print_color(67, y, color, black, &format!("{}", attribute.base + attribute.modifiers));
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 {
        ctx.set(72, y, color, black, rltk::to_cp437('+'));
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 && map.visible_tiles[idx]
        {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );
}

pub fn show_field_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_inventory(gs, ctx);

    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();
    let mut y = (25 - (count / 2)) as i32;

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in
        (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_item_targeting(
    gs: &mut State,
    ctx: &mut Rltk,
) -> (ItemTargetingResult, Option<Entity>) {
    // とりあえず味方だけ
    // TODO: targetが必要ないもの…食料とか…では表示しないようにしたい
    show_inventory(gs, ctx);

    let entities = gs.ecs.entities();
    let pools = gs.ecs.read_storage::<Pools>();
    let name = gs.ecs.read_storage::<Name>();
    let player = gs.ecs.read_storage::<Player>();

    let count = (&pools, &player).join().count();
    let mut y = (25 - (count / 2)) as i32;
    let mut j = 0;
    let mut targets: Vec<Entity> = Vec::new();

    for (entity, _pools, name) in (&entities, &pools, &name).join() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));
        ctx.print(21, y, &name.name.to_string());

        targets.push(entity);

        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemTargetingResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemTargetingResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemTargetingResult::Selected, Some(targets[selection as usize]));
                }
                (ItemTargetingResult::NoResponse, None)
            }
        },
    }
}

pub fn show_battle_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_inventory(gs, ctx);

    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let consumable = gs.ecs.read_storage::<Consumable>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();
    let mut y = (25 - (count / 2)) as i32;

    // 戦闘中は消費アイテムしか使えない
    let mut useable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name, _consumable) in (&entities, &backpack, &names, &consumable)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        useable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(useable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut useable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in
        (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        useable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(useable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join();
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Remove Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in
        (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    // let assets = gs.ecs.fetch::<RexAssets>();
    // ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(24, 18, 31, 10, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color_centered(
        20,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Battle Digger Clone",
    );
    ctx.print_color_centered(21, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "by Kijimad");
    ctx.print_color_centered(
        22,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        "Use Up/Down Arrows and Enter",
    );

    let mut y = 24;
    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        } else {
            ctx.print_color_centered(
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                "Begin New Game",
            );
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    "Load Game",
                );
            } else {
                ctx.print_color_centered(
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    "Load Game",
                );
            }
            y += 1
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(y, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit }
                }
                VirtualKeyCode::Up => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection { selected: newselection };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection { selected: newselection };
                }
                VirtualKeyCode::Return => return MainMenuResult::Selected { selected: selection },
                _ => return MainMenuResult::NoSelection { selected: selection },
            },
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

pub fn draw_battle_ui(ecs: &World, ctx: &mut Rltk) {
    // メッセージボックス
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let log = ecs.fetch::<BattleLog>();
    let mut y = 48;
    for s in log.entries.iter().rev() {
        if y > 43 {
            ctx.print(2, y, s);
        }
        y -= 1;
    }

    // 敵一覧
    let names = ecs.read_storage::<Name>();
    let pools = ecs.read_storage::<Pools>();
    let monsters = ecs.read_storage::<Monster>();
    let combatants = ecs.read_storage::<Combatant>();

    let mut i = 1;
    for (name, pools, _combatant, _monster) in (&names, &pools, &combatants, &monsters).join() {
        ctx.print(
            (80 * i) / (1 + combatants.count()),
            20,
            format!("[{}]({})", name.name, pools.hit_points.current),
        );
        i += 1;
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum BattleCommandResult {
    NoResponse,
    Attack,
    ShowInventory,
    RunAway,
    RunAwayFailed,
}

pub fn battle_command(ecs: &mut World, ctx: &mut Rltk) -> BattleCommandResult {
    let y = 30;
    ctx.print(2, y, "[a] Attack");
    ctx.print(2, y + 1, "[i] Inventory");
    ctx.print(2, y + 2, "[r] Run away");

    match ctx.key {
        None => BattleCommandResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::A => BattleCommandResult::Attack,
            VirtualKeyCode::I => BattleCommandResult::ShowInventory,
            VirtualKeyCode::R => {
                let mut rng = RandomNumberGenerator::new();
                let num = rng.range(0, 2);
                if num == 0 {
                    // 逃走成功
                    run_away_battle(ecs);
                    return BattleCommandResult::RunAway;
                } else {
                    // 逃走失敗
                    let mut log = ecs.write_resource::<BattleLog>();
                    log.entries.push(format!("Failed run away!"));
                    return BattleCommandResult::RunAwayFailed;
                }
            }
            _ => BattleCommandResult::NoResponse,
        },
    }
}

// 逃走。
// 敵シンボルは消さずに、戦闘用エンティティだけ削除する
fn run_away_battle(ecs: &mut World) {
    let mut combatants = ecs.write_storage::<Combatant>();
    let mut monsters = ecs.read_storage::<Monster>();
    let entities = ecs.entities();

    for (entity, _combatant, _monster) in (&entities, &combatants, &monsters).join() {
        entities.delete(entity).expect("Delete failed")
    }

    // battle削除
    let mut battle = ecs.write_storage::<Battle>();
    battle.clear();

    let mut log = ecs.write_resource::<BattleLog>();
    log.entries.push(format!("Run away!"));
}

pub enum BattleTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn battle_target(gs: &mut State, ctx: &mut Rltk) -> (BattleTargetingResult, Option<Entity>) {
    let entities = gs.ecs.entities();
    let pools = gs.ecs.write_storage::<Pools>();
    let monsters = gs.ecs.read_storage::<Monster>();
    let combatants = gs.ecs.read_storage::<Combatant>();

    let mut x = 1;
    let mut j = 0;

    let mut targets: Vec<Entity> = Vec::new();
    for (entity, _pools, _combatant, _monster) in (&entities, &pools, &combatants, &monsters).join()
    {
        let base = 2 + (80 * x) / (1 + pools.count());
        ctx.set(
            base + 0,
            22,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            base + 1,
            22,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            base + 2,
            22,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        targets.push(entity);
        x += 1;
        j += 1;
    }

    match ctx.key {
        None => (BattleTargetingResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (BattleTargetingResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < targets.len() as i32 {
                    return (BattleTargetingResult::Selected, Some(targets[selection as usize]));
                }
                (BattleTargetingResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum BattleResult {
    NoResponse,
    Enter,
}

pub fn show_battle_win_result(_gs: &mut State, ctx: &mut Rltk) -> BattleResult {
    ctx.print_color(70, 44, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "[Enter]");

    match ctx.key {
        None => (BattleResult::NoResponse),
        Some(key) => match key {
            VirtualKeyCode::Return => (BattleResult::Enter),
            _ => BattleResult::NoResponse,
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Your journey has ended!",
    );
    ctx.print_color_centered(
        17,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "That day, sadly, is not in this chapter..",
    );

    ctx.print_color_centered(
        20,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press any key to return to the menu.",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}

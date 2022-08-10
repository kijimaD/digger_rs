use super::{
    gamelog, tooltips, Attribute, Attributes, Consumable, Equipped, HungerClock, HungerState,
    InBackpack, Map, Name, Player, Point, Pools,
};
use rltk::prelude::*;
use specs::prelude::*;

/// Draw outer line
fn draw_framework(draw_batch: &mut DrawBatch) {
    let gray = RGB::named(rltk::GRAY).to_rgba(1.0);
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);

    // separators
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 79, 59), ColorPair::new(gray, black)); // Overall box
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 49, 45), ColorPair::new(gray, black)); // Map box
    draw_batch.draw_hollow_box(Rect::with_size(0, 45, 79, 14), ColorPair::new(gray, black)); // Log box
    draw_batch.draw_hollow_box(Rect::with_size(49, 0, 30, 8), ColorPair::new(gray, black)); // Top-right panel

    // connectors
    draw_batch.set(Point::new(0, 45), ColorPair::new(gray, black), to_cp437('├'));
    draw_batch.set(Point::new(49, 8), ColorPair::new(gray, black), to_cp437('├'));
    draw_batch.set(Point::new(49, 0), ColorPair::new(gray, black), to_cp437('┬'));
    draw_batch.set(Point::new(49, 45), ColorPair::new(gray, black), to_cp437('┴'));
    draw_batch.set(Point::new(79, 8), ColorPair::new(gray, black), to_cp437('┤'));
    draw_batch.set(Point::new(79, 45), ColorPair::new(gray, black), to_cp437('┤'));
}

/// Draw the town name
fn draw_map_level(ecs: &World, draw_batch: &mut DrawBatch) {
    let gray = RGB::named(rltk::GRAY).to_rgba(1.0);
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);

    let map = ecs.fetch::<Map>();
    let name_length = map.name.len() + 1;
    let x_pos = (22 - (name_length / 2)) as i32;
    draw_batch.set(Point::new(x_pos, 0), ColorPair::new(gray, black), to_cp437('┤'));
    draw_batch.set(
        Point::new(x_pos + name_length as i32, 0),
        ColorPair::new(gray, black),
        to_cp437('├'),
    );
    draw_batch.print_color(Point::new(x_pos + 1, 0), &map.name, ColorPair::new(white, black));
    std::mem::drop(map);
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, draw_batch: &mut DrawBatch) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    draw_batch.print_color(Point::new(50, y), name, ColorPair::new(attr_gray, black));
    let color: RGB = if attribute.modifiers < 0 {
        RGB::from_f32(1.0, 0.0, 0.0)
    } else if attribute.modifiers == 0 {
        RGB::named(rltk::WHITE)
    } else {
        RGB::from_f32(0.0, 1.0, 0.0)
    };
    draw_batch.print_color(
        Point::new(67, y),
        &format!("{}", attribute.base + attribute.modifiers),
        ColorPair::new(color, black),
    );
    draw_batch.print_color(
        Point::new(73, y),
        &format!("{}", attribute.bonus),
        ColorPair::new(color, black),
    );
    if attribute.bonus > 0 {
        draw_batch.set(Point::new(72, y), ColorPair::new(color, black), to_cp437('+'));
    }
}

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health =
        format!("Health:{}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
    let mana = format!("Mana:{}/{}", player_pools.mana.current, player_pools.mana.max);

    draw_batch.print_color(Point::new(50, 1), &health, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(50, 2), &mana, ColorPair::new(white, black));
    draw_batch.bar_horizontal(
        Point::new(64, 1),
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK)),
    );
    draw_batch.bar_horizontal(
        Point::new(64, 2),
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        ColorPair::new(RGB::named(rltk::BLUE), RGB::named(rltk::BLACK)),
    );

    let xp = format!("Level:  {}", player_pools.level);
    draw_batch.print_color(Point::new(50, 3), &xp, ColorPair::new(white, black));
    let xp_level_start = (player_pools.level - 1) * 1000;
    draw_batch.bar_horizontal(
        Point::new(64, 3),
        14,
        player_pools.xp - xp_level_start,
        1000,
        ColorPair::new(RGB::named(rltk::GOLD), RGB::named(rltk::BLACK)),
    );
}

fn draw_attributes(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();

    draw_attribute("Might:", &attr.might, 4, draw_batch);
    draw_attribute("Quickness:", &attr.quickness, 5, draw_batch);
    draw_attribute("Fitness:", &attr.fitness, 6, draw_batch);
    draw_attribute("Intelligence:", &attr.intelligence, 7, draw_batch);
}

fn initiative_weight(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let attributes = ecs.read_storage::<Attributes>();
    let attr = attributes.get(*player_entity).unwrap();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();

    draw_batch.print_color(
        Point::new(50, 9),
        &format!(
            "{:.0} kg ({} kg max)",
            player_pools.total_weight,
            (attr.might.base + attr.might.modifiers) * 15
        ),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(50, 10),
        &format!("Initiative Penalty: {:.0}", player_pools.total_initiative_penalty),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(50, 11),
        &format!("Gold: {:.1}", player_pools.gold),
        ColorPair::new(rltk::RGB::named(rltk::GOLD), black),
    );
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = ecs.fetch::<Entity>();

    draw_framework(&mut draw_batch);
    draw_map_level(ecs, &mut draw_batch);
    draw_stats(ecs, &mut draw_batch, &player_entity);
    draw_attributes(ecs, &mut draw_batch, &player_entity);
    initiative_weight(ecs, &mut draw_batch, &player_entity);

    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);

    let player_entity = ecs.fetch::<Entity>();

    // Equipped
    let mut y = 13;
    let equipped = ecs.read_storage::<Equipped>();
    let name = ecs.read_storage::<Name>();
    for (equipped_by, item_name) in (&equipped, &name).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(50, y, white, black, &item_name.name);
            y += 1;
        }
    }

    // Consumables
    y += 1;
    let green = RGB::from_f32(0.0, 1.0, 0.0);
    let yellow = RGB::named(rltk::YELLOW);
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (carried_by, _consumable, item_name) in (&backpack, &consumables, &name).join() {
        if carried_by.owner == *player_entity && index < 10 {
            ctx.print_color(50, y, yellow, black, &format!("↑{}", index));
            ctx.print_color(53, y, green, black, &item_name.name);
            y += 1;
            index += 1;
        }
    }

    // Hunger State
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => {
            ctx.print_color(50, 44, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Well Fed")
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            ctx.print_color(50, 44, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "Hungry")
        }
        HungerState::Starving => {
            ctx.print_color(50, 44, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "Starving")
        }
    }

    // Log
    gamelog::print_log(&mut rltk::BACKEND_INTERNAL.lock().consoles[1].console, Point::new(1, 23));

    // Tooltip
    tooltips::draw_tooltips(ecs, ctx);
}

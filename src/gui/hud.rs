use super::{
    gamelog, tooltips, Attribute, Attributes, Combatant, Consumable, Equipped, HungerClock,
    HungerState, InBackpack, Map, Name, Party, Player, Point, Pools,
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
    draw_batch.set(Point::new(0, 0), ColorPair::new(gray, black), to_cp437('-'));
    draw_batch.print_color(Point::new(1, 0), &map.name, ColorPair::new(white, black));
    draw_batch.set(
        Point::new(name_length, 0),
        ColorPair::new(gray, black),
        to_cp437('-'),
    );
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

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();

    // TODO: 表示を複数対応にする
    for (_player, _combatant, player_pools, _entity) in
        (&players, &combatants, &pools, &entities).join()
    {
        let health =
            format!("HP: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
        let sp = format!("SP: {}/{}", player_pools.sp.current, player_pools.sp.max);

        draw_batch.print_color(Point::new(50, 1), &health, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(50, 2), &sp, ColorPair::new(white, black));
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
            player_pools.sp.current,
            player_pools.sp.max,
            ColorPair::new(RGB::named(rltk::BLUE), RGB::named(rltk::BLACK)),
        );

        let xp = format!("Level: {}", player_pools.level);
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
}

fn initiative_weight(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let parties = ecs.read_storage::<Party>();
    let party = parties.get(*player_entity).unwrap();

    draw_batch.print_color(
        Point::new(50, 5),
        &format!(
            "{:.0} kg ({} kg max)",
            party.total_weight,
            100 // MEMO: 戦闘用エンティティ分割で計算できなくなったので一時的に固定
        ),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(50, 6),
        &format!("Initiative Penalty: {:.0}", party.total_initiative_penalty),
        ColorPair::new(white, black),
    );
    draw_batch.print_color(
        Point::new(50, 7),
        &format!("Gold: {:.1}", party.gold),
        ColorPair::new(rltk::RGB::named(rltk::GOLD), black),
    );
}

fn consumables(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let mut y = 9;
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);
    let green = RGB::named(rltk::GREEN);
    let entities = ecs.entities();
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let names = ecs.read_storage::<Name>();
    let mut index = 1;
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        let name = names.get(entity).unwrap();
        if carried_by.owner == *player_entity && index < 10 {
            draw_batch.print_color(
                Point::new(50, y),
                &format!("↑{}", index),
                ColorPair::new(yellow, black),
            );
            draw_batch.print_color(Point::new(53, y), &name.name, ColorPair::new(green, black));
            y += 1;
            index += 1;
        }
    }
}

fn hunger_status(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let mut y = 44;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();

    match hc.state {
        HungerState::WellFed => {
            draw_batch.print_color(
                Point::new(50, y),
                "Well Fed",
                ColorPair::new(RGB::named(rltk::GREEN), RGB::named(rltk::BLACK)),
            );
            y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(
                Point::new(50, y),
                "Hungry",
                ColorPair::new(RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK)),
            );
            y -= 1;
        }
        HungerState::Starving => {
            draw_batch.print_color(
                Point::new(50, y),
                "Starving",
                ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK)),
            );
            y -= 1;
        }
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    let player_entity = ecs.fetch::<Entity>();

    draw_map_level(ecs, &mut draw_batch);
    draw_stats(ecs, &mut draw_batch);
    initiative_weight(ecs, &mut draw_batch, &player_entity);
    consumables(ecs, &mut draw_batch, &player_entity);
    hunger_status(ecs, &mut draw_batch, &player_entity);
    gamelog::print_log(
        &crate::gamelog::FIELD_LOG,
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );
    tooltips::draw_tooltips(ecs, ctx);

    draw_batch.submit(5000); // There are 80x60(4800) possible tiles in the map.
}

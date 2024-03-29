use super::{
    gamelog, tooltips, Combatant, Consumable, HungerClock, HungerState, InBackpack, Map, Name,
    Party, Player, Point, Pools,
};
use rltk::prelude::*;
use specs::prelude::*;

const RIGHT_MENU_X: i32 = 56;

/// Draw the town name
fn draw_map_level(ecs: &World, draw_batch: &mut DrawBatch) {
    let gray = RGB::named(rltk::GRAY).to_rgba(1.0);
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);

    let y = 1;

    let map = ecs.fetch::<Map>();
    draw_batch.set(Point::new(1, y), ColorPair::new(gray, black), to_cp437('-'));
    draw_batch.print_color(Point::new(2, y), &map.name, ColorPair::new(white, black));
    draw_batch.set(Point::new(2 + map.name.len(), y), ColorPair::new(gray, black), to_cp437('-'));
    std::mem::drop(map);
}

fn draw_stats(ecs: &World, draw_batch: &mut DrawBatch) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let pools = ecs.read_storage::<Pools>();
    let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();

    let x = RIGHT_MENU_X;
    let mut y = 10;
    // TODO: 表示を複数対応にする
    for (_player, _combatant, player_pools, _entity, name) in
        (&players, &combatants, &pools, &entities, &names).join()
    {
        let level = format!("Lv. {}", player_pools.level);
        let health =
            format!("HP: {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
        let sp = format!("SP: {}/{}", player_pools.sp.current, player_pools.sp.max);

        draw_batch.print_color(Point::new(x, y), &name.name, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(x + 12, y), &level, ColorPair::new(white, black));
        y += 1;

        draw_batch.print_color(Point::new(x, y), &health, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(x + 12, y), &sp, ColorPair::new(white, black));
        y += 2;
    }
}

fn initiative_weight(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let parties = ecs.read_storage::<Party>();
    let party = parties.get(*player_entity).unwrap();
    let mut y = 1;

    draw_batch.print_color(
        Point::new(RIGHT_MENU_X, y),
        &format!(
            "{:.0} kg ({} kg max)",
            party.total_weight,
            100 // MEMO: 戦闘用エンティティ分割で計算できなくなったので一時的に固定
        ),
        ColorPair::new(white, black),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(RIGHT_MENU_X, y),
        &format!("Initiative Penalty: {:.0}", party.total_initiative_penalty),
        ColorPair::new(white, black),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(RIGHT_MENU_X, y),
        &format!("Gold: {:.1}", party.gold),
        ColorPair::new(rltk::RGB::named(rltk::GOLD), black),
    );
}

fn consumables(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let mut y = 5;
    let black = RGB::named(rltk::BLACK);
    let yellow = RGB::named(rltk::YELLOW);
    let green = RGB::named(rltk::GREEN);
    let entities = ecs.entities();
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let names = ecs.read_storage::<Name>();
    let mut index = 1;
    let margin = 3;
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        let name = names.get(entity).unwrap();
        if carried_by.owner == *player_entity && index < 10 {
            draw_batch.print_color(
                Point::new(RIGHT_MENU_X, y),
                &format!("↑{}", index),
                ColorPair::new(yellow, black),
            );
            draw_batch.print_color(
                Point::new(RIGHT_MENU_X + margin, y),
                &name.name,
                ColorPair::new(green, black),
            );
            y += 1;
            index += 1;
        }
    }
}

fn hunger_status(ecs: &World, draw_batch: &mut DrawBatch, player_entity: &Entity) {
    let x = 1;
    let y = 44;
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();

    match hc.state {
        HungerState::WellFed => {
            draw_batch.print_color(
                Point::new(x, y),
                "Well Fed",
                ColorPair::new(RGB::named(rltk::GREEN), RGB::named(rltk::BLACK)),
            );
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(
                Point::new(x, y),
                "Hungry",
                ColorPair::new(RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK)),
            );
        }
        HungerState::Starving => {
            draw_batch.print_color(
                Point::new(x, y),
                "Starving",
                ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK)),
            );
        }
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);
    let player_entity = ecs.fetch::<Entity>();

    draw_map_level(ecs, &mut draw_batch);
    initiative_weight(ecs, &mut draw_batch, &player_entity);
    consumables(ecs, &mut draw_batch, &player_entity);
    draw_stats(ecs, &mut draw_batch);
    hunger_status(ecs, &mut draw_batch, &player_entity);
    tooltips::draw_tooltips(ecs, ctx);
    tooltips::draw_item_tooltips(ecs, ctx);

    let _ = draw_batch.submit(5000); // There are 80x60(4800) possible tiles in the map.

    gamelog::print_log(
        &crate::gamelog::FIELD_LOG,
        &mut rltk::BACKEND_INTERNAL.lock().consoles[2].console,
        Point::new(1, 46),
    );
}

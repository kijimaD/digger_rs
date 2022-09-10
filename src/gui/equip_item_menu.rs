use super::{
    item_result_menu, Attributes, Combatant, Equippable, Equipped, InBackpack, ItemMenuResult,
    Name, Player, Pools, State,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn equip_item_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    entity: Entity,
) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    draw_status(&gs.ecs, &mut draw_batch, entity);
    draw_character_selector(&gs.ecs, &mut draw_batch, entity);

    let names = gs.ecs.read_storage::<Name>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    // 装備スロット
    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &equipped).join().filter(|item| item.1.owner == entity).for_each(|item| {
        let name = names.get(item.0).unwrap();
        items.push((item.0, name.name.clone()))
    });

    let result =
        item_result_menu(&mut draw_batch, "Equipped", items.len(), &items, ctx.key, Some(4), None);

    let _ = draw_batch.submit(6000);
    result
}

fn draw_status(ecs: &World, draw_batch: &mut DrawBatch, entity: Entity) {
    draw_batch.draw_box(
        Rect::with_size(40, 20, 31, 12),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let names = ecs.read_storage::<Name>();
    let attribute = attributes.get(entity).unwrap();
    let pools = pools.get(entity).unwrap();
    let name = names.get(entity).unwrap();

    let x = 41;
    let mut y = 21;
    let bar_x = x + 12;
    let bar_width = 14;

    // 左側の装備スロット
    // TODO: 4つの装備スロットを表示する。同じ部位の防具は同時に装備できない
    // ダガー
    // 鉄の鎧
    // 空き
    // 空き

    // 下部の選択中のメンバーのステータス
    let name_text = format!("{}", name.name);
    let level = format!("LV: {}", pools.level);
    let next_level = format!("Until Next Level: {} xp", (pools.level * 1000 - pools.xp));
    let hp = format!("HP: {}/{}", pools.hit_points.current, pools.hit_points.max);
    let sp = format!("SP: {}/{}", pools.sp.current, pools.sp.max);
    let might = format!(
        "Might: {}({})",
        attribute.might.base + attribute.might.modifiers + attribute.might.bonus,
        attribute.might.base,
    );
    let fitness = format!(
        "Fitness: {}({})",
        attribute.fitness.base + attribute.fitness.modifiers + attribute.fitness.bonus,
        attribute.fitness.base
    );
    let quickness = format!(
        "Quickness: {}({})",
        attribute.quickness.base + attribute.quickness.modifiers + attribute.quickness.bonus,
        attribute.quickness.base
    );
    let intelligence = format!(
        "intelligence: {}({})",
        attribute.intelligence.base
            + attribute.intelligence.modifiers
            + attribute.intelligence.bonus,
        attribute.intelligence.base
    );

    draw_batch.print_color(
        Point::new(x, y),
        name_text,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        level,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        next_level,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;

    draw_batch.print_color(
        Point::new(x, y),
        &hp,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.bar_horizontal(
        Point::new(bar_x, y),
        bar_width,
        pools.hit_points.current,
        pools.hit_points.max,
        ColorPair::new(RGB::named(rltk::GREEN), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        &sp,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.bar_horizontal(
        Point::new(bar_x, y),
        bar_width,
        pools.sp.current,
        pools.sp.max,
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );
    y += 2;

    draw_batch.print_color(
        Point::new(x, y),
        might,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        fitness,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        quickness,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    y += 1;
    draw_batch.print_color(
        Point::new(x, y),
        intelligence,
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
}

fn draw_character_selector(ecs: &World, draw_batch: &mut DrawBatch, entity: Entity) {
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();
    let mut x = 4;
    let mut y = 4;

    draw_batch.draw_box(
        Rect::with_size(x, y, 40, 4),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    x += 1;
    y += 2;

    for (inloop_entity, _combatant, _player, name) in
        (&entities, &combatants, &players, &names).join()
    {
        if inloop_entity == entity {
            draw_batch.print_color(
                Point::new(x, y - 1),
                format!("↓"),
                ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            );
        }

        draw_batch.print_color(
            Point::new(x, y),
            &name.name,
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        x += name.name.len() + 1;
    }
    y += 1;

    // TODO: アイテム数がp, nまで達するようになると、機能しなくなる
    draw_batch.print_color(
        Point::new(5, y),
        format!("p <-{}-> n", " ".to_string().repeat(31)),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
}

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentMenuResult {
    Next,
    Prev,
    NoResponse,
}

pub fn equipment_key_move(
    gs: &State,
    ctx: &Rltk,
    entity: &Entity,
    index: i32,
) -> (EquipmentMenuResult, Entity, i32) {
    match ctx.key {
        None => (EquipmentMenuResult::NoResponse, *entity, index),
        Some(key) => match key {
            VirtualKeyCode::N => {
                let result = increment_entity(&gs.ecs, &entity, index, 1);
                (EquipmentMenuResult::Next, result.0, result.1)
            }
            VirtualKeyCode::P => {
                let result = increment_entity(&gs.ecs, &entity, index, -1);
                (EquipmentMenuResult::Prev, result.0, result.1)
            }
            _ => (EquipmentMenuResult::NoResponse, *entity, index),
        },
    }
}

fn increment_entity(ecs: &World, old_entity: &Entity, index: i32, direction: i32) -> (Entity, i32) {
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let entities = ecs.entities();

    let party_count = (&entities, &combatants, &players).join().count();
    if (direction == -1 && index <= 0) || (direction == 1 && index >= party_count as i32) {
        return (*old_entity, index);
    }

    for (entity, _combatant, _player) in
        (&entities, &combatants, &players).join().nth((index + direction) as usize)
    {
        return (entity, index + direction);
    }
    (*old_entity, index)
}

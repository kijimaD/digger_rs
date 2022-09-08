use super::{
    item_result_menu, Attributes, Combatant, Equippable, Equipped, InBackpack, ItemMenuResult,
    Name, Player, Pools, State,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn equip_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    draw_batch.draw_box(
        Rect::with_size(40, 20, 31, 10),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    let players = gs.ecs.read_storage::<Player>();
    let combatants = gs.ecs.read_storage::<Combatant>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let attributes = gs.ecs.read_storage::<Attributes>();
    let pools = gs.ecs.read_storage::<Pools>();
    let entities = gs.ecs.entities();

    // 左側の装備スロット
    // TODO: 4つの装備スロットを表示する。同じ部位の防具は同時に装備できない
    // ダガー
    // 鉄の鎧
    // 空き
    // 空き

    let mut items: Vec<(Entity, String)> = Vec::new();
    let x = 41;
    let mut y = 21;
    let bar_x = x + 12;
    let bar_width = 14;

    // TODO: まだパーティ最初の一人固定なので、選択できるようにする
    for (entity, _combatant, _player, attribute, pools) in
        (&entities, &combatants, &players, &attributes, &pools).join().take(1)
    {
        (&entities, &equipped).join().filter(|item| item.1.owner == entity).for_each(|item| {
            let name = names.get(item.0).unwrap();
            items.push((item.0, name.name.clone()))
        });

        // 下部の選択中のメンバーのステータス
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
    let result =
        item_result_menu(&mut draw_batch, "Equipped", items.len(), &items, ctx.key, Some(4), None);

    draw_batch.submit(6000);
    result
}

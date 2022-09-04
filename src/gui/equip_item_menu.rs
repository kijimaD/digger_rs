use super::{item_result_menu, Equippable, InBackpack, ItemMenuResult, Name, State};
use rltk::prelude::*;
use specs::prelude::*;

pub fn equip_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    // 右側の装備スロット
    draw_batch.draw_box(
        Rect::with_size(50, 15, 20, 4),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    // 左側のインベントリ
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let equippables = gs.ecs.read_storage::<Equippable>();
    let entities = gs.ecs.entities();

    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &backpack).join().filter(|item| item.1.owner == *player_entity).for_each(|item| {
        let name = names.get(item.0).unwrap();
        items.push((item.0, name.name.clone()))
    });

    let result = item_result_menu(
        &mut draw_batch,
        "Equip which item?",
        items.len(),
        &items,
        ctx.key,
        None,
        None,
    );
    draw_batch.submit(6000);
    result
}

use super::{
    item_result_menu, Combatant, Equippable, Equipped, InBackpack, ItemMenuResult, Name, Player,
    State,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn equip_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    let players = gs.ecs.read_storage::<Player>();
    let combatants = gs.ecs.read_storage::<Combatant>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    // 左側の装備スロット
    // TODO: 4つのスロットを作る。↓みたいに
    // ダガー
    // 空き
    // 空き
    // 空き
    let mut items: Vec<(Entity, String)> = Vec::new();
    // TODO: まだパーティ最初の一人固定なので、選択できるようにする
    for (entity, _combatant, _player) in (&entities, &combatants, &players).join().take(1) {
        (&entities, &equipped).join().filter(|item| item.1.owner == entity).for_each(|item| {
            let name = names.get(item.0).unwrap();
            items.push((item.0, name.name.clone()))
        });
    }
    let result =
        item_result_menu(&mut draw_batch, "Equipped", items.len(), &items, ctx.key, Some(4), None);

    // 下部の選択中のメンバーのステータス
    // TODO: 未実装
    draw_batch.draw_box(
        Rect::with_size(40, 20, 31, 6),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(41, 21),
        "party member status",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    draw_batch.submit(6000);
    result
}

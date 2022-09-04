use super::{
    item_result_menu, Combatant, Equippable, Equipped, InBackpack, ItemMenuResult, Name, Player,
    State,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn equip_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    let player_entity = gs.ecs.fetch::<Entity>();
    let players = gs.ecs.read_storage::<Player>();
    let combatants = gs.ecs.read_storage::<Combatant>();
    let names = gs.ecs.read_storage::<Name>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    // 右側の装備スロット
    let mut items: Vec<(Entity, String)> = Vec::new();
    // TODO: まだパーティ最初の一人固定なので選択できるようにする
    for (entity, _combatant, _player) in (&entities, &combatants, &players).join().take(1) {
        (&entities, &equipped).join().filter(|item| item.1.owner == entity).for_each(|item| {
            let name = names.get(item.0).unwrap();
            items.push((item.0, name.name.clone()))
        });
    }
    item_result_menu(&mut draw_batch, "Equipped", items.len(), &items, ctx.key, Some(40), None);

    // 左側のインベントリ
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let equippables = gs.ecs.read_storage::<Equippable>();

    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &backpack, &equippables)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .for_each(|item| {
            let name = names.get(item.0).unwrap();
            items.push((item.0, name.name.clone()))
        });

    let result =
        item_result_menu(&mut draw_batch, "Inventory", items.len(), &items, ctx.key, Some(4), None);
    draw_batch.submit(6000);
    result
}

use super::{item_result_menu, Equipped, ItemMenuResult, Name, State};
use rltk::prelude::*;
use specs::prelude::*;

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();

    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &backpack).join().filter(|item| item.1.owner == *player_entity).for_each(|item| {
        let name = names.get(item.0).unwrap();
        items.push((item.0, name.name.clone()))
    });

    let result = item_result_menu(
        &mut draw_batch,
        "Remove which item?",
        items.len(),
        &items,
        ctx.key,
        None,
        None,
    );

    let _ = draw_batch.submit(6000);
    result
}

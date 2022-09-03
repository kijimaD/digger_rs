use super::{item_result_menu, Consumable, InBackpack, Name, State};
use rltk::prelude::*;
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_use_item(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let consumables = gs.ecs.read_storage::<Consumable>();
    let entities = gs.ecs.entities();

    let mut draw_batch = DrawBatch::new();

    let mut items: Vec<(Entity, String)> = Vec::new();
    (&entities, &backpack, &consumables)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .for_each(|item| {
            let name = names.get(item.0).unwrap();
            items.push((item.0, name.name.clone()))
        });
    let result = item_result_menu(&mut draw_batch, "Use Item", items.len(), &items, ctx.key);

    draw_batch.submit(6000);
    result
}

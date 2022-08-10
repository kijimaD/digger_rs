use super::{show_inventory, Name, Player, Pools, State};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_item_targeting(
    gs: &mut State,
    ctx: &mut Rltk,
) -> (ItemTargetingResult, Option<Entity>) {
    // とりあえず味方だけ
    // TODO: targetが必要ないもの…食料とか…では表示しないようにしたい
    show_inventory(gs, ctx);

    let entities = gs.ecs.entities();
    let pools = gs.ecs.read_storage::<Pools>();
    let name = gs.ecs.read_storage::<Name>();
    let player = gs.ecs.read_storage::<Player>();

    let count = (&pools, &player).join().count();
    let mut y = (25 - (count / 2)) as i32;
    let mut j = 0;
    let mut targets: Vec<Entity> = Vec::new();

    for (entity, _pools, name) in (&entities, &pools, &name).join() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));
        ctx.print(21, y, &name.name.to_string());

        targets.push(entity);

        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemTargetingResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemTargetingResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemTargetingResult::Selected, Some(targets[selection as usize]));
                }
                (ItemTargetingResult::NoResponse, None)
            }
        },
    }
}

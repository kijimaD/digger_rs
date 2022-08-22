use super::{show_inventory, Name, Player, Pools, State};
use rltk::prelude::*;
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
    let mut draw_batch = DrawBatch::new();

    let entities = gs.ecs.entities();
    let pools = gs.ecs.read_storage::<Pools>();
    let name = gs.ecs.read_storage::<Name>();
    let player = gs.ecs.read_storage::<Player>();

    let count = (&pools, &player).join().count();
    let mut y = (25 - (count / 2)) as i32;
    let mut j = 0;
    let mut targets: Vec<Entity> = Vec::new();

    // とりあえず味方用アイテムだけ
    // TODO: アイテムによって対象選択画面を変えたい。敵単体や味方単体の場合には対象選択を表示し、敵全体、味方全体の場合は表示しない
    for (entity, _pools, name, player) in (&entities, &pools, &name, &player).join() {
        targets.push(entity);

        draw_batch.draw_box(
            Rect::with_size(15, y - 2, 31, (count + 3) as i32),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        draw_batch.print_color(
            Point::new(18, y - 2),
            "Select target",
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        );
        draw_batch.set(
            Point::new(17, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437('('),
        );
        draw_batch.set(
            Point::new(18, y),
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            97 + j as rltk::FontCharType,
        );
        draw_batch.set(
            Point::new(19, y),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437(')'),
        );
        draw_batch.print_color(
            Point::new(21, y),
            &name.name,
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        );
        draw_batch.print_color(
            Point::new(18, y + count as i32 + 1),
            "ESCAPE to cancel",
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        );

        y += 1;
        j += 1;
    }

    draw_batch.submit(6000);

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

use super::{OnBattle, State};
use rltk::prelude::*;
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum BattleWinResult {
    NoResponse,
    Enter,
}

pub fn show_battle_result(gs: &mut State, ctx: &mut Rltk) -> BattleWinResult {
    let mut draw_batch = DrawBatch::new();

    let on_battles = gs.ecs.read_storage::<OnBattle>();
    for on_battle in (&on_battles).join() {
        let xp = on_battle.xp.to_string();
        let gold = on_battle.gold.to_string();

        let x = 18;
        let mut y = 30;

        draw_batch.draw_box(
            Rect::with_size(15, y, 31, 6),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        y += 1;

        draw_batch.print_color(
            Point::new(x, y),
            format!("{} xp", xp),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        y += 1;

        draw_batch.print_color(
            Point::new(x, y),
            format!("{} gold", gold),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        y += 1;
    }

    draw_batch.print_color(
        Point::new(70, 44),
        "[Enter]",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleWinResult::NoResponse),
        Some(key) => match key {
            VirtualKeyCode::Return => (BattleWinResult::Enter),
            _ => BattleWinResult::NoResponse,
        },
    }
}

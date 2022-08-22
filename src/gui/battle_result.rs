use super::State;
use rltk::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum BattleResult {
    NoResponse,
    Enter,
}

pub fn show_battle_result(_gs: &mut State, ctx: &mut Rltk) -> BattleResult {
    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color(
        Point::new(70, 44),
        "[Enter]",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleResult::NoResponse),
        Some(key) => match key {
            VirtualKeyCode::Return => (BattleResult::Enter),
            _ => BattleResult::NoResponse,
        },
    }
}

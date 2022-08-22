use super::run_away_system;
use rltk::prelude::*;
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum BattleCommandResult {
    NoResponse,
    Attack,
    ShowInventory,
    RunAway,
    RunAwayFailed,
}

pub fn show_battle_command(ecs: &mut World, ctx: &mut Rltk) -> BattleCommandResult {
    let mut draw_batch = DrawBatch::new();

    let y = 30;

    draw_batch.draw_box(
        Rect::with_size(15, y - 2, 31, 6 as i32),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(16, y),
        "[a] Attack",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(16, y + 1),
        "[i] Inventory",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(16, y + 2),
        "[r] Run away",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    draw_batch.submit(5000);

    match ctx.key {
        None => BattleCommandResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::A => BattleCommandResult::Attack,
            VirtualKeyCode::I => BattleCommandResult::ShowInventory,
            VirtualKeyCode::R => match run_away_system::run_away_roll(ecs) {
                run_away_system::RunAwayResult::Success => return BattleCommandResult::RunAway,
                run_away_system::RunAwayResult::Fail => return BattleCommandResult::RunAwayFailed,
            },
            _ => BattleCommandResult::NoResponse,
        },
    }
}

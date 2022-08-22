use super::{gamelog, run_away_system, Combatant, Monster, Name, Pools, State};
use rltk::prelude::*;
use specs::prelude::*;

/// すべての戦闘系stateで共通表示するUI
pub fn draw_battle_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    // ログ
    draw_batch.draw_box(
        Rect::with_size(0, 45, 79, 14),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    gamelog::print_log(
        &crate::gamelog::BATTLE_LOG,
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );

    // 敵一覧
    let names = ecs.read_storage::<Name>();
    let pools = ecs.read_storage::<Pools>();
    let monsters = ecs.read_storage::<Monster>();
    let combatants = ecs.read_storage::<Combatant>();

    let mut i = 1;
    for (name, pools, _combatant, _monster) in (&names, &pools, &combatants, &monsters).join() {
        draw_batch.print_color(
            Point::new((80 * i) / (1 + combatants.count()), 20),
            format!("[{}]({})", name.name, pools.hit_points.current),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        i += 1;
    }

    draw_batch.submit(5000);
}

#[derive(PartialEq, Copy, Clone)]
pub enum BattleCommandResult {
    NoResponse,
    Attack,
    ShowInventory,
    RunAway,
    RunAwayFailed,
}

pub fn battle_command(ecs: &mut World, ctx: &mut Rltk) -> BattleCommandResult {
    let mut draw_batch = DrawBatch::new();

    let y = 30;
    draw_batch.print_color(
        Point::new(2, y),
        "[a] Attack",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(2, y + 1),
        "[i] Inventory",
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    draw_batch.print_color(
        Point::new(2, y + 2),
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

pub enum BattleTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn battle_target(gs: &mut State, ctx: &mut Rltk) -> (BattleTargetingResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();

    let entities = gs.ecs.entities();
    let pools = gs.ecs.write_storage::<Pools>();
    let monsters = gs.ecs.read_storage::<Monster>();
    let combatants = gs.ecs.read_storage::<Combatant>();

    let mut x = 1;
    let mut j = 0;

    let mut targets: Vec<Entity> = Vec::new();
    for (entity, _pools, _combatant, _monster) in (&entities, &pools, &combatants, &monsters).join()
    {
        let base = 2 + (80 * x) / (1 + pools.count());
        draw_batch.set(
            Point::new(base + 0, 22),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437('('),
        );
        draw_batch.set(
            Point::new(base + 1, 22),
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            97 + j as rltk::FontCharType,
        );
        draw_batch.set(
            Point::new(base + 2, 22),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437(')'),
        );

        targets.push(entity);
        x += 1;
        j += 1;
    }

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleTargetingResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (BattleTargetingResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < targets.len() as i32 {
                    return (BattleTargetingResult::Selected, Some(targets[selection as usize]));
                }
                (BattleTargetingResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum BattleResult {
    NoResponse,
    Enter,
}

pub fn show_battle_win_result(_gs: &mut State, ctx: &mut Rltk) -> BattleResult {
    let mut draw_batch = DrawBatch::new();

    draw_batch.print_color(Point::new(70, 44), "[Enter]", ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)));

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleResult::NoResponse),
        Some(key) => match key {
            VirtualKeyCode::Return => (BattleResult::Enter),
            _ => BattleResult::NoResponse,
        },
    }
}

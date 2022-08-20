use super::{
    gamelog, show_inventory, Combatant, Consumable, InBackpack, ItemMenuResult,
    Monster, Name, OnBattle, Pools, State,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn show_battle_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_inventory(gs, ctx);

    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let consumable = gs.ecs.read_storage::<Consumable>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();
    let mut y = (25 - (count / 2)) as i32;

    // 戦闘中は消費アイテムしか使えない
    let mut useable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name, _consumable) in (&entities, &backpack, &names, &consumable)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
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
        useable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(useable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn draw_battle_ui(ecs: &World, ctx: &mut Rltk) {
    // メッセージボックス
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

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
        ctx.print(
            (80 * i) / (1 + combatants.count()),
            20,
            format!("[{}]({})", name.name, pools.hit_points.current),
        );
        i += 1;
    }
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
    let y = 30;
    ctx.print(2, y, "[a] Attack");
    ctx.print(2, y + 1, "[i] Inventory");
    ctx.print(2, y + 2, "[r] Run away");

    match ctx.key {
        None => BattleCommandResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::A => BattleCommandResult::Attack,
            VirtualKeyCode::I => BattleCommandResult::ShowInventory,
            VirtualKeyCode::R => {
                let mut rng = RandomNumberGenerator::new();
                let num = rng.range(0, 2);
                if num == 0 {
                    // 逃走成功
                    run_away_battle(ecs);
                    return BattleCommandResult::RunAway;
                } else {
                    // 逃走失敗
                    // log.entries.push(format!("Failed run away!"));
                    return BattleCommandResult::RunAwayFailed;
                }
            }
            _ => BattleCommandResult::NoResponse,
        },
    }
}

// 逃走。
// 敵シンボルは消さずに、戦闘用エンティティだけ削除する
// TODO: このファイルにあるべき関数ではない
fn run_away_battle(ecs: &mut World) {
    let combatants = ecs.write_storage::<Combatant>();
    let monsters = ecs.read_storage::<Monster>();
    let entities = ecs.entities();

    for (entity, _combatant, _monster) in (&entities, &combatants, &monsters).join() {
        entities.delete(entity).expect("Delete failed")
    }

    // battle削除
    let mut battle = ecs.write_storage::<OnBattle>();
    battle.clear();

    // log.entries.push(format!("Run away!"));
}

pub enum BattleTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn battle_target(gs: &mut State, ctx: &mut Rltk) -> (BattleTargetingResult, Option<Entity>) {
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
        ctx.set(
            base + 0,
            22,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            base + 1,
            22,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            base + 2,
            22,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        targets.push(entity);
        x += 1;
        j += 1;
    }

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
    ctx.print_color(70, 44, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "[Enter]");

    match ctx.key {
        None => (BattleResult::NoResponse),
        Some(key) => match key {
            VirtualKeyCode::Return => (BattleResult::Enter),
            _ => BattleResult::NoResponse,
        },
    }
}

use super::{Combatant, Monster, Pools, State};
use rltk::prelude::*;
use specs::prelude::*;

pub enum BattleAttackTargetingResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_attack_target(
    gs: &mut State,
    ctx: &mut Rltk,
) -> (BattleAttackTargetingResult, Option<Entity>) {
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
            rltk::to_cp437('['),
        );
        draw_batch.set(
            Point::new(base + 1, 22),
            ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
            97 + j as rltk::FontCharType,
        );
        draw_batch.set(
            Point::new(base + 2, 22),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            rltk::to_cp437(']'),
        );

        targets.push(entity);
        x += 1;
        j += 1;
    }

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleAttackTargetingResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (BattleAttackTargetingResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < targets.len() as i32 {
                    return (
                        BattleAttackTargetingResult::Selected,
                        Some(targets[selection as usize]),
                    );
                }
                (BattleAttackTargetingResult::NoResponse, None)
            }
        },
    }
}

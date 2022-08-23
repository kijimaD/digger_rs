use super::{Equipped, MeleeWeapon, Name, State};
use rltk::prelude::*;
use specs::prelude::*;

pub enum BattleAttackWayResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_attack_way(gs: &mut State, ctx: &mut Rltk) -> (BattleAttackWayResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();
    let mut targets: Vec<Entity> = Vec::new();

    let entities = gs.ecs.entities();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let weapon = gs.ecs.read_storage::<MeleeWeapon>();
    let names = gs.ecs.read_storage::<Name>();
    let player_entity = gs.ecs.fetch::<Entity>();

    let base_x = 18;
    let mut y = 32;
    let mut j = 0;

    draw_batch.draw_box(
        Rect::with_size(15, y - 2, 31, 6),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );

    for (entity, equipped_by, _weapon) in (&entities, &equipped, &weapon).join() {
        if equipped_by.owner == *player_entity {
            let name = names.get(entity).unwrap();
            draw_batch.print_color(
                Point::new(base_x + 3, y),
                &name.name,
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            );

            draw_batch.set(
                Point::new(base_x - 1, y),
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
                rltk::to_cp437('['),
            );
            draw_batch.set(
                Point::new(base_x, y),
                ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
                97 + j as rltk::FontCharType,
            );
            draw_batch.set(
                Point::new(base_x + 1, y),
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
                rltk::to_cp437(']'),
            );
            y += 1;
            j += 1;
            targets.push(entity);
        }
    }

    draw_batch.submit(5000);

    match ctx.key {
        None => (BattleAttackWayResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (BattleAttackWayResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < targets.len() as i32 {
                    return (BattleAttackWayResult::Selected, Some(targets[selection as usize]));
                }
                (BattleAttackWayResult::NoResponse, None)
            }
        },
    }
}

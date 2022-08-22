use super::{menu_box, menu_option};
use crate::State;
use rltk::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
    Heal,
    Reveal,
    GodMode,
    SpawnMonster { monster_x: i32, monster_y: i32 },
}

pub fn show_cheat_mode(gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let mut draw_batch = DrawBatch::new();
    let count = 5;
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count + 3) as i32, "Cheating!");
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );

    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('T'), "Teleport to next level");
    y += 1;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('H'), "Heal all wounds");
    y += 1;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('R'), "Reveal the map");
    y += 1;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('G'), "God Mode (No Death)");
    y += 1;
    menu_option(&mut draw_batch, 17, y, rltk::to_cp437('S'), "Spawn Monster");

    draw_batch.submit(6000);

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::R => CheatMenuResult::Reveal,
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::S => {
                let player_pos = gs.ecs.fetch::<Point>();
                CheatMenuResult::SpawnMonster {
                    monster_x: player_pos.x + 2,
                    monster_y: player_pos.y,
                }
            }
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}

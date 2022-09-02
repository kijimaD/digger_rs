use super::{RunState, State};
use crate::rex_assets::RexAssets;
use rltk::prelude::*;
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    ctx.set_active_console(2);
    let runstate = gs.ecs.fetch::<RunState>();
    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    let mut draw_batch = DrawBatch::new();
    let save_exists = crate::saveload_system::does_save_exist();
    let x = 8;
    let mut y = 20;

    draw_batch.print_color(
        Point::new(x, y - 4),
        "Diggers",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );
    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            draw_batch.print_color(
                Point::new(x, y),
                "Begin New Game",
                ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)),
            );
        } else {
            draw_batch.print_color(
                Point::new(x, y),
                "Begin New Game",
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            );
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                draw_batch.print_color(
                    Point::new(x, y),
                    "Load Game",
                    ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)),
                );
            } else {
                draw_batch.print_color(
                    Point::new(x, y),
                    "Load Game",
                    ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
                );
            }
            y += 1
        }

        if selection == MainMenuSelection::Quit {
            draw_batch.print_color(
                Point::new(x, y),
                "Quit",
                ColorPair::new(RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK)),
            );
        } else {
            draw_batch.print_color(
                Point::new(x, y),
                "Quit",
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
            );
        }

        draw_batch.submit(6000);

        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit }
                }
                VirtualKeyCode::Up => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection { selected: newselection };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection { selected: newselection };
                }
                VirtualKeyCode::Return => return MainMenuResult::Selected { selected: selection },
                _ => return MainMenuResult::NoSelection { selected: selection },
            },
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

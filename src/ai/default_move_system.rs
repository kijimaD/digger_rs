use crate::{EntityMoved, Map, MoveMode, Movement, MyTurn, Position, Viewshed};
use specs::prelude::*;

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            move_mode,
            mut positions,
            mut map,
            mut viewsheds,
            mut entity_moved,
            mut rng,
            entities,
        ) = data;
        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, mut pos, mode, mut viewshed, _myturn) in
            (&entities, &mut positions, &move_mode, &mut viewsheds, &turns).join()
        {
            turn_done.push(entity);

            match mode.mode {
                Movement::Static => {}
                Movement::Random => {
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !map.blocked[dest_idx] {
                            let idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = false;
                            pos.x = x;
                            pos.y = y;
                            entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert");
                            map.blocked[dest_idx] = true;
                            viewshed.dirty = true;
                        }
                    }
                }
            }
        }

        // Remove marker
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

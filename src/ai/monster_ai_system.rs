use crate::{Map, Monster, MyTurn, Position, RunState, Viewshed};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, MyTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, runstate, entities, mut viewshed, monster, mut position, turns) =
            data;

        for (entity, mut viewshed, _monster, mut pos, turn) in
            (&entities, &mut viewshed, &monster, &mut position, &turns).join()
        {
            // let can_act = true;

            // if can_act {
            //     let distance =
            //         rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            //     if viewshed.visible_tiles.contains(&*player_pos) {
            //         // Path to the player
            //         let path = rltk::a_star_search(
            //             map.xy_idx(pos.x, pos.y),
            //             map.xy_idx(player_pos.x, player_pos.y),
            //             &*map,
            //         );
            //         if path.success && path.steps.len() > 1 {
            //             let mut idx = map.xy_idx(pos.x, pos.y);
            //             map.blocked[idx] = false;
            //             pos.x = path.steps[1] as i32 % map.width;
            //             pos.y = path.steps[1] as i32 / map.width;
            //             idx = map.xy_idx(pos.x, pos.y);
            //             map.blocked[idx] = true;
            //             viewshed.dirty = true;
            //         }
            //     }
            // }
        }
    }
}

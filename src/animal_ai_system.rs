use super::{
    particle_system::ParticleBuilder, EntityMoved, Herbivore, Item, Map, Position, RunState,
    Viewshed, WantsToMelee,
};
use rltk::Point;
use specs::prelude::*;

pub struct AnimalAI {}

impl<'a> System<'a> for AnimalAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Herbivore>,
        ReadStorage<'a, Item>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            herbivore,
            item,
            mut wants_to_melee,
            mut entity_moved,
            mut position,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        // Herbivores run away a lot
        for (entity, mut viewshed, _herbivore, mut pos) in
            (&entities, &mut viewshed, &herbivore, &mut position).join()
        {
            let mut run_away_from: Vec<usize> = Vec::new();
            for other_tile in viewshed.visible_tiles.iter() {
                let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                for other_entity in map.tile_content[view_idx].iter() {
                    // Then don't run away from items
                    if item.get(*other_entity).is_none() {
                        run_away_from.push(view_idx);
                    }
                }
            }

            if !run_away_from.is_empty() {
                let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let flee_map = rltk::DijkstraMap::new(
                    map.width as usize,
                    map.height as usize,
                    &run_away_from,
                    &*map,
                    100.0,
                );
                let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
                if let Some(flee_target) = flee_target {
                    if !map.blocked[flee_target as usize] {
                        map.blocked[my_idx] = false;
                        map.blocked[flee_target as usize] = true;
                        viewshed.dirty = true;
                        pos.x = flee_target as i32 % map.width;
                        pos.y = flee_target as i32 / map.width;
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                    }
                }
            }
        }
    }
}

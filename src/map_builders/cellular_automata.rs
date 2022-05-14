use super::{
    apply_room_to_map, generate_voronoi_spawn_regions,
    remove_unreachable_areas_returning_most_distant, spawner, BuilderMap, InitialMapBuilder, Map,
    Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 {
                    build_data.map.tiles[idx] = TileType::Floor
                } else {
                    build_data.map.tiles[idx] = TileType::Wall
                }
            }
        }
        build_data.take_snapshot();

        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height - 1 {
                for x in 1..build_data.map.width - 1 {
                    let idx = build_data.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - (build_data.map.width as usize - 1)]
                        == TileType::Wall
                    {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - (build_data.map.width as usize + 1)]
                        == TileType::Wall
                    {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + (build_data.map.width as usize - 1)]
                        == TileType::Wall
                    {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + (build_data.map.width as usize + 1)]
                        == TileType::Wall
                    {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}

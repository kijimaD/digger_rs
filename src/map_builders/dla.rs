use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, spawner, Map, MapBuilder,
    Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

#[derive(PartialEq, Copy, Clone)]
pub enum DLASymmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: DLASymmetry,
    floor_percent: f32,
}

impl MapBuilder for DLABuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }
    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl DLABuilder {
    pub fn new(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Carve a starting seed
        self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        // while floor_tile_count  < desired_floor_tiles {

        //     match self.algorithm {
        //         DLAAlgorithm::WalkInwards => {
        //             let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
        //             let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
        //             let mut prev_x = digger_x;
        //             let mut prev_y = digger_y;
        //             let mut digger_idx = self.map.xy_idx(digger_x, digger_y);
        //             while self.map.tiles[digger_idx] == TileType::Wall {
        //                 prev_x = digger_x;
        //                 prev_y = digger_y;
        //                 let stagger_direction = rng.roll_dice(1, 4);
        //                 match stagger_direction {
        //                     1 => { if digger_x > 2 { digger_x -= 1; } }
        //                     2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
        //                     3 => { if digger_y > 2 { digger_y -=1; } }
        //                     _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
        //                 }
        //                 digger_idx = self.map.xy_idx(digger_x, digger_y);
        //             }
        //             self.paint(prev_x, prev_y);
        //         }
        //         _ => {}
        //     }
        // }
    }

    fn paint(&mut self, x: i32, y: i32) {
        let digger_idx = self.map.xy_idx(x, y);
        self.map.tiles[digger_idx] = TileType::Floor;
    }
}

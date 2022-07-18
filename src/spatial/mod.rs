use crate::{map::tile_walkable, Map};
use specs::prelude::*;
use std::sync::Mutex;

struct SpatialMap {
    blocked: Vec<bool>,
    tile_content: Vec<Vec<Entity>>,
}

impl SpatialMap {
    fn new() -> Self {
        Self { blocked: Vec::new(), tile_content: Vec::new() }
    }
}

lazy_static! {
    static ref SPATIAL_MAP: Mutex<SpatialMap> = Mutex::new(SpatialMap::new());
}

pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![false; map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}

pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.clear();
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}

pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i, tile) in map.tiles.iter().enumerate() {
        lock.blocked[i] = !tile_walkable(*tile);
    }
}

pub fn index_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].push(entity);
}

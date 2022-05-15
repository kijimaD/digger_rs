use super::{BuilderMap, MetaMapBuilder, Rect, TileType};
use rltk::RandomNumberGenerator;

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data)
    }
}

impl RoomDrawer {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomDrawer> {
        Box::new(RoomDrawer {})
    }

    fn build(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Rounding require a builder with room structures");
        }

        for room in rooms.iter() {
            for y in room.y1 + 1..=room.y2 {
                for x in room.x1 + 1..=room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0
                        && idx < ((build_data.map.width * build_data.map.height) - 1) as usize
                    {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }
    }
}

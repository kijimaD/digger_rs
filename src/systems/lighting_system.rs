use crate::{LightSource, Map, Position, Viewshed};
use rltk::RGB;
use specs::prelude::*;

pub struct LightingSystem {}

impl<'a> System<'a> for LightingSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightSource>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, viewshed, positions, lighting) = data;

        if map.outdoors {
            return;
        }

        let black = RGB::from_f32(0.3, 0.3, 0.3);
        for l in map.light.iter_mut() {
            *l = black;
        }

        for (viewshed, pos, light) in (&viewshed, &positions, &lighting).join() {
            let light_point = rltk::Point::new(pos.x, pos.y);
            let range_f = light.range as f32;
            for t in viewshed.visible_tiles.iter() {
                if t.x > 0 && t.x < map.width && t.y > 0 && t.y < map.height {
                    let idx = map.xy_idx(t.x, t.y);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(light_point, *t);
                    let intensity = (range_f - distance * 1.6) / range_f + 0.8;

                    map.light[idx] = map.light[idx] + (light.color * intensity);
                }
            }
        }
    }
}

use crate::map::tile_glyph;
use crate::{Map, Position, Renderable};
use rltk::prelude::*;
use specs::prelude::*;

const SHOW_BOUNDARIES: bool = false;

pub fn render_camera(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    // 地形
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    let map_width = map.width;
    let map_height = map.height;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
                    draw_batch.set(Point::new(x, y), ColorPair::new(fg, bg), glyph);
                }
            } else if SHOW_BOUNDARIES {
                draw_batch.set(
                    Point::new(x, y),
                    ColorPair::new(RGB::named(rltk::GRAY), RGB::named(rltk::BLACK)),
                    to_cp437('·'),
                );
            }
            x += 1;
        }
        y += 1;
    }

    let _ = draw_batch.submit(1000);

    draw_batch.target(1);

    // キャラ
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let map = ecs.fetch::<Map>();

    // 見えてる範囲のrenderableを描画する
    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0
                && entity_screen_x < map_width
                && entity_screen_y > 0
                && entity_screen_y < map_height
            {
                draw_batch.set(
                    Point::new(entity_screen_x, entity_screen_y),
                    ColorPair::new(render.fg, render.bg),
                    render.glyph,
                );
            }
        }
    }

    let _ = draw_batch.submit(0);
}

pub fn get_screen_bounds(ecs: &World, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

pub fn render_debug_map(map: &Map, ctx: &mut Rltk) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    let map_width = map.width - 1;
    let map_height = map.height - 1;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(rltk::GRAY), RGB::named(rltk::BLACK), rltk::to_cp437('·'));
            }
            x += 1;
        }
        y += 1;
    }
}

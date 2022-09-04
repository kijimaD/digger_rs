use super::{camera, Faction, Name, Position};
use rltk::prelude::*;
use specs::prelude::*;

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2i32
    }

    fn render(&self, draw_batch: &mut DrawBatch, y: i32) {
        let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        draw_batch.draw_box(
            Rect::with_size(1, y, self.width() - 1, self.height() - 1),
            ColorPair::new(white, box_gray),
        );
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            draw_batch.print_color(Point::new(2, y + i as i32 + 1), &s, ColorPair::new(col, black));
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let factions = ecs.read_storage::<Faction>();
    let entities = ecs.entities();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for (entity, name, position) in (&entities, &names, &positions).join() {
        if position.x == mouse_map_pos.0 + 1 && position.y == mouse_map_pos.1 + 1 {
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

            // Comment on faction
            let faction = factions.get(entity);
            if let Some(faction) = faction {
                tip.add(format!("{}", faction.name));
            }

            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() {
        return;
    }

    let mut y = 2;

    for tip_box in tip_boxes.iter() {
        tip_box.render(&mut draw_batch, y);
        y += tip_box.height();
    }

    draw_batch.submit(7000);
}

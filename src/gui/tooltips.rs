use super::{
    camera, clone_menu, AttributeBonus, Faction, Item, Map, MeleeWeapon, Name, Position, Wearable,
};
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
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x - 1;
    mouse_map_pos.1 += min_y - 1;

    // マウスカーソルを画面外に移動したとき、マップの範囲外になる対策
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }

    // only show on visible tiles
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0 + 1, mouse_map_pos.1 + 1)] {
        return;
    }

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

    draw_batch.submit(6000);
}

// TODO: メニューを閉じたあとmenuから消す
pub fn draw_item_tooltips(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    ctx.set_active_console(2);
    let names = ecs.read_storage::<Name>();
    let items = ecs.read_storage::<Item>();
    let weapons = ecs.read_storage::<MeleeWeapon>();
    let wearables = ecs.read_storage::<Wearable>();
    let attribute_bonuses = ecs.read_storage::<AttributeBonus>();

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for menu in clone_menu().iter() {
        if let Some(name) = names.get(menu.item.0) {
            let entity = menu.item.0;
            let x = menu.item.1.x;
            let y = menu.item.1.y;
            if x <= ctx.mouse_pos().0
                && ctx.mouse_pos().0 <= x + 5 + name.name.len() as i32
                && y == ctx.mouse_pos().1
            {
                let mut tip = Tooltip::new();
                tip.add(name.name.to_string());

                // Comments on Weapon
                let weapon = weapons.get(entity);
                let mut damage_bonus = "";
                let mut hit_bonus = "";
                if let Some(weapon) = weapon {
                    if weapon.damage_bonus > 0 {
                        damage_bonus = "+";
                    }
                    if weapon.hit_bonus > 0 {
                        hit_bonus = "+";
                    }
                    tip.add(format!("Slot: Weapon"));
                    tip.add(format!(
                        "Weapon Class: {}d{}{}{}({}{}) {}",
                        weapon.damage_n_dice,
                        weapon.damage_die_type,
                        damage_bonus,
                        weapon.damage_bonus,
                        weapon.hit_bonus,
                        hit_bonus,
                        weapon.attribute
                    ));
                }

                // Comments on Wearable
                let wearable = wearables.get(entity);
                if let Some(wearable) = wearable {
                    tip.add(format!("Slot: {}", wearable.slot));
                    tip.add(format!("Armor Class: {}", wearable.armor_class));
                }

                // Comments on AttributeBonus
                let attribute_bonus = attribute_bonuses.get(entity);
                if let Some(attribute_bonus) = attribute_bonus {
                    if attribute_bonus.might.is_some() {
                        tip.add(format!("Might: +{}", attribute_bonus.might.unwrap()));
                    }
                    if attribute_bonus.fitness.is_some() {
                        tip.add(format!("Fitness: +{}", attribute_bonus.fitness.unwrap()));
                    }
                    if attribute_bonus.quickness.is_some() {
                        tip.add(format!("Quickness: +{}", attribute_bonus.quickness.unwrap()));
                    }
                    if attribute_bonus.intelligence.is_some() {
                        tip.add(format!(
                            "Intelligence: +{}",
                            attribute_bonus.intelligence.unwrap()
                        ));
                    }
                }

                // Comments on Item
                let item = items.get(entity);
                if let Some(item) = item {
                    tip.add(format!("Weight(kg): {}", item.weight_kg));
                    tip.add(format!("Value: {}", item.base_value));
                }

                tip_boxes.push(tip);

                if tip_boxes.is_empty() {
                    return;
                }

                let mut y = 2;

                for tip_box in tip_boxes.iter() {
                    tip_box.render(&mut draw_batch, y);
                    y += tip_box.height();
                }
            }
        }
    }
    draw_batch.submit(7000);
}

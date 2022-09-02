use super::{gamelog, Combatant, Monster, Name, Player, Pools};
use crate::rex_assets::RexAssets;
use rltk::prelude::*;
use specs::prelude::*;

/// すべての戦闘系stateで共通表示するUI
pub fn draw_battle_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.set_active_console(2); // 背景用
    let assets = ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.forest, 0, 0);

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    // トップバー
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let blue = RGB::named(rltk::SKYBLUE).to_rgba(1.0);
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let pools = ecs.read_storage::<Pools>();
    let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();

    let mut x = 1;
    let left_width: i32 = 10;
    let right_width: i32 = 8;
    let margin: i32 = 1;

    for (player, combatant, player_pools, entity, name) in
        (&players, &combatants, &pools, &entities, &names).join()
    {
        let level = format!("Lv. {}", player_pools.level);
        let health =
            format!("HP {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
        let sp = format!("SP {}/{}", player_pools.sp.current, player_pools.sp.max);

        draw_batch.print_color(Point::new(x, 1), &name.name, ColorPair::new(blue, black));
        draw_batch.print_color(Point::new(x + left_width, 1), &level, ColorPair::new(blue, black));
        draw_batch.print_color(Point::new(x, 2), &health, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(x, 3), &sp, ColorPair::new(white, black));
        draw_batch.bar_horizontal(
            Point::new(x + left_width, 2),
            right_width,
            player_pools.hit_points.current,
            player_pools.hit_points.max,
            ColorPair::new(RGB::named(rltk::GREEN), RGB::named(rltk::BLACK)),
        );
        draw_batch.bar_horizontal(
            Point::new(x + left_width, 3),
            right_width,
            player_pools.sp.current,
            player_pools.sp.max,
            ColorPair::new(RGB::named(rltk::GOLD), RGB::named(rltk::BLACK)),
        );

        x += left_width + right_width + margin;
    }

    // 敵一覧
    let names = ecs.read_storage::<Name>();
    let pools = ecs.read_storage::<Pools>();
    let monsters = ecs.read_storage::<Monster>();
    let combatants = ecs.read_storage::<Combatant>();

    let mut i = 1;
    for (name, pools, _combatant, _monster) in (&names, &pools, &combatants, &monsters).join() {
        draw_batch.print_color(
            Point::new((80 * i) / (combatants.count()), 18),
            format!("{}({})", name.name, pools.hit_points.current),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        i += 1;
    }

    draw_batch.submit(5000);

    gamelog::print_log(
        &crate::gamelog::BATTLE_LOG,
        &mut rltk::BACKEND_INTERNAL.lock().consoles[2].console,
        Point::new(1, 50),
    );

    draw_batch.target(3);

    let mut i = 1;
    for (name, pools, _combatant, _monster) in (&names, &pools, &combatants, &monsters).join() {
        draw_batch.print_color(
            Point::new((10 * i) / (combatants.count()), 1),
            format!("g"),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::WHITE)),
        );
        i += 1;
    }

    draw_batch.submit(6000);
}

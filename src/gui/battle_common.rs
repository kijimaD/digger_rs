use super::{gamelog, Combatant, Monster, Name, Player, Pools};
use rltk::prelude::*;
use specs::prelude::*;

/// すべての戦闘系stateで共通表示するUI
pub fn draw_battle_ui(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    // トップバー
    let black = RGB::named(rltk::BLACK).to_rgba(1.0);
    let white = RGB::named(rltk::WHITE).to_rgba(1.0);
    let players = ecs.read_storage::<Player>();
    let combatants = ecs.read_storage::<Combatant>();
    let pools = ecs.read_storage::<Pools>();
    let names = ecs.read_storage::<Name>();
    let entities = ecs.entities();

    for (player, combatant, player_pools, entity, name) in
        (&players, &combatants, &pools, &entities, &names).join()
    {
        let level = format!("Lv. {}", player_pools.level);
        let health =
            format!("HP {}/{}", player_pools.hit_points.current, player_pools.hit_points.max);
        let sp = format!("SP {}/{}", player_pools.sp.current, player_pools.sp.max);

        draw_batch.print_color(Point::new(1, 1), &name.name, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(12, 1), &level, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(1, 2), &health, ColorPair::new(white, black));
        draw_batch.print_color(Point::new(1, 3), &sp, ColorPair::new(white, black));
        draw_batch.bar_horizontal(
            Point::new(12, 2),
            10,
            player_pools.hit_points.current,
            player_pools.hit_points.max,
            ColorPair::new(RGB::named(rltk::RED), RGB::named(rltk::BLACK)),
        );
        draw_batch.bar_horizontal(
            Point::new(12, 3),
            10,
            player_pools.sp.current,
            player_pools.sp.max,
            ColorPair::new(RGB::named(rltk::GOLD), RGB::named(rltk::BLACK)),
        );
    }

    // ログ
    draw_batch.draw_box(
        Rect::with_size(0, 45, 79, 14),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
    );
    gamelog::print_log(
        &crate::gamelog::BATTLE_LOG,
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );

    // 敵一覧
    let names = ecs.read_storage::<Name>();
    let pools = ecs.read_storage::<Pools>();
    let monsters = ecs.read_storage::<Monster>();
    let combatants = ecs.read_storage::<Combatant>();

    let mut i = 1;
    for (name, pools, _combatant, _monster) in (&names, &pools, &combatants, &monsters).join() {
        draw_batch.print_color(
            Point::new((80 * i) / (1 + combatants.count()), 20),
            format!("{}({})", name.name, pools.hit_points.current),
            ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        );
        i += 1;
    }

    draw_batch.submit(5000);
}

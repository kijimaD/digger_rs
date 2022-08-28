use super::{
    gamelog, player_hp_at_level, sp_at_level, Attributes, Combatant, Equipped, InBackpack,
    LootTable, Map, Monster, Name, OnBattle, Party, Player, Pools, Position, RunState,
};
use specs::prelude::*;

/// 戦闘エンティティの死亡判定と削除。
/// TODO: 長い関数の分割および、effectでやってることとの違いが曖昧なので明確にする必要がある
pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    let mut maybe_win = false;
    // Using a scope to make the borrow checker happy
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let combatant = ecs.read_storage::<Combatant>();

        let entities = ecs.entities();
        for (entity, pools, _combatant) in (&entities, &pools, &combatant).join() {
            if pools.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            crate::gamelog::Logger::new()
                                .color(rltk::YELLOW)
                                .append(&victim_name.name)
                                .color(rltk::WHITE)
                                .append("is")
                                .color(rltk::RED)
                                .append("dead.")
                                .log(&crate::gamelog::LogKind::Battle);
                        }
                        dead.push(entity);

                        maybe_win = true;
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // HPが0になったbattle entityの削除
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }

    // 勝利判定
    if maybe_win {
        check_battle_win(ecs);
    }
}

/// 戦闘中の敵が残ってないとき、勝利。アイテムドロップ、シンボルエンティティを消す、state切り替えなどをやる
fn check_battle_win(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let pools = ecs.read_storage::<Pools>();
        let monsters = ecs.read_storage::<Monster>();
        let combatants = ecs.read_storage::<Combatant>();
        let on_battles = ecs.read_storage::<OnBattle>();

        if (&entities, &pools, &monsters, &combatants).join().count() == 0 {
            for (_entity, on_battle) in (&entities, &on_battles).join() {
                dead.push(on_battle.monster);
            }
        }
    }

    {
        let entities = ecs.entities();
        for victim in dead.clone() {
            crate::gamelog::Logger::new().append("You win!").log(&crate::gamelog::LogKind::Battle);

            let player_entity = ecs.fetch::<Entity>();
            let mut pools = ecs.write_storage::<Pools>();
            let mut attributes = ecs.write_storage::<Attributes>();
            let mut parties = ecs.write_storage::<Party>();
            let mut on_battles = ecs.write_storage::<OnBattle>();
            let combatants = ecs.read_storage::<Combatant>();
            let players = ecs.read_storage::<Player>();

            let mut party = parties.get_mut(*player_entity).unwrap();
            let on_battle = on_battles.get(*player_entity).unwrap();

            for (_entity, player_stats, player_attributes, _player, _combatant) in
                (&entities, &mut pools, &mut attributes, &players, &combatants).join()
            {
                player_stats.xp += on_battle.xp;

                // level up
                if player_stats.xp >= player_stats.level * 1000 {
                    player_stats.level += 1;
                    gamelog::Logger::new()
                        .append(format!("Congratulations, you are now level{}", player_stats.level))
                        .color(rltk::MAGENTA)
                        .append("Congratulations, you are now level")
                        .append(format!("{}", player_stats.level))
                        .log(&crate::gamelog::LogKind::Field);
                    player_stats.hit_points.max = player_hp_at_level(
                        player_attributes.fitness.base + player_attributes.fitness.modifiers,
                        player_stats.level,
                    );
                    player_stats.hit_points.current = player_stats.hit_points.max;
                    player_stats.sp.max = sp_at_level(
                        player_attributes.intelligence.base
                            + player_attributes.intelligence.modifiers,
                        player_stats.level,
                    );
                    player_stats.sp.current = player_stats.sp.max;
                }
            }
            party.gold += on_battle.gold;

            entities.delete(victim).expect("Delete failed");
            let mut runstate = ecs.write_resource::<RunState>();
            *runstate = RunState::BattleWinResult;
        }
    }

    // アイテムドロップ(フィールド)
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    {
        let mut to_drop: Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();

        for victim in dead.iter() {
            let pos = positions.get(*victim);
            for (entity, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }
            for (entity, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    // Drop their stuff
                    if let Some(pos) = pos {
                        to_drop.push((entity, pos.clone()));
                    }
                }
            }

            if let Some(table) = loot_tables.get(*victim) {
                let drop_finder = crate::raws::get_item_drop(
                    &crate::raws::RAWS.lock().unwrap(),
                    &mut rng,
                    &table.table,
                );
                if let Some(tag) = drop_finder {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, pos.clone()));
                    }
                }
            }
        }

        for drop in to_drop.iter() {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert position");
        }
    }

    {
        for drop in to_spawn.iter() {
            crate::raws::spawn_named_item(
                &crate::raws::RAWS.lock().unwrap(),
                ecs,
                &drop.0,
                crate::raws::SpawnType::AtPosition { x: drop.1.x, y: drop.1.y },
            );
        }
    }
}

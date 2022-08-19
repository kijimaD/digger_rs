use super::{
    gamelog::BattleLog, Attributes, OnBattle, Combatant, Equipped, InBackpack, LootTable, Map,
    Monster, Name, Player, Pools, Position, RunState,
};
use specs::prelude::*;

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
        let mut log = ecs.write_resource::<BattleLog>();
        for (entity, pools, _combatant) in (&entities, &pools, &combatant).join() {
            if pools.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
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

    // HPが0になったentityの削除
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }

    // 勝利判定
    if maybe_win {
        check_battle_win(ecs);
    }
}

/// 戦闘中の敵が残ってないとき、勝利。stateを切り替え、敵のシンボルエンティティ(Battleを持つ)を消す。
/// 敵のシンボルエンティティがbattleを持つというのがわかりにくい。
fn check_battle_win(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let pools = ecs.read_storage::<Pools>();
        let monster = ecs.read_storage::<Monster>();
        let combatant = ecs.read_storage::<Combatant>();
        let battle_ecs = ecs.write_storage::<OnBattle>();

        if (&entities, &pools, &monster, &combatant).join().count() == 0 {
            for (entity, _battle) in (&entities, &battle_ecs).join() {
                dead.push(entity);
            }
        }
    }

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

    {
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<BattleLog>();

        for victim in dead {
            log.entries.push(format!("You win!"));

            entities.delete(victim).expect("Delete failed");

            let mut runstate = ecs.write_resource::<RunState>();
            *runstate = RunState::BattleResult;
        }
    }
}

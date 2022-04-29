use super::{
    gamelog::BattleLog, Battle, CombatStats, Map, Monster, Name, Player, Position, RunState,
    SufferDamage,
};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, CombatStats>, WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    let mut maybe_win = false;
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<BattleLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
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

    // HPが0になった戦闘entityの削除
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }

    // 勝利判定
    if maybe_win {
        check_battle_win(ecs);
    }
}

// 戦闘に勝利していたら、stateを切り替えmap entityを削除する
fn check_battle_win(ecs: &mut World) {
    let mut want_remove_battle = false;
    let mut dead_map_entity: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let monster = ecs.read_storage::<Monster>();
        let mut log = ecs.write_resource::<BattleLog>();
        let positions = ecs.read_storage::<Position>();
        let mut map = ecs.write_resource::<Map>();

        // 攻撃の結果敵が残ってないときは*勝利*
        // 攻撃してなくて敵が残ってないときは*逃走*
        if (&entities, &combat_stats, &monster).join().count() == 0 {
            let battle_ecs = ecs.read_storage::<Battle>();

            for battle in (&battle_ecs).join() {
                let mut runstate = ecs.write_resource::<RunState>();
                *runstate = RunState::BattleResult;
                dead_map_entity.push(battle.monster);

                let pos = positions.get(battle.monster);
                if let Some(pos) = pos {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }

                want_remove_battle = true;
                log.entries.push(format!("You win!"));
            }
        }
    }

    // 勝ったらmap_entityを削除する
    if want_remove_battle {
        for map_entity in dead_map_entity {
            ecs.delete_entity(map_entity).expect("Unable to delete");
        }
    }

    // 勝ったらbattleを削除する
    if want_remove_battle {
        let mut battle = ecs.write_storage::<Battle>();
        battle.clear();
    }
}

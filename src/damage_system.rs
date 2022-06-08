use super::{
    gamelog::BattleLog, Battle, Combatant, Map, Monster, Name, Player, Pools, Position, RunState,
    SufferDamage,
};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, Pools>, WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut pools, mut damage) = data;

        for (mut pools, damage) in (&mut pools, &damage).join() {
            pools.hit_points.current -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    let mut maybe_win = false;
    // Using a scope to make the borrow checker happy
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let combatant = ecs.read_storage::<Combatant>();
        let positions = ecs.read_storage::<Position>();
        let mut map = ecs.write_resource::<Map>();

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

// 戦闘中の敵が残ってないとき、勝利。stateを切り替え、敵のシンボルエンティティ(Battleを持つ)を消す
fn check_battle_win(ecs: &mut World) {
    {
        let entities = ecs.entities();
        let pools = ecs.read_storage::<Pools>();
        let monster = ecs.read_storage::<Monster>();
        let combatant = ecs.read_storage::<Combatant>();
        let mut log = ecs.write_resource::<BattleLog>();
        let battle_ecs = ecs.write_storage::<Battle>();
        let positions = ecs.read_storage::<Position>();
        let mut map = ecs.write_resource::<Map>();

        if (&entities, &pools, &monster, &combatant).join().count() == 0 {
            for (entity, _battle) in (&entities, &battle_ecs).join() {
                log.entries.push(format!("You win!"));

                entities.delete(entity).expect("Delete failed");

                // floor blood
                let pos = positions.get(entity);
                if let Some(pos) = pos {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }

                let mut runstate = ecs.write_resource::<RunState>();
                *runstate = RunState::BattleResult;
            }
        }
    }
}

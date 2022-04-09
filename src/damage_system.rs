use specs::prelude::*;
use super::{CombatStats, SufferDamage, Player, Name, gamelog::GameLog, RunState, Battle, Monster};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
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
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // TODO: map上entityの削除もする。現在は戦闘用しか削除してない
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }

    // 戦闘に勝利したらmap entityを削除する
    // TODO: 逃げたときもmap entityが消えている
    let mut want_remove_battle = false;
    let mut dead_map_entity : Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let combat_stats = ecs.read_storage::<CombatStats>();
        let monster = ecs.read_storage::<Monster>();

        if (&entities, &combat_stats, &monster).join().count() == 0 {
            let battle_ecs = ecs.read_storage::<Battle>();

            for battle in (&battle_ecs).join() {
                // 残ってないときは戦闘終了
                let mut runstate = ecs.write_resource::<RunState>();
                *runstate = RunState::AwaitingInput;
                dead_map_entity.push(battle.monster);
                want_remove_battle = true;

            }
        }
    }

    {
        let mut battle = ecs.write_storage::<Battle>();
        if want_remove_battle {
            battle.clear();
        }
    }

    {
        for map_entity in dead_map_entity {
            ecs.delete_entity(map_entity).expect("Unable to delete");
        }
    }
}

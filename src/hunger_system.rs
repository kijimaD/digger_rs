use super::{effects::*, gamelog::GameLog, HungerClock, HungerState, MyTurn, Pools};
use specs::prelude::*;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>, // The player
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, MyTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, mut log, pools, turns) = data;

        for (entity, mut clock, _myturn) in (&entities, &mut hunger_clock, &turns).join() {
            clock.duration -= 1;
            if clock.duration < 1 {
                match clock.state {
                    HungerState::WellFed => {
                        clock.state = HungerState::Normal;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.push("You are no longer well fed.".to_string());
                        }
                    }
                    HungerState::Normal => {
                        clock.state = HungerState::Hungry;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.push("You are hungry.".to_string());
                        }
                    }
                    HungerState::Hungry => {
                        clock.state = HungerState::Starving;
                        clock.duration = 200;
                        if entity == *player_entity {
                            log.entries.push("You are starving!".to_string());
                        }
                    }
                    HungerState::Starving => {
                        // Inflict damage from hunger
                        if entity == *player_entity {
                            log.entries.push(
                                "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                    .to_string(),
                            );
                        }
                        for (entity, _pools) in (&entities, &pools).join() {
                            add_effect(
                                Some(entity),
                                EffectType::Damage { amount: 1 },
                                Targets::Single { target: entity },
                            );
                        }
                    }
                }
            }
        }
    }
}

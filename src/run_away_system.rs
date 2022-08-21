use super::{gamelog, Combatant, Monster, OnBattle};
use rltk::prelude::*;
use specs::prelude::*;

pub enum RunAwayResult {
    Success,
    Fail,
}

pub fn run_away_roll(ecs: &mut World) -> RunAwayResult {
    let mut rng = RandomNumberGenerator::new();
    let num = rng.range(0, 2);

    if num == 0 {
        run_away_battle(ecs);
        gamelog::Logger::new()
            .color(rltk::GREEN)
            .append("Run away!")
            .log(&crate::gamelog::LogKind::Battle);
        return RunAwayResult::Success;
    } else {
        gamelog::Logger::new().append("Failed run away!").log(&crate::gamelog::LogKind::Battle);
        return RunAwayResult::Fail;
    }
}

fn run_away_battle(ecs: &mut World) {
    let combatants = ecs.write_storage::<Combatant>();
    let monsters = ecs.read_storage::<Monster>();
    let entities = ecs.entities();

    for (entity, _combatant, _monster) in (&entities, &combatants, &monsters).join() {
        entities.delete(entity).expect("Delete failed")
    }

    // battle削除
    let mut battle = ecs.write_storage::<OnBattle>();
    battle.clear();
}

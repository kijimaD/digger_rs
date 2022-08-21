use super::{
    OnBattle, Combatant, Monster, gamelog
};
use specs::prelude::*;

pub fn run_away_battle(ecs: &mut World) {
    let combatants = ecs.write_storage::<Combatant>();
    let monsters = ecs.read_storage::<Monster>();
    let entities = ecs.entities();

    for (entity, _combatant, _monster) in (&entities, &combatants, &monsters).join() {
        entities.delete(entity).expect("Delete failed")
    }

    // battle削除
    let mut battle = ecs.write_storage::<OnBattle>();
    battle.clear();

    gamelog::Logger::new()
        .color(rltk::GREEN)
        .append("Run away!")
        .log(&crate::gamelog::LogKind::Battle);
}

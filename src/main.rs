extern crate serde;
use rltk::{GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod battle_action_system;
use battle_action_system::BattleActionSystem;
mod ai;
mod encounter_system;
mod gamelog;
mod gui;
mod inventory_system;
mod spawner;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem};
pub mod camera;
mod gamesystem;
mod hunger_system;
pub mod map_builders;
mod particle_system;
pub mod random_table;
pub mod raws;
pub mod rex_assets;
pub mod saveload_system;
pub use gamesystem::*;
mod lighting_system;

#[macro_use]
extern crate lazy_static;

const SHOW_MAPGEN_VISUALIZER: bool = false;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    BattleEncounter,
    BattleCommand,
    BattleInventory,
    BattleItemTargeting { item: Entity },
    BattleTurn,
    BattleResult,
    BattleAwaiting,
    BattleTargeting,
    ShowInventory,
    ItemTargeting { item: Entity },
    ShowDropItem,
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    ShowRemoveItem,
    GameOver,
    MapGeneration,
    ShowCheatMenu,
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut initiative = ai::InitiativeSystem {};
        initiative.run_now(&self.ecs);
        let mut visible = ai::VisibleAI {};
        visible.run_now(&self.ecs);
        let mut adjacent = ai::AdjacentAI {};
        adjacent.run_now(&self.ecs);
        let mut approach = ai::ApproachAI {};
        approach.run_now(&self.ecs);
        let mut flee = ai::FleeAI {};
        flee.run_now(&self.ecs);
        let mut defaultmove = ai::DefaultMoveAI {};
        defaultmove.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut itemuse = ItemUseSystem {};
        itemuse.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);
        let mut hunger = hunger_system::HungerSystem {};
        hunger.run_now(&self.ecs);
        let mut particles = particle_system::ParticleSpawnSystem {};
        particles.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut lighting = lighting_system::LightingSystem {};
        lighting.run_now(&self.ecs);
        let mut quipper = ai::QuipSystem {};
        quipper.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn run_battle_systems(&mut self) {
        let mut battle_action = BattleActionSystem {};
        battle_action.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut itemuse = ItemUseSystem {};
        itemuse.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        // マップUI表示
        match newrunstate {
            // 除外
            RunState::MainMenu { .. }
            | RunState::GameOver
            | RunState::BattleEncounter
            | RunState::BattleCommand
            | RunState::BattleInventory
            | RunState::BattleItemTargeting { .. }
            | RunState::BattleTurn
            | RunState::BattleAwaiting
            | RunState::BattleTargeting
            | RunState::BattleResult => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        // 戦闘UI表示
        match newrunstate {
            RunState::BattleEncounter
            | RunState::BattleCommand
            | RunState::BattleInventory
            | RunState::BattleItemTargeting { .. }
            | RunState::BattleTurn
            | RunState::BattleAwaiting
            | RunState::BattleTargeting
            | RunState::BattleResult => gui::draw_battle_ui(&self.ecs, ctx),
            _ => {}
        }

        // 全体処理
        match newrunstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                if self.mapgen_index < self.mapgen_history.len() {
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                }

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        newrunstate = self.mapgen_next_state.unwrap();
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::Ticking => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::AwaitingInput => newrunstate = RunState::AwaitingInput,
                    _ => newrunstate = RunState::Ticking,
                }
            }
            RunState::BattleEncounter => {
                newrunstate = RunState::BattleAwaiting;
            }
            RunState::BattleCommand => {
                // 戦闘コマンド
                let result = gui::battle_command(&mut self.ecs, ctx);

                // メインメニュー表示
                match result {
                    gui::BattleCommandResult::NoResponse => {}
                    gui::BattleCommandResult::Attack => newrunstate = RunState::BattleTargeting,
                    gui::BattleCommandResult::ShowInventory => {
                        newrunstate = RunState::BattleInventory
                    }
                    gui::BattleCommandResult::RunAway => newrunstate = RunState::BattleResult,
                    gui::BattleCommandResult::RunAwayFailed => newrunstate = RunState::BattleTurn,
                }
            }
            RunState::BattleInventory => {
                let result = gui::show_battle_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::BattleCommand,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        newrunstate = RunState::BattleItemTargeting { item: item_entity };
                    }
                }
            }
            RunState::BattleItemTargeting { item } => {
                let result = gui::show_item_targeting(self, ctx);
                match result.0 {
                    gui::ItemTargetingResult::Cancel => newrunstate = RunState::BattleInventory,
                    gui::ItemTargetingResult::NoResponse => {}
                    gui::ItemTargetingResult::Selected => {
                        let target = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::BattleTurn;
                    }
                }
            }
            RunState::BattleTurn => {
                // 選んだコマンドを実行 + AIのコマンドを実行
                // 行動1つ1つでenter送りにできるのが望ましいが、現在はターン毎に結果を表示してenter待ちにしている
                self.run_battle_systems();
                self.ecs.maintain();

                newrunstate = RunState::BattleAwaiting;
            }
            RunState::BattleAwaiting => {
                // enter待ち状態にする。enter後はcommand選択画面へ遷移
                match ctx.key {
                    None => {}
                    Some(key) => match key {
                        VirtualKeyCode::Return => {
                            newrunstate = RunState::BattleCommand;
                        }
                        _ => {}
                    },
                }

                ctx.print_color(
                    70,
                    44,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::BLACK),
                    "[Enter]",
                );
            }
            RunState::BattleTargeting => {
                // 攻撃目標選択
                let result = gui::battle_target(self, ctx);
                let entities = self.ecs.entities();
                let player = self.ecs.read_storage::<Player>();
                let pools = self.ecs.write_storage::<Pools>();
                let mut wants_to_melee = self.ecs.write_storage::<WantsToMelee>();

                // TODO: 複数キャラのコマンドに対応してない
                for (entity, _player, _pools) in (&entities, &player, &pools).join() {
                    match result.0 {
                        gui::BattleTargetingResult::Cancel => newrunstate = RunState::BattleCommand,
                        gui::BattleTargetingResult::NoResponse => {}
                        gui::BattleTargetingResult::Selected => {
                            let target_entity = result.1.unwrap();
                            wants_to_melee
                                .insert(entity, WantsToMelee { target: target_entity })
                                .expect("Unable to insert WantsToMelee");

                            newrunstate = RunState::BattleTurn
                        }
                    }
                }
            }
            RunState::BattleResult => {
                // 戦闘終了(勝利 or 逃走)
                let result = gui::show_battle_win_result(self, ctx);
                match result {
                    gui::BattleResult::NoResponse => {}
                    gui::BattleResult::Enter => newrunstate = RunState::AwaitingInput,
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_field_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        newrunstate = RunState::ItemTargeting { item: item_entity };
                    }
                }
            }
            RunState::ItemTargeting { item } => {
                let result = gui::show_item_targeting(self, ctx);
                match result.0 {
                    gui::ItemTargetingResult::Cancel => newrunstate = RunState::ShowInventory,
                    gui::ItemTargetingResult::NoResponse => {}
                    gui::ItemTargetingResult::Selected => {
                        let target = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu { menu_selection: selected }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate =
                            RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate =
                    RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame };
            }
            RunState::NextLevel => {
                self.goto_level(1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::PreRun;
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        // FIXME: 勝利しても、1ターン残る
        damage_system::delete_the_dead(&mut self.ecs);

        if encounter_system::is_encounter(&mut self.ecs) {
            spawner::battle_monster(&mut self.ecs, "orcA");
        }
        encounter_system::invoke_battle(&mut self.ecs);
    }
}

impl State {
    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            let eq = equipped.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.push("You change level.".to_string());
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Replace the world maps
        self.ecs.insert(map::MasterDungeonMap::new());

        // Build a new map and place the player
        self.generate_world_map(1, 0);
    }

    fn generate_world_map(&mut self, new_depth: i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context =
        RltkBuilder::simple(80, 60).unwrap().with_title("Battle Digger Clone").build()?;
    context.with_post_scanlines(true);
    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToEncounter>();
    gs.ecs.register::<Battle>();
    gs.ecs.register::<Combatant>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<Wearable>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<Quips>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);

    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration {});
    gs.ecs.insert(gamelog::GameLog { entries: vec!["Enter the dungeon...".to_string()] });
    gs.ecs.insert(gamelog::BattleLog { entries: vec!["".to_string()] });
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}

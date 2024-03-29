extern crate serde;
use rltk::{GameState, Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
use components::*;
mod formatter;
mod map;
use map::*;
mod player;
use player::*;
mod rect;
use rect::Rect;
mod ai;
mod damage_system;
mod effects;
mod encounter_system;
mod gamelog;
mod gamesystem;
mod gui;
mod inventory_system;
mod map_builders;
mod random_table;
mod raws;
mod rex_assets;
mod run_away_system;
mod saveload_system;
mod spawner;
mod systems;
use gamesystem::*;
mod spatial;

#[macro_use]
extern crate lazy_static;

const SHOW_MAPGEN_VISUALIZER: bool = false;

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode {
    Buy,
    Sell,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    BattleEncounter,
    BattleCommand,
    BattleInventory,
    BattleItemTargeting { item: Entity },
    BattleTurn,
    BattleRunAwayResult,
    BattleWinResult,
    BattleAwaiting,
    BattleAttackWay,
    BattleAttackTargeting { way: Entity },
    AwaitingInput,
    PreRun,
    Ticking,
    ShowUseItem,
    ItemTargeting { item: Entity },
    ShowEquipItem { entity: Entity, index: i32 },
    ShowDropItem,
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    ShowRemoveItem,
    GameOver,
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode: VendorMode },
    TeleportingToOtherLevel { x: i32, y: i32, depth: i32 },
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_field_systems(&mut self) {
        let mut vis = systems::VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut initiative = ai::InitiativeSystem {};
        initiative.run_now(&self.ecs);
        let mut quipper = ai::QuipSystem {};
        quipper.run_now(&self.ecs);
        let mut moving = systems::MovementSystem {};
        moving.run_now(&self.ecs);
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
        let mut mapindex = systems::MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut pickup = inventory_system::ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut itemequip = inventory_system::ItemEquipOnUse {};
        itemequip.run_now(&self.ecs);
        let mut itemuse = inventory_system::ItemUseSystem {};
        itemuse.run_now(&self.ecs);
        let mut drop_items = inventory_system::ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        let mut item_remove = inventory_system::ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);
        let mut hunger = systems::HungerSystem {};
        hunger.run_now(&self.ecs);
        let mut encumbrance = ai::EncumbranceSystem {};
        encumbrance.run_now(&self.ecs);
        let mut trigger = systems::TriggerSystem {};
        trigger.run_now(&self.ecs);

        effects::run_effects_queue(&mut self.ecs);
        let mut particles = systems::ParticleSpawnSystem {};
        particles.run_now(&self.ecs);
        let mut lighting = systems::LightingSystem {};
        lighting.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn run_battle_systems(&mut self) {
        let mut battle_action = systems::BattleActionSystem {};
        battle_action.run_now(&self.ecs);
        let mut melee = systems::MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut itemuse = inventory_system::ItemUseSystem {};
        itemuse.run_now(&self.ecs);

        effects::run_effects_queue(&mut self.ecs);

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

        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        ctx.set_active_console(3);
        ctx.cls();
        ctx.set_active_console(0);

        systems::cull_dead_particles(&mut self.ecs, ctx);

        // マップUI表示
        match newrunstate {
            RunState::AwaitingInput
            | RunState::PreRun
            | RunState::Ticking
            | RunState::ShowUseItem
            | RunState::ShowEquipItem { .. }
            | RunState::ItemTargeting { .. }
            | RunState::ShowDropItem
            | RunState::SaveGame
            | RunState::NextLevel
            | RunState::PreviousLevel
            | RunState::TownPortal
            | RunState::ShowRemoveItem
            | RunState::MapGeneration
            | RunState::ShowCheatMenu
            | RunState::ShowVendor { .. }
            | RunState::TeleportingToOtherLevel { .. } => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
            _ => {}
        }

        // 戦闘UI表示
        match newrunstate {
            RunState::BattleEncounter
            | RunState::BattleCommand
            | RunState::BattleInventory
            | RunState::BattleItemTargeting { .. }
            | RunState::BattleTurn
            | RunState::BattleAwaiting
            | RunState::BattleAttackWay
            | RunState::BattleAttackTargeting { .. }
            | RunState::BattleRunAwayResult
            | RunState::BattleWinResult => gui::draw_battle_ui(&self.ecs, ctx),
            _ => {}
        }

        // 全体処理
        match newrunstate {
            RunState::BattleEncounter => {
                newrunstate = RunState::BattleAwaiting;
            }
            RunState::BattleCommand => {
                // 戦闘コマンド
                let result = gui::show_battle_command(&mut self.ecs, ctx);

                // コマンドメニュー表示
                match result {
                    gui::BattleCommandResult::NoResponse => {}
                    gui::BattleCommandResult::Attack => newrunstate = RunState::BattleAttackWay,
                    gui::BattleCommandResult::ShowInventory => {
                        newrunstate = RunState::BattleInventory
                    }
                    gui::BattleCommandResult::RunAway => {
                        newrunstate = RunState::BattleRunAwayResult
                    }
                    gui::BattleCommandResult::RunAwayFailed => newrunstate = RunState::BattleTurn,
                }
            }
            RunState::BattleInventory => {
                let result = gui::show_use_item(self, ctx);
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
            RunState::BattleAttackWay => {
                // 攻撃方法選択
                let result = gui::show_attack_way(self, ctx);

                match result.0 {
                    gui::BattleAttackWayResult::Cancel => newrunstate = RunState::BattleCommand,
                    gui::BattleAttackWayResult::NoResponse => {}
                    gui::BattleAttackWayResult::Selected => {
                        newrunstate = RunState::BattleAttackTargeting { way: result.1.unwrap() }
                    }
                }
            }
            RunState::BattleAttackTargeting { way } => {
                // 攻撃目標選択
                let result = gui::show_attack_target(self, ctx);
                let entities = self.ecs.entities();
                let player = self.ecs.read_storage::<Player>();
                let pools = self.ecs.write_storage::<Pools>();
                let mut wants_to_melee = self.ecs.write_storage::<WantsToMelee>();

                // TODO: 複数キャラのコマンドに対応してない
                for (entity, _player, _pools) in (&entities, &player, &pools).join() {
                    match result.0 {
                        gui::BattleAttackTargetingResult::Cancel => {
                            newrunstate = RunState::BattleAttackWay
                        }
                        gui::BattleAttackTargetingResult::NoResponse => {}
                        gui::BattleAttackTargetingResult::Selected => {
                            let target_entity = result.1.unwrap();
                            wants_to_melee
                                .insert(
                                    entity,
                                    WantsToMelee { target: target_entity, way: Some(way) },
                                )
                                .expect("Unable to insert WantsToMelee");

                            newrunstate = RunState::BattleTurn
                        }
                    }
                }
            }
            RunState::BattleRunAwayResult => {
                // 戦闘終了(逃走)
                match ctx.key {
                    None => {}
                    Some(key) => match key {
                        VirtualKeyCode::Return => {
                            newrunstate = RunState::AwaitingInput;
                        }
                        _ => {}
                    },
                }
            }
            RunState::BattleWinResult => {
                // 戦闘終了(勝利)
                {
                    let result = gui::show_battle_result(self, ctx);
                    match result {
                        gui::BattleWinResult::NoResponse => {}
                        gui::BattleWinResult::Enter => {
                            newrunstate = RunState::AwaitingInput;

                            // MEMO: 倒した敵が消えないため
                            self.ecs.maintain();
                            self.run_field_systems();
                        }
                    }
                }
            }
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
                self.run_field_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
                if newrunstate != RunState::AwaitingInput {
                    crate::gamelog::record_event("Turn", 1);
                }
            }
            RunState::Ticking => {
                while newrunstate == RunState::Ticking {
                    self.run_field_systems();
                    self.ecs.maintain();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => newrunstate = RunState::AwaitingInput,
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel { x, y, depth } => {
                            newrunstate = RunState::TeleportingToOtherLevel { x, y, depth }
                        }
                        _ => newrunstate = RunState::Ticking,
                    }
                }
            }
            RunState::ShowUseItem => {
                let result = gui::show_use_item(self, ctx);
                let consumables = self.ecs.read_storage::<Consumable>();

                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        // フィールドエンティティ向けの場合はtarget選択画面をスキップ
                        if let Some(consumable) = consumables.get(item_entity) {
                            match consumable.target {
                                ItemTarget::Field => {
                                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                                    let player_entity = self.ecs.fetch::<Entity>();
                                    intent
                                        .insert(
                                            *self.ecs.fetch::<Entity>(),
                                            WantsToUseItem {
                                                item: item_entity,
                                                target: *player_entity,
                                            },
                                        )
                                        .expect("Unable to insert intent");
                                    newrunstate = RunState::Ticking;
                                }
                                ItemTarget::Battle => {
                                    newrunstate = RunState::ItemTargeting { item: item_entity }
                                }
                            }
                        } else {
                            newrunstate = RunState::ItemTargeting { item: item_entity };
                        }
                    }
                }
            }
            RunState::ItemTargeting { item } => {
                let result = gui::show_item_targeting(self, ctx);
                match result.0 {
                    gui::ItemTargetingResult::Cancel => newrunstate = RunState::ShowUseItem,
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
            RunState::ShowEquipItem { entity, index } => {
                let result = gui::equip_item_menu(self, ctx, entity);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        newrunstate = RunState::Ticking;
                    }
                }

                // 1つのmatch文ですべてのパターンを網羅したいが、共通のメニュー関数を使っているため↑に書けない。
                let result = gui::equipment_key_move(self, ctx, &entity, index);
                match result.0 {
                    gui::EquipmentMenuResult::Next => {
                        newrunstate = RunState::ShowEquipItem { entity: result.1, index: result.2 }
                    }
                    gui::EquipmentMenuResult::Prev => {
                        newrunstate = RunState::ShowEquipItem { entity: result.1, index: result.2 }
                    }
                    gui::EquipmentMenuResult::NoResponse => {}
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
            RunState::TownPortal => {
                // Spawn the portal
                spawner::spawn_town_portal(&mut self.ecs);

                // Transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - (map_depth - 1);
                self.goto_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.goto_level(depth - 1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<rltk::Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::ShowVendor { vendor, mode } => {
                use crate::raws::*;
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {}
                    gui::VendorResult::Sell => {
                        let price = self
                            .ecs
                            .read_storage::<Item>()
                            .get(result.1.unwrap())
                            .unwrap()
                            .base_value
                            * 0.8;
                        self.ecs
                            .write_storage::<Party>()
                            .get_mut(*self.ecs.fetch::<Entity>())
                            .unwrap()
                            .gold += price;
                        self.ecs.delete_entity(result.1.unwrap()).expect("Unable to delete");
                    }
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut parties = self.ecs.write_storage::<Party>();
                        let party = parties.get_mut(*self.ecs.fetch::<Entity>()).unwrap();
                        if party.gold >= price {
                            party.gold -= price;
                            std::mem::drop(parties);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            crate::raws::spawn_named_item(
                                &RAWS.lock().unwrap(),
                                &mut self.ecs,
                                &tag,
                                SpawnType::Carried { by: player_entity },
                            );
                        }
                    }
                    gui::VendorResult::BuyMode => {
                        newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Buy }
                    }
                    gui::VendorResult::SellMode => {
                        newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Sell }
                    }
                }
            }
            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::Reveal => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        }
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut parties = self.ecs.write_storage::<Party>();
                        let mut party = parties.get_mut(*player).unwrap();
                        party.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::SpawnMonster { monster_x, monster_y } => {
                        raws::spawn_named_entity(
                            &raws::RAWS.lock().unwrap(),
                            &mut self.ecs,
                            "Lime",
                            raws::SpawnType::AtPosition { x: monster_x, y: monster_y },
                        );
                        newrunstate = RunState::AwaitingInput;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        // 毎ループ最後に実行するため、system化できない
        damage_system::delete_the_dead(&mut self.ecs);

        // systemにできない理由としては、mut ecsを使うから。ecs.create_entityする必要があるが、system内ではecsを取得できないのでそれを行うことができない。ほかのsystemではcomponentをいじくるだけで、ecsを直接必要としないので可能
        encounter_system::invoke_battle(&mut self.ecs);

        let _ = rltk::render_draw_buffer(ctx);
    }
}

impl State {
    fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth + offset, offset);

        gamelog::Logger::new().append("You change Level.").log(&crate::gamelog::LogKind::Field);
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

        gamelog::clear_log(&crate::gamelog::FIELD_LOG);
        gamelog::Logger::new()
            .append("Enter the")
            .color(rltk::RED)
            .append("dungeon...")
            .log(&crate::gamelog::LogKind::Field);

        gamelog::clear_events();
    }
}

fn main() -> rltk::BError {
    use rltk::BTermBuilder;

    const SCREEN_WIDTH: i32 = 80;
    const SCREEN_HEIGHT: i32 = 60;
    const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;

    rltk::embedded_resource!(DUNGEON_FONT, "../resources/dungeonfont.png");
    rltk::link_resource!(DUNGEON_FONT, "resources/dungeonfont.png");

    let context = BTermBuilder::new()
        .with_title("Diggers")
        .with_fps_cap(60.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_font("vga8x16.png", 8, 16)
        .with_font("dungeonfont.png", 32, 32)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png") // 0.フィールドのタイル画像(小)
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png") // 1. フィールドキャラクタの画像(小)
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "vga8x16.png") // 2. 文字表示
        .with_simple_console_no_bg(DISPLAY_WIDTH / 4, DISPLAY_HEIGHT / 4, "dungeonfont.png") // 3.戦闘時の敵画像(大)
        .build()?;

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
    gs.ecs.register::<Description>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<Party>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToEncounter>();
    gs.ecs.register::<OnBattle>();
    gs.ecs.register::<Combatant>();
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
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<Wearable>();
    gs.ecs.register::<AttributeBonus>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<Quips>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    let battle_entity1 =
        raws::spawn_named_fighter(&raws::RAWS.lock().unwrap(), &mut gs.ecs, "Ishihara");
    let battle_entity2 =
        raws::spawn_named_fighter(&raws::RAWS.lock().unwrap(), &mut gs.ecs, "Shirase");

    let field_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(battle_entity1);
    gs.ecs.insert(battle_entity2);
    gs.ecs.insert(field_entity);
    gs.ecs.insert(RunState::MapGeneration {});
    gamelog::clear_log(&crate::gamelog::FIELD_LOG);
    gs.ecs.insert(systems::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}

use super::Renderable;
use serde::Deserialize;
use std::collections::HashMap;

/// マップ上でシンボルとして表示されるプレイヤー以外のエンティティ。
/// 敵だった場合、戦闘イベントを起こして各モンスターを生成する。そのため、この構造体は戦闘発生後に関する情報を持たない。
#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub blocks_tile: bool,
    pub vision_range: i32,
    pub movement: String,
    pub quips: Option<Vec<String>>,
    pub equipped: Option<Vec<String>>, // TODO: move to battle entity
    pub natural: Option<MobNatural>,   // TODO: move to battle entity
    pub loot_table: Option<String>,
    pub light: Option<MobLight>,
    pub faction: Option<String>,
    pub vendor: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub might: Option<i32>,
    pub fitness: Option<i32>,
    pub quickness: Option<i32>,
    pub intelligence: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct MobNatural {
    pub armor_class: Option<i32>,
    pub attacks: Option<Vec<NaturalAttack>>,
}

#[derive(Deserialize, Debug)]
pub struct NaturalAttack {
    pub name: String,
    pub hit_bonus: i32,
    pub damage: String,
}

#[derive(Deserialize, Debug)]
pub struct MobLight {
    pub range: i32,
    pub color: String,
}

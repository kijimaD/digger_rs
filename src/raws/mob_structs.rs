use super::Renderable;
use serde::Deserialize;

/// マップ上でシンボルとして表示される、プレイヤー以外のエンティティ。
/// 敵だった場合、戦闘イベントを起こして各モンスターを生成する。そのため、この構造体は戦闘発生後に関する情報を持たない。
#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub blocks_tile: bool,
    pub vision_range: i32,
    pub movement: String,
    pub quips: Option<Vec<String>>,
    pub loot_table: Option<String>,
    pub light: Option<MobLight>,
    pub faction: Option<String>,
    pub vendor: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct MobLight {
    pub range: i32,
    pub color: String,
}

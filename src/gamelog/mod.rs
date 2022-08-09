use rltk::{Point, Rltk, RGB};
mod logstore;
use logstore::*;
pub use logstore::{clear_log, clone_log, log_display, restore_log};
mod builder;
pub use builder::*;

pub struct BattleLog {
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}

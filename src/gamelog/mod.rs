use rltk::{Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
mod logstore;
use logstore::*;
pub use logstore::{clear_log, clone_log, log_display, print_log, restore_log};
mod builder;
pub use builder::*;
mod events;
pub use events::*;

pub struct BattleLog {
    pub entries: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}

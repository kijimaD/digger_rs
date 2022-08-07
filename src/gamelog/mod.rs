use rltk::{Point, Rltk, RGB};
mod logstore;
use logstore::*;
pub use logstore::{clear_log, log_display};
mod builder;
pub use builder::*;

pub struct GameLog {
    pub entries: Vec<String>,
}

pub struct BattleLog {
    pub entries: Vec<String>,
}

pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}

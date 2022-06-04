use super::Renderable;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub blocks_tile: bool,
    pub vision_range: i32,
    pub ai: String,
    pub quips: Option<Vec<String>>
}

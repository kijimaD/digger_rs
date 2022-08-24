use super::Renderable;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Fighter {
    pub name: String,
    pub attributes: MobAttributes,
    pub skills: Option<HashMap<String, i32>>,
    pub natural: Option<MobNatural>,
    pub level: Option<i32>,
    pub hp: Option<i32>,
    pub sp: Option<i32>,
    pub equipped: Option<Vec<String>>,
    pub gold: Option<String>,
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

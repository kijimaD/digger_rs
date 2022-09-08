use rltk::prelude::*;
use specs::prelude::*;
use std::sync::Mutex;

#[derive(Clone)]
pub struct MenuItem {
    pub item: (Entity, Point),
}

pub type ItemType = Mutex<Vec<MenuItem>>;

lazy_static! {
    pub static ref MENU_ITEM: ItemType = Mutex::new(Vec::new());
}

pub fn append_menu(item: MenuItem) {
    MENU_ITEM.lock().unwrap().push(item);
}

pub fn clear_menu() {
    MENU_ITEM.lock().unwrap().clear();
}

pub fn clone_menu() -> Vec<MenuItem> {
    MENU_ITEM.lock().unwrap().clone()
}

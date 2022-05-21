use std:collections::HashMap;
use specs::prelude::*;
use create::components::*;
use super::{Raws};

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws{ items: Vec::new() },
            item_index: HashMap::new()
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }
}

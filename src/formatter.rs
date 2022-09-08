use super::{EquipmentSlot, WeaponAttribute};
use std::fmt;

impl fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EquipmentSlot::Melee => write!(f, "Melee"),
            EquipmentSlot::Shield => write!(f, "Shield"),
            EquipmentSlot::Head => write!(f, "Head"),
            EquipmentSlot::Torso => write!(f, "Torso"),
            EquipmentSlot::Legs => write!(f, "Legs"),
            EquipmentSlot::Feet => write!(f, "Feet"),
            EquipmentSlot::Hands => write!(f, "Hands"),
        }
    }
}

impl fmt::Display for WeaponAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WeaponAttribute::Might => write!(f, "Might"),
            WeaponAttribute::Quickness => write!(f, "Quickness"),
        }
    }
}

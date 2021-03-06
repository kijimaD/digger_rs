use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Stalactite,
    Stalagmite,
    UpStairs,
    DownStairs,
    Road,
    Grass,
    Gravel,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::Floor
        | TileType::UpStairs
        | TileType::DownStairs
        | TileType::Road
        | TileType::Grass
        | TileType::Gravel
        | TileType::ShallowWater
        | TileType::WoodFloor
        | TileType::Bridge => true,
        _ => false,
    }
}

pub fn tile_opaque(tt: TileType) -> bool {
    match tt {
        TileType::Wall | TileType::Stalactite | TileType::Stalagmite => true,
        _ => false,
    }
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}

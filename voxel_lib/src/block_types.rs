use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BlockType {
    Null = 0,
    Air = 1,
    SolidMana
}
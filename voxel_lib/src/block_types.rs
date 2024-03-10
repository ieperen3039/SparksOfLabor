use num_derive::FromPrimitive;

// max 2^18 = 262144 types
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
pub enum BlockType {
    Air = 0,
    Dirt,
    Grass,
    Slate,
    Chest,
}
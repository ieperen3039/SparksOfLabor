use num_derive::FromPrimitive;

// max 262144 types
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
pub enum BlockType {
    Null = 0,
    Air = 1,
    Slate,
}
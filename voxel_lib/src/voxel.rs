use minecraft_protocol::{
    ids::blocks::Block,
    nbt::{self, NbtTag},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Voxel {
    block_id: u32,
    nbt: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VoxelRef<'a> {
    Real(&'a Voxel),
    Inferred(u32),
}

impl Voxel {
    pub fn is_simple(&self) -> bool {
        self.nbt.is_empty()
    }

    pub fn get_block(&self) -> Block {
        Block::from_id(self.block_id).expect("Corrupted voxel")
    }

    pub fn get_block_id(&self) -> u32 {
        self.block_id
    }

    pub fn get_nbt_data(&self) -> NbtTag {
        if self.nbt.is_empty() {
            return NbtTag::Null;
        }
        
        match nbt::parse_nbt(&self.nbt) {
            Ok((result, _)) => return result,
            Err(_) => panic!("Corrupted voxel nbt"),
        }
    }

    pub fn from_id(block: u32) -> Voxel {
        Voxel {
            block_id: block,
            nbt: Vec::new(),
        }
    }
}

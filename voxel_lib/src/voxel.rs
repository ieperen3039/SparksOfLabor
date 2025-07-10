use minecraft_protocol::{
    data::blocks::Block,
    nbt::{self, NbtTag},
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
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
    pub fn from_nbt(id: u32, nbt: NbtTag) -> Voxel {
        let mut voxel = Self::from_id(id);
        nbt.serialize(&mut voxel.nbt);
        return voxel;
    }

    pub fn from_id(id: u32) -> Voxel {
        Voxel {
            block_id: id,
            nbt: Vec::new(),
        }
    }

    pub fn from_block(block: Block) -> Voxel {
        Self::from_id(block as u32)
    }

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

    pub fn is_air(&self) -> bool {
        self.get_block().is_air_block()
    }
}

impl Debug for Voxel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Voxel")
            .field("block_id", &self.block_id)
            .field("nbt", &!self.nbt.is_empty()) // true if present
            .finish()
    }
}

impl VoxelRef<'_> {
    pub fn is_simple(&self) -> bool {
        match self {
            VoxelRef::Inferred(_) => true,
            VoxelRef::Real(v) => v.is_simple(),
        }
    }

    pub fn get_block(&self) -> Block {
        match self {
            VoxelRef::Inferred(id) => Block::from_id(*id).expect("Corrupted voxel"),
            VoxelRef::Real(v) => v.get_block(),
        }
    }

    pub fn get_block_id(&self) -> u32 {
        match self {
            VoxelRef::Inferred(id) => *id,
            VoxelRef::Real(v) => v.get_block_id(),
        }
    }

    pub fn get_nbt_data(&self) -> NbtTag {
        return NbtTag::Null;
    }

    pub fn is_air(&self) -> bool {
        self.get_block().is_air_block()
    }
}

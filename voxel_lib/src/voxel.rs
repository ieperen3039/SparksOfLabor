use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::nbt::{self, NbtTag};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Voxel {
    block_id: BlockWithState,
    nbt: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VoxelRef<'a> {
    /// Reference to an existing Voxel object
    Real(&'a Voxel),
    /// A simple voxel may be constructed from the given id.
    /// See `VoxelRef::get_block`
    Inferred(BlockWithState),
}

impl Voxel {
    pub fn from_nbt(id: BlockWithState, nbt: NbtTag) -> Voxel {
        let mut voxel = Self::from_block(id);
        nbt.serialize(&mut voxel.nbt);
        return voxel;
    }

    pub fn from_block(block: BlockWithState) -> Voxel {
        Voxel {
            block_id: block,
            nbt: Vec::new(),
        }
    }

    pub fn is_simple(&self) -> bool {
        self.nbt.is_empty()
    }

    pub fn get_block(&self) -> BlockWithState {
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
    /// true if there is no nbt data on this block
    pub fn is_simple(&self) -> bool {
        match self {
            VoxelRef::Inferred(_) => true,
            VoxelRef::Real(v) => v.is_simple(),
        }
    }

    pub fn get_block(&self) -> BlockWithState {
        match self {
            VoxelRef::Inferred(id) => *id,
            VoxelRef::Real(v) => v.get_block(),
        }
    }

    pub fn get_nbt_data(&self) -> NbtTag {
        match self {
            VoxelRef::Inferred(_) => NbtTag::Null,
            VoxelRef::Real(v) => v.get_nbt_data(),
        }
    }
}

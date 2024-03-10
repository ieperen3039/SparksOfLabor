
use serde::{Deserialize, Serialize};

use crate::{advanced_voxel::AdvancedVoxel,  block::BaseVoxel} ;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum Voxel {
    // bitfield of type + variant + rotation
    Simple(BaseVoxel),
    // reference to heap-allocated voxel definition
    Advanced(AdvancedVoxel),
}

// not a reference to 1 of 2 types, but 1 of 2 references to a type
// this prevents needing to copy a loose BaseVoxel or AdvancedVoxel just to create a &Voxel
pub enum VoxelRef<'a> {
    Simple(&'a BaseVoxel),
    Advanced(&'a AdvancedVoxel),
}

impl Voxel {
    pub fn as_voxel_ref<'a>(&'a self) -> VoxelRef<'a> {
        match self {
            Voxel::Simple(v) => VoxelRef::Simple(v),
            Voxel::Advanced(v) => VoxelRef::Advanced(v),
        }
    }
}
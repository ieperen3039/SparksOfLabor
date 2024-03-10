
use serde::{Deserialize, Serialize};

use crate:: block::BaseVoxel ;

#[typetag::serde]
pub trait AdvancedVoxel {
    fn get_base_block(&self) -> BaseVoxel;
    fn get_copy(&self) -> Box<dyn AdvancedVoxel>;
}

#[derive(Serialize, Deserialize)]
pub enum Voxel {
    // bitfield of type + variant + rotation
    Simple(BaseVoxel),
    // reference to heap-allocated voxel definition
    Advanced(Box<dyn AdvancedVoxel>),
}

pub enum VoxelRef<'a> {
    // bitfield of type + variant + rotation
    Simple(&'a BaseVoxel),
    // reference to heap-allocated voxel definition
    Advanced(&'a dyn AdvancedVoxel),
}

impl Voxel {
    pub fn as_voxel_ref<'a>(&'a self) -> VoxelRef<'a> {
        match self {
            Voxel::Simple(v) => VoxelRef::Simple(v),
            Voxel::Advanced(v) => VoxelRef::Advanced(v.as_ref()),
        }
    }
}
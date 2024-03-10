use serde::{Deserialize, Serialize};

use crate::{
    block::{BaseVoxel, VoxelOrientation},
    block_types::BlockType,
};

use super::AdvancedVoxelI;

#[derive(Serialize, Deserialize, Eq)]
pub struct ChestVoxel {
    orientation: VoxelOrientation,
}

impl PartialEq for ChestVoxel {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl AdvancedVoxelI for ChestVoxel {
    fn get_base_block(&self) -> BaseVoxel {
        BaseVoxel::new(BlockType::Chest, 0, self.orientation)
    }
}

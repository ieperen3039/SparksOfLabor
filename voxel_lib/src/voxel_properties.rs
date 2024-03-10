use std::collections::HashMap;

use crate::{block_types::BlockType, block::BaseVoxel};


pub struct VoxelProperties {
    texture_index : u32,
}

pub struct VoxelTypeDefinitions {
    map: HashMap<BlockType, VoxelProperties>,
}

impl VoxelTypeDefinitions {
    pub fn get_properties_of(&self, voxel: BaseVoxel) -> Option<&VoxelProperties> {
        self.map.get(&voxel.get_type())
    }
}

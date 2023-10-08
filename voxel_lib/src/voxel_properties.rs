use std::collections::HashMap;

use crate::{block_types::BlockType, voxel::ByteVoxel};


pub struct VoxelProperties {
    texture_index : u32,
}

pub struct VoxelTypeDefinitions {
    map: HashMap<BlockType, VoxelProperties>,
}

impl VoxelTypeDefinitions {
    pub fn get_properties_of(&self, voxel: ByteVoxel) -> Option<&VoxelProperties> {
        self.map.get(&voxel.get_type())
    }
}

use std::collections::HashMap;
use crate::block_types::BlockType;


pub struct VoxelProperties {}

pub struct VoxelTypeDefinitions {
    map: HashMap<BlockType, VoxelProperties>,
}

impl VoxelTypeDefinitions {
    pub fn get_properties_of(&self, voxel: BlockType) -> Option<&VoxelProperties> {
        self.map.get(&voxel)
    }
}

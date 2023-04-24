use std::collections::HashMap;
use crate::block_types::{Block, BlockType};


pub struct VoxelProperties {}

pub struct VoxelTypeDefinitions {
    map: HashMap<BlockType, VoxelProperties>,
}

impl VoxelTypeDefinitions {
    pub fn get_properties_of(&self, voxel: Block) -> Option<&VoxelProperties> {
        self.map.get(&voxel.get_type())
    }
}

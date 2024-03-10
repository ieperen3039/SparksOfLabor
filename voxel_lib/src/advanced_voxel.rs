pub mod chest;

use serde::{Deserialize, Serialize};

use crate::block::BaseVoxel;

use self::chest::ChestVoxel;

pub trait AdvancedVoxelI {
    fn get_base_block(&self) -> BaseVoxel;
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum AdvancedVoxel {
    Chest(Box<ChestVoxel>)
}

impl AdvancedVoxel {
    pub fn as_trait(&self) -> &dyn AdvancedVoxelI {
        match self {
            AdvancedVoxel::Chest(c) => c.as_ref(),
        }
    }
}
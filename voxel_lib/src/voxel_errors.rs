use crate::{block, vector_alias::*};
use simple_error::SimpleError;

#[derive(Debug)]
pub struct VoxelIndexError {
    pub coordinate: Coordinate,
}

#[derive(Debug)]
pub struct UnknownBlockTypeError {
    pub value: block::BaseVoxel,
    pub coordinate: Coordinate,
}

impl std::error::Error for VoxelIndexError {}

impl std::fmt::Display for VoxelIndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "Requested coordinate ({}, {}, {}) ",
            self.coordinate.x, self.coordinate.y, self.coordinate.z
        )
    }
}

impl From<VoxelIndexError> for SimpleError {
    fn from(err: VoxelIndexError) -> Self {
        SimpleError::new(err.to_string())
    }
}

impl std::error::Error for UnknownBlockTypeError {}

impl std::fmt::Display for UnknownBlockTypeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "Unknown block type at ({}, {}, {}) ",
            self.coordinate.x, self.coordinate.y, self.coordinate.z
        )
    }
}

impl From<UnknownBlockTypeError> for SimpleError {
    fn from(err: UnknownBlockTypeError) -> Self {
        SimpleError::new(err.to_string())
    }
}

use crate::vector_alias::*;
use simple_error::SimpleError;

#[derive(Debug)]
pub struct VoxelIndexError {
    pub value: Coordinate,
}

impl std::error::Error for VoxelIndexError {}

impl std::fmt::Display for VoxelIndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "Requested coordiate ({}, {}, {}) ",
            self.value.x, self.value.y, self.value.z
        )
    }
}

impl From<VoxelIndexError> for SimpleError {
    fn from(err: VoxelIndexError) -> Self {
        SimpleError::new(err.to_string())
    }
}

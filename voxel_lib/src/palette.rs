use serde::{Deserialize, Serialize};

use crate::{
    vector_alias::ICoordinate,
    voxel::{Voxel, VoxelRef},
};

#[derive(Serialize, Deserialize)]
pub struct Palette {
    base: Vec<BlockMapping>,
    // every nbt_voxel also exists as a simple voxel
    nbt_voxels: Vec<NbtVoxel>,
    size: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum MappingData {
    // 2^16 = 65536, and there are only 2^12 = 4096 voxels per chunk.
    Simple { num_elements: u16 },
    // forwards to the list of nbt_voxels
    Nbt { idx: u16 },
    Empty,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct BlockMapping {
    block_id: u32,
    data: MappingData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NbtVoxel {
    pub voxel: Voxel,
    pub relative_x: u8,
    pub relative_y: u8,
    pub relative_z: u8,
    is_used: bool,
}

impl Palette {
    pub fn new() -> Palette {
        Palette {
            base: Vec::new(),
            nbt_voxels: Vec::new(),
            size: 0,
        }
    }

    pub fn fill(fill_value: u32) -> Palette {
        Palette {
            base: vec![BlockMapping {
                block_id: fill_value,
                data: MappingData::Simple { num_elements: 1 },
            }],
            nbt_voxels: Vec::new(),
            size: 1,
        }
    }

    pub fn add(&mut self, block_id: u32) -> u16 {
        // first see if it is already in here
        for idx in 0..self.base.len() {
            let block_mapping = &mut self.base[idx];
            if block_mapping.block_id == block_id {
                if let MappingData::Simple { num_elements } = &mut block_mapping.data {
                    *num_elements += 1
                }
            }
        }

        let new_mapping = BlockMapping {
            block_id,
            data: MappingData::Simple { num_elements: 1 },
        };

        // otherwise, find the first gap in the indices; we will use this index
        for idx in 0..self.base.len() {
            let block_mapping = self.base[idx];
            if let MappingData::Empty = block_mapping.data {
                self.base[idx] = new_mapping;
                self.size += 1;

                return idx as u16;
            }
        }
        // no gap in the indices; append
        let new_id = self.base.len() as u16;
        self.base.push(new_mapping);
        self.size += 1;

        return new_id;
    }

    pub fn add_voxel(&mut self, voxel: Voxel, coord: ICoordinate) -> u16 {
        let id = self.add(voxel.get_block_id());

        if voxel.is_simple() {
            return id;
        }
        let new_voxel = NbtVoxel {
            voxel,
            relative_x: coord.x as u8,
            relative_y: coord.y as u8,
            relative_z: coord.z as u8,
            is_used: true,
        };

        // find the first unused element
        for nbt in &mut self.nbt_voxels {
            if !nbt.is_used {
                *nbt = new_voxel;
                return id;
            }
        }

        self.nbt_voxels.push(new_voxel);
        return id;
    }

    pub fn remove(&mut self, id: u16) -> u32 {
        let elt = &mut self.base[id as usize];

        let block_id = elt.block_id;

        match elt.data {
            MappingData::Simple { mut num_elements } => {
                num_elements -= 1;

                if num_elements == 0 {
                    elt.data = MappingData::Empty;
                    self.size -= 1;
                } else {
                    elt.data = MappingData::Simple { num_elements };
                }
            },
            MappingData::Nbt { idx } => {
                self.nbt_voxels[idx as usize].is_used = false;
                elt.data = MappingData::Empty;
            },
            MappingData::Empty => panic!("id was already empty"),
        }

        return block_id;
    }

    pub fn get(&self, id: u16) -> VoxelRef {
        let mapping = &self.base[id as usize];
        match mapping.data {
            MappingData::Simple { .. } => VoxelRef::Inferred(mapping.block_id),
            MappingData::Nbt { idx } => VoxelRef::Real(&self.nbt_voxels[idx as usize].voxel),
            MappingData::Empty => panic!("get on non-existent id"),
        }
    }

    pub fn find(&self, block_id: u32) -> Option<u16> {
        for idx in 0..self.base.len() {
            let elt = &self.base[idx];
            if elt.block_id == block_id {
                return Some(idx as u16);
            }
        }

        unreachable!("Id not found")
    }

    pub fn all(&self) -> Vec<u32> {
        self.base.iter().map(|m| m.block_id).collect()
    }

    pub fn all_nbt_voxels(&self) -> Vec<NbtVoxel> {
        self.nbt_voxels.clone()
    }

    pub fn len(&self) -> usize {
        self.size
    }
}

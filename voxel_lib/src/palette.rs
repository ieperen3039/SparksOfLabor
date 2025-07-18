use minecraft_protocol::components::blocks as mc_blocks;
use minecraft_protocol::data::block_states::BlockWithState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    Nbt { nbt_idx: u16 },
    Empty,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct BlockMapping {
    block_id: BlockWithState,
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

    pub fn fill(fill_value: BlockWithState) -> Palette {
        Palette {
            base: vec![BlockMapping {
                block_id: fill_value,
                data: MappingData::Simple { num_elements: 4096 },
            }],
            nbt_voxels: Vec::new(),
            size: 1,
        }
    }

    pub fn add_simple(&mut self, block_id: BlockWithState) -> u16 {
        // first see if it is already in here
        for idx in 0..self.base.len() {
            let block_mapping = &mut self.base[idx];
            if block_mapping.block_id == block_id {
                if let MappingData::Simple { num_elements } = &mut block_mapping.data {
                    *num_elements += 1;
                    return idx as u16;
                }
            }
        }

        // not already in here; make a new one
        let new_mapping = BlockMapping {
            block_id,
            data: MappingData::Simple { num_elements: 1 },
        };

        self.add_id_internal(new_mapping)
    }

    fn add_id_internal(&mut self, new_mapping: BlockMapping) -> u16 {
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
        new_id
    }

    fn add_nbt_internal(&mut self, new_voxel: NbtVoxel) -> u16 {
        // find the first unused element
        for idx in 0..self.nbt_voxels.len() {
            let mut nbt = &mut self.nbt_voxels[idx];
            if !nbt.is_used {
                *nbt = new_voxel;
                return idx as u16;
            }
        }

        let new_id = self.nbt_voxels.len() as u16;
        self.nbt_voxels.push(new_voxel);
        new_id
    }

    pub fn add_voxel(&mut self, voxel: Voxel, coord: ICoordinate) -> u16 {
        let block_id = voxel.get_block();
        if voxel.is_simple() {
            return self.add_simple(block_id);
        }

        let new_voxel = NbtVoxel {
            voxel,
            relative_x: coord.x as u8,
            relative_y: coord.y as u8,
            relative_z: coord.z as u8,
            is_used: true,
        };

        let nbt_idx = self.add_nbt_internal(new_voxel);

        // nbt voxels always end up as new
        let new_mapping = BlockMapping {
            block_id,
            data: MappingData::Nbt { nbt_idx },
        };

        self.add_id_internal(new_mapping)
    }

    // upgrades a previously added simple block into an nbt block.
    pub fn set_block_entity(&mut self, voxel: mc_blocks::BlockEntity, coord: ICoordinate) -> u16 {
        let block_id = voxel.get_block() as BlockWithState;
        self.remove(
            self.find(block_id)
                .expect("block entity overwrites block id not in this palette"),
        );

        let new_voxel = NbtVoxel {
            voxel: Voxel::from_nbt(block_id, voxel.data),
            relative_x: coord.x as u8,
            relative_y: coord.y as u8,
            relative_z: coord.z as u8,
            is_used: true,
        };

        let nbt_idx = self.add_nbt_internal(new_voxel);

        // nbt voxels always end up as new
        let new_mapping = BlockMapping {
            block_id,
            data: MappingData::Nbt { nbt_idx },
        };

        self.add_id_internal(new_mapping)
    }

    pub fn remove(&mut self, idx: u16) -> BlockWithState {
        let elt = &mut self.base[idx as usize];

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
            MappingData::Nbt { nbt_idx: idx } => {
                self.nbt_voxels[idx as usize].is_used = false;
                elt.data = MappingData::Empty;
            },
            MappingData::Empty => panic!("id was already empty"),
        }

        return block_id;
    }

    pub fn get(&self, idx: u16) -> VoxelRef {
        assert!((idx as usize) < self.base.len(), "get out of bounds");

        let mapping = &self.base[idx as usize];
        match mapping.data {
            MappingData::Simple { .. } => VoxelRef::Inferred(mapping.block_id),
            MappingData::Nbt { nbt_idx: idx } => {
                VoxelRef::Real(&self.nbt_voxels[idx as usize].voxel)
            },
            MappingData::Empty => panic!("get on non-existent id"),
        }
    }

    pub fn find(&self, block_id: BlockWithState) -> Option<u16> {
        for idx in 0..self.base.len() {
            let elt = &self.base[idx];
            if elt.block_id == block_id {
                return Some(idx as u16);
            }
        }

        unreachable!("Id not found")
    }

    /// The returned vector is the mapping from old indices to new indices.
    /// Removed indices will have the value `u16::MAX`
    pub fn remove_holes(&mut self) -> Vec<u16> {
        // TODO also clean up the nbt_voxels list

        let mut new_to_old = Vec::new();
        let mut new_base = Vec::new();

        for block_mapping in self.base.iter() {
            if !matches!(block_mapping.data, MappingData::Empty) {
                let new_index = new_base.len() as u16;
                new_base.push(block_mapping.clone());
                new_to_old.push(new_index);
            } else {
                new_to_old.push(u16::MAX);
            }
        }

        self.base = new_base;

        return new_to_old;
    }

    pub fn get_block_to_id_mapping(&self) -> HashMap<BlockWithState, u16> {
        let mut mapping = HashMap::new();

        for (id, block_mapping) in self.base.iter().enumerate() {
            mapping.insert(block_mapping.block_id, id as u16);
        }

        return mapping;
    }

    pub fn set_to_zero(&mut self) {
        assert_eq!(self.len(), 1);

        if self.base.len() == 1 {
            return;
        }

        let mut only_element = BlockMapping {
            block_id: BlockWithState::default(),
            data: MappingData::Empty,
        };
        for elt in &self.base {
            if let MappingData::Empty = elt.data {
                continue;
            }
            assert!(matches!(only_element.data, MappingData::Empty));
            only_element = elt.clone();
        }

        assert_eq!(matches!(only_element.data, MappingData::Empty), false);

        self.base.clear();
        self.base.push(only_element)
    }

    // returns every unique id of simple voxels
    pub fn all_simple(&self) -> Vec<BlockWithState> {
        self.base
            .iter()
            .filter(|m| matches!(m.data, MappingData::Simple { .. }))
            .map(|m| m.block_id)
            .collect()
    }

    pub fn all_nbt_voxels(&self) -> Vec<NbtVoxel> {
        self.nbt_voxels.clone()
    }

    // number of unique ids
    pub fn len(&self) -> usize {
        self.size
    }
}

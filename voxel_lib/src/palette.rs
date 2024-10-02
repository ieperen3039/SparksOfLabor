use serde::{Deserialize, Serialize};

use crate::voxel::Voxel;

#[derive(Serialize, Deserialize)]
pub struct Palette {
    mapping: Vec<BlockMapping>,
    size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BlockMapping {
    // 2^16 = 65536, and there are only 2^12 = 4096 voxels per chunk.
    num_elements: u16,
    block_type: Voxel,
}

impl Palette {
    pub fn new() -> Palette {
        Palette {
            mapping: Vec::new(),
            size: 0,
        }
    }

    pub fn fill(fill_value: u32) -> Palette {
        Palette {
            mapping: vec![BlockMapping {
                num_elements: 1,
                block_type: Voxel::from_id(fill_value),
            }],
            size: 0,
        }
    }

    pub fn add(&mut self, voxel: Voxel) -> u16 {
        // first see if it is already in here
        for idx in 0..self.mapping.len() {
            let block_mapping = &mut self.mapping[idx];
            if block_mapping.block_type == voxel {
                block_mapping.num_elements += 1;
            }
        }

        // otherwise, find the _last_ gap in the indices; we will use this index
        // (we find the last, not the first, because insertion is cheaper this way)
        for idx in (0..self.mapping.len()).rev() {
            let block_mapping = self.mapping[idx];
            if block_mapping.num_elements == 0 {
                self.mapping[idx] = BlockMapping {
                    num_elements: 1,
                    block_type: voxel,
                };

                self.size += 1;

                return idx as u16;
            }
        }

        // no gap in the indices; append
        let new_id = self.mapping.len() as u16;
        self.mapping.push(BlockMapping {
            num_elements: 1,
            block_type: voxel,
        });

        self.size += 1;

        return new_id;
    }

    pub fn remove(&mut self, id: u16) -> u32 {
        let elt = &mut self.mapping[id as usize];
        elt.num_elements -= 1;
        let block_id = elt.block_type.get_block_id();

        if elt.num_elements == 0 {
            self.size -= 1;
        }

        return block_id;
    }

    pub fn get(&self, id: u16) -> &Voxel {
        &self.mapping[id as usize].block_type
    }

    pub fn find(&self, block_id: u32) -> Option<u16> {
        for idx in 0..self.mapping.len() {
            let elt = &self.mapping[idx];
            if elt.block_type.get_block_id() == block_id {
                return Some(idx as u16);
            }
        }

        unreachable!("Id not found")
    }

    pub fn all(&self) -> Vec<Voxel> {
        self.mapping.iter().map(|m| m.block_type).collect()
    }

    pub fn len(&self) -> usize {
        self.size
    }
}

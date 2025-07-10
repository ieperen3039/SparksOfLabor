use core::panic;
use minecraft_protocol::data::blocks::Block;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::vector_alias::AxisDirection;

const BITS_BLOCK_TYPE_START: u32 = 0;
const NUM_BITS_BLOCK_TYPE: u32 = 18; // 2^18 = 262144 types

const BITS_VARIANT_START: u32 = BITS_BLOCK_TYPE_START + NUM_BITS_BLOCK_TYPE;
const NUM_BITS_VARIANT: u32 = 8; // 128 variants per block type

const BITS_ORIENTATION_START: u32 = BITS_VARIANT_START + NUM_BITS_VARIANT;
const NUM_BITS_ORIENTATION: u32 = VoxelOrientation::NUM_BITS_X + VoxelOrientation::NUM_BITS_Z; // 6: see get_orientation

const TOTAL_NUM_BITS: u32 = NUM_BITS_BLOCK_TYPE + NUM_BITS_VARIANT + NUM_BITS_ORIENTATION;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct BaseVoxel {
    byte: u32,
}

impl BaseVoxel {
    pub fn new(block_type: Block, variant: u32, orientation: VoxelOrientation) -> BaseVoxel {
        let mut voxel = BaseVoxel { byte: 0 };
        voxel.set_type(block_type);
        voxel.set_variant(variant);
        voxel.set_orientation(orientation);
        return voxel;
    }

    pub fn get_type(&self) -> Block {
        Block::from_id(self.get_bits(BITS_BLOCK_TYPE_START, NUM_BITS_BLOCK_TYPE))
            // TODO resilience to corruption and manipulation
            .expect("unknown block type value")
    }

    pub fn get_variant(&self) -> u32 {
        self.get_bits(BITS_VARIANT_START, NUM_BITS_VARIANT)
    }

    pub fn get_orientation(&self) -> VoxelOrientation {
        let x_axis_value =
            self.get_bits(VoxelOrientation::BITS_X_START, VoxelOrientation::NUM_BITS_X);
        let z_raw_offset =
            self.get_bits(VoxelOrientation::BITS_Z_START, VoxelOrientation::NUM_BITS_Z);

        // if z_corrected_offset == 3, then z would map to the opposite of x (which is impossible)
        // if z_corrected_offset == 0 (or 6), then z would be equal to x (which is also impossible)
        let z_corrected_offset = match z_raw_offset {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 5,
            _ => panic!(),
        };
        let z_axis_value = (x_axis_value + z_corrected_offset) % 6;

        VoxelOrientation {
            x_dir: VoxelOrientation::axis_from_u32(x_axis_value),
            z_dir: VoxelOrientation::axis_from_u32(z_axis_value),
        }
    }

    pub fn set_type(&mut self, new_type: Block) {
        self.set_bits(BITS_BLOCK_TYPE_START, NUM_BITS_BLOCK_TYPE, new_type.id());
    }

    pub fn set_variant(&mut self, new_value: u32) {
        self.set_bits(BITS_VARIANT_START, NUM_BITS_VARIANT, new_value);
    }
    pub fn set_orientation(&mut self, new_value: VoxelOrientation) {
        let x_axis_value = VoxelOrientation::axis_to_u32(new_value.x_dir);
        let z_axis_value = VoxelOrientation::axis_to_u32(new_value.z_dir);

        // let z_axis_value = (x_axis_value + z_corrected_offset) % 6;
        let z_value_corrected = (z_axis_value - x_axis_value + 6) % 6;
        // if z_corrected_offset == 3, then z would map to the opposite of x (which is impossible)
        // if z_corrected_offset == 0 (or 6), then z would be equal to x (which is also impossible)
        // Hence, only (1, 2, 4 and 5) are valid values for z_axis.
        // This means that we can encode z in 2 bits
        let z_value_raw = match z_value_corrected {
            1 => 0,
            2 => 1,
            4 => 2,
            5 => 3,
            _ => panic!(),
        };
        self.set_bits(
            VoxelOrientation::BITS_X_START,
            VoxelOrientation::NUM_BITS_X,
            x_axis_value,
        );
        self.set_bits(
            VoxelOrientation::BITS_Z_START,
            VoxelOrientation::NUM_BITS_Z,
            z_value_raw,
        );
    }

    #[inline(always)]
    fn get_bits(&self, first_bit: u32, num_bits: u32) -> u32 {
        debug_assert!((first_bit + num_bits) < 32, "bit range out of bounds");
        // 0b0000_0001 << 6 = 0b0100_0000
        // 0b0100_0000 - 1 = 0b0011_1111 (a mask for 6 bits)
        let mask = (1 << num_bits) - 1;
        (self.byte >> first_bit) & mask
    }

    #[inline(always)]
    fn set_bits(&mut self, first_bit: u32, num_bits: u32, value: u32) {
        debug_assert!((first_bit + num_bits) < 32, "bit range out of bounds");
        debug_assert!(value < (1 << num_bits), "value too large for set_bits");

        // 0b0000_0001 << 6 = 0b0100_0000
        // 0b0100_0000 - 1 = 0b0011_1111 (a mask for 6 bits)
        // 0b0011_1111 << 1 = 0b0111_1110 (a mask for bit 1..=6)
        let mask = ((1 << num_bits) - 1) << first_bit;
        // set the bits of the mask to zero, then OR with the shifted value
        self.byte &= !mask;
        self.byte |= value << first_bit;
    }
}

impl Debug for BaseVoxel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Voxel({:X})", self.byte))
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct VoxelOrientation {
    pub x_dir: AxisDirection,
    pub z_dir: AxisDirection,
}

impl VoxelOrientation {
    const BITS_Z_START: u32 = BITS_ORIENTATION_START;
    const NUM_BITS_Z: u32 = 2; // one of 4
    const BITS_X_START: u32 = VoxelOrientation::BITS_Z_START + VoxelOrientation::NUM_BITS_Z;
    const NUM_BITS_X: u32 = 4; // one of 6

    pub fn new() -> VoxelOrientation {
        VoxelOrientation {
            x_dir: AxisDirection::PosX,
            z_dir: AxisDirection::PosZ,
        }
    }

    fn axis_from_u32(axis_id: u32) -> AxisDirection {
        match axis_id {
            // the exact mapping is used in compacting the z_dir to 2 bits
            0 => AxisDirection::PosX,
            1 => AxisDirection::PosY,
            2 => AxisDirection::PosZ,
            3 => AxisDirection::NegX,
            4 => AxisDirection::NegY,
            5 => AxisDirection::NegZ,
            _ => panic!("axis id larger than 5, but there exist only 6 axes"),
        }
    }

    fn axis_to_u32(axis: AxisDirection) -> u32 {
        match axis {
            AxisDirection::PosX => 0,
            AxisDirection::PosY => 1,
            AxisDirection::PosZ => 2,
            AxisDirection::NegX => 3,
            AxisDirection::NegY => 4,
            AxisDirection::NegZ => 5,
        }
    }
}

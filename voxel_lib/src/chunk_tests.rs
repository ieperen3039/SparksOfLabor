#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_protocol::ids::blocks as mc_ids;
    use crate::vector_alias::Coordinate16;
    use crate::voxel::Voxel;

    #[test]
    fn test_new() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap(); // Assuming 1 is a valid block ID
        let chunk = Chunk16::new(location, fill_value);

        assert_eq!(chunk.num_non_air_blocks, 16 * 16 * 16);
        assert_eq!(chunk.zero_coordinate, Coordinate::from(location));
    }

    #[test]
    fn test_get_voxel() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap();
        let chunk = Chunk16::new(location, fill_value);
        let coord = Coordinate::new(0, 0, 0);

        let voxel_ref = chunk.get_voxel(coord).unwrap();
        assert_eq!(voxel_ref.get_block_id(), fill_value.id());
    }

    #[test]
    fn test_set_voxel() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap();
        let mut chunk = Chunk16::new(location, fill_value);
        let coord = Coordinate::new(0, 0, 0);
        let new_voxel = Voxel::from_block(mc_ids::Block::from_id(2).unwrap()); // Assuming 2 is a valid block ID

        chunk.set_voxel(coord, new_voxel).unwrap();
        let voxel_ref = chunk.get_voxel(coord).unwrap();
        assert_eq!(voxel_ref.get_block_id(), new_voxel.get_block_id());
    }

    #[test]
    fn test_from_minecraft() {
        // Assuming mc_chunk::Chunk and Coordinate16 are properly defined and usable here
        let mc_chunk = mc_chunk::Chunk {
            // Initialize with appropriate values
        };
        let position = Coordinate16::new(0, 0, 0);
        let chunk = Chunk16::from_minecraft(&mc_chunk, position);

        // Add assertions based on expected values
    }

    #[test]
    fn test_to_minecraft() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap();
        let chunk = Chunk16::new(location, fill_value);

        let (mc_chunk, coord, block_entities) = Chunk16::to_minecraft(&chunk);

        // Add assertions based on expected values
    }
	
	fn add_voxel(chunk: &mut Chunk16, i: i32) {
		let new_voxel = Voxel::from_block(mc_ids::Block::from_id(i).unwrap());
		let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
		chunk.set_voxel(coord, new_voxel).unwrap();
	}
	
	fn check_voxels(chunk: &Chunk16, max: i32) -> bool {
		// check 0..max has ids as expected
		for i in 0..max {
			let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
			let voxel_ref = chunk.get_voxel(coord).unwrap();

			let expected_id = mc_ids::Block::from_id(i).unwrap();
			if voxel_ref.get_block_id() != expected_id {
				return false;
			}
		}
		
		// check the rest is id 1
		for i in max..4096 {
			let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
			let voxel_ref = chunk.get_voxel(coord).unwrap();

			let expected_id = mc_ids::Block::from_id(1).unwrap();
			if voxel_ref.get_block_id() != expected_id {
				return false;
			}
		}
		true
	}

    #[test]
    fn test_upgrade() {
		const MAX_PALETTES_B8: usize = (1 << 8) - 1; // 255
		const MAX_PALETTES_B4: usize = (1 << 4) - 1; // 15
		const MAX_PALETTES_B2: usize = (1 << 2) - 1; // 3
		const MAX_PALETTES_B0: usize = 1;
		
		const B32_START: usize = MAX_PALETTES_B8 + 1;
		const B8_START: usize = MAX_PALETTES_B4 + 1;
		const B4_START: usize = MAX_PALETTES_B2 + 1;
		const B2_START: usize = MAX_PALETTES_B0 + 1;


        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(0).unwrap(); // Assuming 0 is a valid block ID
        let mut chunk = Chunk16::new(location, fill_value);

        // Initially, the grid should be B0
        assert!(matches!(chunk.grid, Chunk16Grid::B0));
		
		// Adding a single voxel should cause an upgrade
		add_voxel(chunk, B2_START);
        assert!(matches!(chunk.grid, Chunk16Grid::B2(_)));
		
		check_voxels(chunk, B2_START);

        // Add more voxels to exceed the B2 limit and trigger another upgrade
        for i in 1..=MAX_PALETTES_B2 {
            add_voxel(chunk, i);
        }
		
        assert!(matches!(chunk.grid, Chunk16Grid::B2(_)));
		add_voxel(chunk, B4_START);
        assert!(matches!(chunk.grid, Chunk16Grid::B4(_)));
		
		check_voxels(chunk, B4_START);

        // Add more voxels to exceed the B4 limit and trigger another upgrade
        for i in B4_START..=MAX_PALETTES_B4 {
            add_voxel(chunk, i);
        }

        assert!(matches!(chunk.grid, Chunk16Grid::B4(_)));
		add_voxel(chunk, B8_START);
        assert!(matches!(chunk.grid, Chunk16Grid::B8(_)));
		
		check_voxels(chunk, B8_START);

        // Add more voxels to exceed the B8 limit and trigger another upgrade
        for i in B8_START..=MAX_PALETTES_B8 {
            add_voxel(chunk, i);
        }

        // After adding enough unique voxels, the grid should upgrade to B32
        assert!(matches!(chunk.grid, Chunk16Grid::B8(_)));
		add_voxel(chunk, B32_START);
        assert!(matches!(chunk.grid, Chunk16Grid::B32(_)));
		
		check_voxels(chunk, B32_START);
    }
	
	fn clear_voxel(chunk: &mut Chunk16, i: i32) {
		// like set_voxel, but setting it to 1
		let new_voxel = Voxel::from_block(mc_ids::Block::from_id(1).unwrap());
		let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
		chunk.set_voxel(coord, new_voxel).unwrap();
	}
	
    #[test]
    fn test_downgrade() {
		const MAX_PALETTES_B8: usize = (1 << 8) - 1; // 255
		const MAX_PALETTES_B4: usize = (1 << 4) - 1; // 15
		const MAX_PALETTES_B2: usize = (1 << 2) - 1; // 3
		const MAX_PALETTES_B0: usize = 1;
		
		const B32_END: usize = MAX_PALETTES_B8 - PALETTE_HYSTERESIS_VALUE;
		const B8_END: usize = MAX_PALETTES_B4 - PALETTE_HYSTERESIS_VALUE;
		const B4_END: usize = MAX_PALETTES_B2 - PALETTE_HYSTERESIS_VALUE;
		
		const PALETTE_HYSTERESIS_VALUE: usize = 1;
		
		for i in 1..=4096 {
            add_voxel(chunk, i);
        }
		
		check_voxels(chunk, 4096);
		
        assert!(matches!(chunk.grid, Chunk16Grid::B32(_)));
		
		for i in B32_END..=4096 {
            clear_voxel(chunk, i);
        }
		
        assert!(matches!(chunk.grid, Chunk16Grid::B8(_)));
		check_voxels(chunk, B32_END);
		
		for i in B8_END..=B32_END {
            clear_voxel(chunk, i);
        }
		
        assert!(matches!(chunk.grid, Chunk16Grid::B4(_)));
		check_voxels(chunk, B8_END);
		
		for i in B4_END..=B8_END {
            clear_voxel(chunk, i);
        }
		
        assert!(matches!(chunk.grid, Chunk16Grid::B2(_)));
		check_voxels(chunk, B4_END);
		
		// now clear everything
		for i in 1..=B4_END {
			clear_voxel(chunk, i);
		}
		
		// we require that if there is only 1 voxel type, that we delete the grid regardless of hysteresis
		assert!(matches!(chunk.grid, Chunk16Grid::B0(_)));
	}
}

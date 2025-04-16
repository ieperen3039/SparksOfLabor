use minecraft_protocol::ids::blocks as mc_ids;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk16::Chunk16;
    use crate::vector_alias::*;
    use crate::voxel::Voxel;
    use crate::voxel_errors::VoxelIndexError;

    #[test]
    fn test_get_voxel() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap();
        let chunk = Chunk16::new(location, fill_value);
        assert_eq!(chunk.zero_coordinate(), Coordinate::from(location));

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
        let block_id = 2;
        let new_voxel = Voxel::from_block(mc_ids::Block::from_id(block_id).unwrap());

        chunk.set_voxel(coord, new_voxel).unwrap();
        let voxel_ref = chunk.get_voxel(coord).unwrap();
        assert_eq!(voxel_ref.get_block_id(), block_id);
    }

    #[test]
    fn test_upgrade() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(0).unwrap();
        let mut chunk = Chunk16::new(location, fill_value);

        let mut min = 1;
        for shift in 1..=9 {
            // every doubling we want to check (chance on upgrade)
            // we only go up to 2^9 = 512 because there are less than 1024 blocks in minecraft
            // (and we use `mc_ids::Block::from_id` to create blocks)
            let max = (1 << shift) - 1;
            for i in min..=max {
                add_voxel(&mut chunk, i);
            }

            check_voxels(&chunk, max);
        }

        let max = min;
        for shift in 8..=1 {
            let min = (1 << shift) - 1;
            for i in min..=max {
                clear_voxel(&mut chunk, i);
            }

            check_voxels(&chunk, max);
        }
    }

    #[test]
    fn test_from_to_minecraft() {
        let location = Coordinate16::new(0, 0, 0);
        let fill_value = mc_ids::Block::from_id(0).unwrap();
        let mut original_chunk = Chunk16::new(location, fill_value);

        // iterate in much the same way as the upgrade test does
        let min = 1;
        for shift in 1..=9 {
            let max = (1 << shift) - 1;
            for i in min..=max {
                add_voxel(&mut original_chunk, i);
            }

            let (mc_chunk, coord, block_entities) = original_chunk.to_minecraft();
            let new_chunk = Chunk16::from_minecraft(&mc_chunk, coord, block_entities);

            check_voxels(&new_chunk, max);
        }
    }

    #[test]
    fn test_set_voxel_out_of_bounds() {
        let location = Coordinate16::new(1, 0, 0);
        let fill_value = mc_ids::Block::from_id(1).unwrap(); // Assuming 1 is a valid block ID
        let mut chunk = Chunk16::new(location, fill_value);

        // Define an out-of-bounds coordinate
        let out_of_bounds_coord = Coordinate::new(0, 0, 0); // x is out of bounds
        let new_voxel = Voxel::from_block(mc_ids::Block::from_id(2).unwrap()); // Assuming 2 is a valid block ID

        // Attempt to set a voxel at the out-of-bounds coordinate
        let result = chunk.set_voxel(out_of_bounds_coord, new_voxel);

        // Check that the result is an error
        assert!(result.is_err());

        // Check that the error is of the correct type
        if let Err(VoxelIndexError { coordinate }) = result {
            assert_eq!(coordinate, out_of_bounds_coord);
        } else {
            panic!("Expected VoxelIndexError");
        }
    }

    fn add_voxel(chunk: &mut Chunk16, i: i32) {
        let new_voxel = Voxel::from_block(mc_ids::Block::from_id(i as u32).unwrap());
        let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
        chunk.set_voxel(coord, new_voxel).unwrap();
    }

    fn check_voxels(chunk: &Chunk16, max: i32) -> bool {
        // check `0..max` has ids as expected
        for i in 0..max {
            let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
            let voxel_ref = chunk.get_voxel(coord).unwrap();

            if voxel_ref.get_block_id() as i32 != i {
                return false;
            }
        }

        // check the rest is id 1
        for i in max..4096 {
            let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
            let voxel_ref = chunk.get_voxel(coord).unwrap();

            if voxel_ref.get_block_id() != 1 {
                return false;
            }
        }
        true
    }

    fn clear_voxel(chunk: &mut Chunk16, i: i32) {
        // like set_voxel, but setting it to 1
        let new_voxel = Voxel::from_block(mc_ids::Block::from_id(1).unwrap());
        let coord = Coordinate::new(i % 16, (i / 16) % 16, (i / 256) % 16);
        chunk.set_voxel(coord, new_voxel).unwrap();
    }
}

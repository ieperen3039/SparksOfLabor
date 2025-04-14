#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::{Voxel, VoxelRef};
    use crate::vector_alias::ICoordinate;

    #[test]
    fn test_fill() {
        let fill_value = 42;
        let palette = Palette::fill(fill_value);
        assert_eq!(palette.base.len(), 1);
        assert_eq!(palette.base[0].block_id, fill_value);
        assert_eq!(palette.size, 1);
    }

    #[test]
    fn test_add() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add(block_id);
        assert_eq!(palette.base.len(), 1);
        assert_eq!(palette.base[id as usize].block_id, block_id);
        assert_eq!(palette.size, 1);
    }

    #[test]
    fn test_add_voxel() {
        let mut palette = Palette::new();
        let voxel = Voxel::new(42); // Assuming Voxel::new exists
        let coord = ICoordinate { x: 1, y: 2, z: 3 };
        let id = palette.add_voxel(voxel.clone(), coord);
        assert_eq!(palette.base.len(), 1);
        assert_eq!(palette.nbt_voxels.len(), 1);
        assert_eq!(palette.size, 1);
    }

    #[test]
    fn test_remove() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add(block_id);
        let removed_block_id = palette.remove(id);
        assert_eq!(removed_block_id, block_id);
        assert_eq!(palette.size, 0);
    }

    #[test]
    fn test_get() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add(block_id);
        let voxel_ref = palette.get(id);
        match voxel_ref {
            VoxelRef::Inferred(bid) => assert_eq!(bid, block_id),
            _ => panic!("Expected VoxelRef::Inferred"),
        }
    }

    #[test]
    fn test_find() {
        let mut palette = Palette::new();
        let block_id = 42;
        palette.add(block_id);
        let found_id = palette.find(block_id).unwrap();
        assert_eq!(palette.base[found_id as usize].block_id, block_id);
    }

    #[test]
    fn test_all() {
        let mut palette = Palette::new();
        let block_ids = vec![1, 2, 3];
        for &id in &block_ids {
            palette.add(id);
        }
        let all_ids = palette.all();
        assert_eq!(all_ids, block_ids);
    }

    #[test]
    fn test_all_nbt_voxels() {
        let mut palette = Palette::new();
        let voxel = Voxel::new(42); // Assuming Voxel::new exists
        let coord = ICoordinate { x: 1, y: 2, z: 3 };
        palette.add_voxel(voxel.clone(), coord);
        let all_nbt_voxels = palette.all_nbt_voxels();
        assert_eq!(all_nbt_voxels.len(), 1);
    }

    #[test]
    fn test_len() {
        let mut palette = Palette::new();
        assert_eq!(palette.len(), 0);
        palette.add(42);
        assert_eq!(palette.len(), 1);
    }
}

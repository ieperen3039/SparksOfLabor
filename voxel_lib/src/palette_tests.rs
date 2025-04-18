#[cfg(test)]
mod tests {
    use crate::palette::Palette;
    use crate::vector_alias::ICoordinate;
    use crate::voxel::{Voxel, VoxelRef};
    use minecraft_protocol::components::blocks::BlockEntity;
    use minecraft_protocol::ids::blocks::Block;
    use minecraft_protocol::nbt::NbtTag;

    #[test]
    fn test_new() {
        let mut palette = Palette::new();
        assert_eq!(palette.len(), 0);
        assert_eq!(palette.all_simple().len(), 0);
        assert_eq!(palette.all_nbt_voxels().len(), 0);
    }

    #[test]
    fn test_all_simple() {
        let mut palette = Palette::new();
        let block_ids = vec![1, 2, 3];
        for &id in &block_ids {
            palette.add_simple(id);
        }

        let mut all_ids = palette.all_simple();
        all_ids.sort();
        assert_eq!(all_ids, block_ids);
    }

    #[test]
    fn test_add_simple_get() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add_simple(block_id);

        assert_eq!(palette.len(), 1);
        assert_eq!(palette.all_simple().len(), 1);
        assert_eq!(palette.all_nbt_voxels().len(), 0);

        assert_eq!(palette.get(0).get_block_id(), block_id);
    }

    #[test]
    fn test_fill() {
        let fill_value = 42;
        let palette = Palette::fill(fill_value);
        assert_eq!(palette.len(), 1);
        assert_eq!(palette.get(0).get_block_id(), fill_value);
    }

    #[test]
    fn test_remove() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add_simple(block_id);

        assert_eq!(palette.len(), 1);
        assert_eq!(palette.all_simple().len(), 1);
        assert_eq!(palette.all_nbt_voxels().len(), 0);

        let removed_block_id = palette.remove(id);
        assert_eq!(removed_block_id, block_id);
        assert_eq!(palette.len(), 0);
        assert_eq!(palette.all_simple().len(), 0);
        assert_eq!(palette.all_nbt_voxels().len(), 0);
    }

    #[test]
    fn test_add_multiple_different() {
        let mut palette = Palette::new();
        palette.add_simple(32);
        palette.add_simple(52);
        palette.add_simple(42);

        assert_eq!(palette.len(), 3);
        assert_eq!(palette.all_simple().len(), 3);
        assert_eq!(palette.all_nbt_voxels().len(), 0);

        let mut all = palette.all_simple();
        all.sort();
        assert_eq!(all, [32, 42, 52]);
    }

    #[test]
    fn test_add_multiple_same() {
        let mut palette = Palette::new();
        palette.add_simple(32);
        palette.add_simple(42);
        palette.add_simple(32);
        palette.add_simple(42);

        // len should return number of unique ids
        assert_eq!(palette.len(), 2);
        assert_eq!(palette.all_simple().len(), 2);
        assert_eq!(palette.all_nbt_voxels().len(), 0);

        // all returns every unique id once
        let mut all = palette.all_simple();
        all.sort();
        assert_eq!(all, [32, 42]);
    }

    #[test]
    fn test_remove_multiple() {
        let mut palette = Palette::new();
        let id1 = palette.add_simple(32);
        let id2 = palette.add_simple(42);
        let id3 = palette.add_simple(32);
        let id4 = palette.add_simple(42);
        assert_eq!(palette.len(), 2);

        palette.remove(id1);
        assert_eq!(palette.len(), 2);

        palette.remove(id3);
        assert_eq!(palette.len(), 1);

        // all returns every unique id once
        assert_eq!(palette.all_simple(), [42]);

        let id5 = palette.add_simple(52);
        assert_eq!(palette.len(), 2);

        let mut all = palette.all_simple();
        all.sort();
        assert_eq!(all, [42, 52]);
    }

    #[test]
    fn test_add_voxel() {
        let mut palette = Palette::new();
        let simple_voxel = Voxel::from_id(42);
        let coord = ICoordinate::new(1, 2, 3);
        let id1 = palette.add_voxel(simple_voxel.clone(), coord);
        assert_eq!(palette.len(), 1);
        assert_eq!(palette.all_simple().len(), 1);

        // same id, but with tag
        let nbt_voxel = Voxel::from_nbt(42, NbtTag::Int(24));
        let coord2 = ICoordinate::new(3, 4, 5);
        let id2 = palette.add_voxel(nbt_voxel.clone(), coord2);
        // 2 different elements in the palette: one simple, one nbt
        assert_eq!(palette.len(), 2);
        assert_eq!(palette.all_simple().len(), 1);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_all_nbt_voxels() {
        let mut palette = Palette::new();

        let voxel1 = Voxel::from_nbt(32, NbtTag::Int(24));
        let coord1 = ICoordinate::new(1, 2, 3);
        palette.add_voxel(voxel1.clone(), coord1);

        let voxel2 = Voxel::from_nbt(42, NbtTag::Int(24));
        let coord2 = ICoordinate::new(3, 4, 5);
        palette.add_voxel(voxel2.clone(), coord2);

        let mut all_nbt_voxels = palette.all_nbt_voxels();
        assert_eq!(all_nbt_voxels.len(), 2);
        all_nbt_voxels.sort_by_key(|v| v.voxel.get_block_id());

        let block_ids: Vec<&Voxel> = all_nbt_voxels.iter().map(|v| &v.voxel).collect();
        assert_eq!(block_ids, [&voxel1, &voxel2]);
    }

    #[test]
    fn test_get_inferred() {
        let mut palette = Palette::new();
        let block_id = 42;
        let id = palette.add_simple(block_id);
        let voxel_ref = palette.get(id);
        match voxel_ref {
            VoxelRef::Inferred(bid) => assert_eq!(bid, block_id),
            _ => panic!("Expected VoxelRef::Inferred"),
        }
    }

    #[test]
    fn test_get_real() {
        let mut palette = Palette::new();
        let voxel = Voxel::from_nbt(42, NbtTag::Int(24));
        let coord = ICoordinate::new(1, 2, 3);
        let id = palette.add_voxel(voxel.clone(), coord);

        let voxel_ref = palette.get(id);
        match voxel_ref {
            VoxelRef::Real(bid) => {
                assert_eq!(bid.get_block_id(), voxel.get_block_id());
                assert_eq!(bid.get_nbt_data(), voxel.get_nbt_data());
            },
            _ => panic!("Expected VoxelRef::Real"),
        }
    }

    #[test]
    fn test_find() {
        let mut palette = Palette::new();
        palette.add_simple(32);
        let id = palette.add_simple(42);
        palette.add_simple(52);

        let found_id = palette.find(42);
        assert_eq!(found_id, Some(id));
    }

    #[test]
    fn test_set_block_entity() {
        let mut palette = Palette::new();
        let id1 = palette.add_simple(2);
        let id2 = palette.add_simple(42);
        let id3 = palette.add_simple(2);
        let id4 = palette.add_simple(42);

        assert_eq!(palette.len(), 2);

        let id5 = palette.set_block_entity(
            BlockEntity::new(0, 0, 0, Block::from_id(2).unwrap(), NbtTag::Int(24)),
            ICoordinate::new(0, 0, 0),
        );

        assert_eq!(palette.len(), 3);

        // location doesnt matter for palette
        let id6 = palette.set_block_entity(
            BlockEntity::new(0, 0, 0, Block::from_id(2).unwrap(), NbtTag::Int(24)),
            ICoordinate::new(0, 0, 0),
        );

        assert_ne!(id5, id6);
        // all simple (id == 2) have been removed, so we still have 3 unique ids
        assert_eq!(palette.len(), 3);
        assert_eq!(palette.all_simple().len(), 1);
        assert_eq!(palette.all_nbt_voxels().len(), 2);
    }

    #[test]
    fn test_remove_holes_and_generate_mapping() {
        let mut palette = Palette::new();
        
        // Add some mappings to the palette
        palette.add_simple(1);
        let id2 = palette.add_simple(2);
        palette.add_simple(3);
        let id4 = palette.add_simple(4);
        palette.add_simple(5);

        assert_eq!(palette.len(), 5);
        palette.remove(id2);
        assert_eq!(palette.len(), 4);
        palette.remove(id4);
        assert_eq!(palette.len(), 3);

        // 1 stays at index 0, 
        // 3 goes from index 2 to index 1, 
        // 5 goes from index 4 to index 2
        let expected_mapping = vec![0, 2, 4];
        let actual_mapping = palette.remove_holes();

        assert_eq!(palette.len(), 3);

        assert_eq!(actual_mapping, expected_mapping);
    }
}

use nalgebra::Vector3;
use simple_error::SimpleError;
use sol_voxel_lib::{
    asset::Assets,
    block_types::BlockType,
    vector_alias::{
        AxisDirection, Coordinate, Coordinate64, ICoordinate,
    },
    voxel::{ByteVoxel, VoxelOrientation},
    world::World,
};

use super::mesh::{self, Mesh, Vertex};

fn add_quad(
    vertices: &mut Vec<mesh::Vertex>,
    indices: &mut Vec<u32>,
    coord: Coordinate,
    direction: AxisDirection,
    block_type: ByteVoxel,
) {
    let normal = direction.get_unit();
    let voxel_middle = Vector3::new(coord.x as f32, coord.y as f32, coord.z as f32).add_scalar(0.5);
    

    for a in [-0.5, 0.5] {
        for b in [-0.5, 0.5] {
            let offset = match direction {
                AxisDirection::PosX => Vector3::new(0.5, a, b),
                AxisDirection::PosY => Vector3::new(a, 0.5, b),
                AxisDirection::PosZ => Vector3::new(a, b, 0.5),
                AxisDirection::NegX => Vector3::new(-0.5, a, b),
                AxisDirection::NegY => Vector3::new(a, -0.5, b),
                AxisDirection::NegZ => Vector3::new(a, b, -0.5),
            };
            let pos = voxel_middle + offset;
            vertices.push(Vertex::new(pos, normal, [a, b]));
        }
    }
}

fn create_mesh(
    display: &glium::Display,
    assets: &Assets,
    world: &World,
    target: Coordinate64,
    texture: glium::texture::RawImage2d<'_, u8>,
) -> Result<Mesh, SimpleError> {
    let chunk64 = {
        let chunk = world.get_chunk(target);

        if chunk.is_none() {
            return Mesh::empty(display, texture);
        }
        chunk.unwrap()
    };

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let default_air = ByteVoxel::new(BlockType::Air, 0, VoxelOrientation::new());
    let mut voxel_grid = [[[default_air; 66]; 66]; 66];

    chunk64.for_each(|coord, voxel| {
        voxel_grid[coord.x + 1][coord.y + 1][coord.z + 1] = voxel.get_base();
    });

    // we iterate over the inner 64x64x64 of voxel_grid, leaving one layer of air around the outside
    for x in 1..65usize {
        for y in 1..65usize {
            for z in 1..65usize {
                let coord =
                    chunk64.get_coordinate_from_index(ICoordinate::new(x - 1, y - 1, z - 1));

                let this_voxel = voxel_grid[x][y][z];
                if this_voxel.get_type() == BlockType::Air { continue; }

                let x_pos_is_air = voxel_grid[x + 1][y][z].get_type() == BlockType::Air;
                let x_neg_is_air = voxel_grid[x - 1][y][z].get_type() == BlockType::Air;
                let y_pos_is_air = voxel_grid[x][y + 1][z].get_type() == BlockType::Air;
                let y_neg_is_air = voxel_grid[x][y - 1][z].get_type() == BlockType::Air;
                let z_pos_is_air = voxel_grid[x][y][z + 1].get_type() == BlockType::Air;
                let z_neg_is_air = voxel_grid[x][y][z - 1].get_type() == BlockType::Air;

                if x_pos_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::PosX, this_voxel)}
                if x_neg_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::NegX, this_voxel)}
                if y_pos_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::PosY, this_voxel)}
                if y_neg_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::NegY, this_voxel)}
                if z_pos_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::PosZ, this_voxel)}
                if z_neg_is_air { add_quad(&mut vertices, &mut indices, coord, AxisDirection::NegZ, this_voxel)}
            }
        }
    }

    Mesh::from_arrays(display, vertices, indices, texture)
}

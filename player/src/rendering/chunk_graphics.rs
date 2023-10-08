use simple_error::SimpleError;
use sol_voxel_lib::{world::World, vector_alias::{Coordinate64, coordinate64_to_absolute, Coordinate}};

use super::mesh::Mesh;

fn create_mesh(display: &glium::Display, world: &World, target: Coordinate64, texture : glium::texture::RawImage2d<'_, u8>) -> Result<Mesh, SimpleError> {
    let chunk = world.get_chunk(target);

    if chunk.is_none() {
        return Mesh::empty(display, texture);
    }
    let chunk = chunk.unwrap();

    let vertices = Vec::new();
    let indices = Vec::new();

    let coord_base = coordinate64_to_absolute(target);

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let coord = coord_base + Coordinate::new(x, y, z);
                let properties = chunk.get_chunk16(coord).unwrap().get_properties(definitions, coord)?;
                
            }
        }
    }

    Mesh::from_arrays(display, vertices, indices, texture)
}
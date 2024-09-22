use minecraft_protocol::components::chunk as mc_chunk;
use crate::{chunk as sol_chunk, vector_alias::Coordinate16, voxel::Voxel};



pub fn from_minecraft(mc_chunk : mc_chunk::Chunk, position: Coordinate16) -> sol_chunk::Chunk16 {
    match mc_chunk.blocks {
        mc_chunk::PalettedData::Paletted { palette, indexed } => {
            sol_chunk::Chunk16::from_raw(&indexed, &palette, position)
        },
        mc_chunk::PalettedData::Single { value } => {
            let sol_chunk = sol_chunk::Chunk16::new(position, Voxel::from_id(value));

            return sol_chunk;
        },
        mc_chunk::PalettedData::Raw { values } => todo!(),
    }
}

pub fn to_minecraft(sol_chunk : sol_chunk::Chunk16, position: Coordinate16) -> mc_chunk::Chunk {
    todo!()
}
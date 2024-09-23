use crate::{
    chunk as sol_chunk,
    vector_alias::{self, Coordinate16},
};
use minecraft_protocol::{components::chunk as mc_chunk, ids::blocks::Block};

impl sol_chunk::Chunk16 {
    pub fn from_minecraft(mc_chunk: mc_chunk::Chunk, position: Coordinate16) -> sol_chunk::Chunk16 {
        match mc_chunk.blocks {
            mc_chunk::PalettedData::Paletted { palette, indexed } => {
                sol_chunk::Chunk16::from_raw_palette(&indexed, &palette, position)
            },
            mc_chunk::PalettedData::Single { value } => {
                // TODO from_id or from_state_id?
                sol_chunk::Chunk16::new(position, Block::from_id(value).expect("unkown block id"))
            },
            mc_chunk::PalettedData::Raw { values } => {
                sol_chunk::Chunk16::from_direct(&values, position)
            },
        }
    }

    pub fn to_minecraft(sol_chunk: sol_chunk::Chunk16) -> (mc_chunk::Chunk, Coordinate16) {
        let byte_size = sol_chunk.get_grid_element_byte_size();
        let blocks = if byte_size >= 8 {
            mc_chunk::PalettedData::Raw {
                values: sol_chunk.as_direct().unwrap(),
            }
        } else if byte_size == 0 {
            mc_chunk::PalettedData::Single {
                value: sol_chunk.as_single().unwrap(),
            }
        } else {
            todo!()
        };

        return (
            mc_chunk::Chunk {
                block_count: sol_chunk.get_num_non_air_blocks() as i16,
                blocks,
                biomes: mc_chunk::PalettedData::Single { value: 0 },
            },
            sol_chunk.get_zero_coordinate() / 16,
        );
    }
}

use sol_voxel_lib::vector_alias::{Position, Rotation};

pub struct PlayerCharacter {
    pub entity_id: u32,
    pub uuid: [i32; 4],
    pub position: Position,
    pub head_rotation: Rotation,
    // TODO
}

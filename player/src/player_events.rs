use minecraft_protocol::components::slots::Hand;
use sol_voxel_lib::vector_alias::Coordinate;

pub struct PlayerPlaceBlockEvent {
    pub hand: Hand,
    pub location: Coordinate,
    pub face: minecraft_protocol::components::blocks::BlockFace,
    pub cursor_position_x: f32,
    pub cursor_position_y: f32,
    pub cursor_position_z: f32,
    pub inside_block: bool,
}
use crate::game_event::Event;
use crate::item_stack::ItemStack;
use crate::player_events::PlayerPlaceBlockEvent;
use crate::player_state::{PlayerState, PLAYER_OFF_HAND_SLOT};
use crate::voxels::world::World;
use minecraft_protocol::components::blocks::BlockFace;
use minecraft_protocol::components::slots::Hand;
use minecraft_registries::block_placement_registry::BlockPlacementParameters;
use minecraft_registries::item_click_registry::ItemClickEvent;
use minecraft_vanilla::registries::Registries;
use sol_log_server::logger_mt::LoggerMt;
use sol_voxel_lib::vector_alias::Coordinate;
use sol_voxel_lib::voxel::Voxel;

pub struct PlayerHandler {
    logger: LoggerMt,
    player_state: PlayerState,
}

impl PlayerHandler {
    pub fn new(player_state: PlayerState, logger: LoggerMt) -> PlayerHandler {
        PlayerHandler {
            logger,
            player_state,
        }
    }
}

// NOTE: removes the item from the player but does not add the block to the world
pub fn handle_block_place_event(
    command: PlayerPlaceBlockEvent,
    player: &mut PlayerState,
    world: &World,
    registries: &Registries,
) -> Option<Event> {
    let slot_idx = match command.hand {
        Hand::MainHand => player.selected_slot,
        Hand::OffHand => PLAYER_OFF_HAND_SLOT,
    };

    let stack = &mut player.slots[slot_idx];

    let placed_item = match stack.take_one() {
        ItemStack::Empty => return None,
        ItemStack::Simple(stack) => stack.item_type(),
        ItemStack::NbtItem(item) => item.item_type(),
    };

    let player_facing_normalized = player.look_direction;

    let how = BlockPlacementParameters {
        block_face: command.face.clone(),
        cursor_position_x: command.cursor_position_x,
        cursor_position_y: command.cursor_position_y,
        cursor_position_z: command.cursor_position_z,
        player_facing_normalized_x: player_facing_normalized.x,
        player_facing_normalized_y: player_facing_normalized.y,
        player_facing_normalized_z: player_facing_normalized.z,
        inside_block: false,
    };

    let target_block = world.get_block(command.location);

    let event = registries.get_item_click_event(placed_item, target_block, how);

    match event {
        ItemClickEvent::Nothing => None,
        ItemClickEvent::Something => todo!(),
        ItemClickEvent::BlockPlacement {
            block,
            change,
            replace,
        } => {
            player.handle_item_change(slot_idx, change);
            if replace {
                Some(Event::VoxelChange {
                    coord: command.location,
                    new_voxel: Voxel::from_block(block),
                })
            } else {
                let new_position = command.location + block_face_to_difference(command.face);
                Some(Event::VoxelChange {
                    coord: new_position,
                    new_voxel: Voxel::from_block(block),
                })
            }
        },
        ItemClickEvent::Eat { .. } => None,
        ItemClickEvent::EntitySpawn { .. } => None,
        ItemClickEvent::EntityThrow { .. } => None,
    }
}

/// given a block coordinate, returns the coordinate _difference_ toward the block at the given block face.
/// ```rust
/// let adjacent_location = location + block_face_to_difference(face);
/// ```
fn block_face_to_difference(face: BlockFace) -> Coordinate {
    match face {
        BlockFace::Bottom => Coordinate::new(0, -1, 0),
        BlockFace::Top => Coordinate::new(0, 1, 0),
        BlockFace::North => Coordinate::new(0, 0, -1),
        BlockFace::South => Coordinate::new(0, 0, 1),
        BlockFace::West => Coordinate::new(-1, 0, 0),
        BlockFace::East => Coordinate::new(1, 0, 0),
    }
}

use crate::game_event::Event;
use crate::game_loop::GameCommand;
use crate::item_stack::ItemStack;
use crate::minecraft_connection::client_connection::ClientSendCommand;
use crate::player_state::{PlayerState, PLAYER_OFF_HAND_SLOT};
use minecraft_protocol::components::slots;
use minecraft_protocol::components::slots::Hand;
use minecraft_registries::block_placement_registry::{BlockPlacementParameters, BlockPlacementRegistry};
use minecraft_registries::block_property_registry::BlockPropertyRegistry;
use minecraft_registries::block_state_registry::BlockStateRegistry;
use minecraft_registries::item_click_registry::ItemClickRegistry;
use minecraft_vanilla::registries::Registries;
use sol_log_server::logger_mt::LoggerMt;
use sol_network_lib::constants;
use sol_voxel_lib::vector_alias::Coordinate;
use std::rc::Rc;
use std::sync::mpsc::{self, RecvError};
use std::time::Instant;

pub enum PlayerCommand {
    Stop,
    PlaceBlock(PlaceBlockCommand),
}

pub struct PlaceBlockCommand {
    pub hand: slots::Hand,
    pub location: Coordinate,
    pub face: minecraft_protocol::components::blocks::BlockFace,
    pub cursor_position_x: f32,
    pub cursor_position_y: f32,
    pub cursor_position_z: f32,
    pub inside_block: bool,
}

pub struct PlayerLoop {
    logger: LoggerMt,
    player_state: PlayerState,
    message_queue: mpsc::Receiver<PlayerCommand>,
    client_comm_channel: mpsc::Sender<ClientSendCommand>,
    world_event_channel: mpsc::Sender<GameCommand>,
    item_click_registry: Rc<ItemClickRegistry>,
    block_property_registry: Rc<BlockPropertyRegistry>,
    block_state_registry: Rc<BlockStateRegistry>,
    block_placement_registry: Rc<BlockPlacementRegistry>,
}

impl PlayerLoop {
    pub fn new(
        player_state: PlayerState,
        logger: LoggerMt,
        player_command_receiver: mpsc::Receiver<PlayerCommand>,
        client_comm_channel: mpsc::Sender<ClientSendCommand>,
        world_event_channel: mpsc::Sender<GameCommand>,
        registries: Registries,
    ) -> PlayerLoop {
        PlayerLoop {
            logger,
            player_state,
            message_queue: player_command_receiver,
            client_comm_channel,
            world_event_channel,
            item_click_registry: registries.item_click_registry,
            block_property_registry: registries.block_property_registry,
            block_state_registry: registries.block_state_registry,
            block_placement_registry: registries.block_placement_registry,
        }
    }

    pub fn run(&mut self) {
        let mut last_loop_end = Instant::now();

        loop {
            match self.message_queue.recv() {
                Ok(PlayerCommand::PlaceBlock(command)) => {
                    let event = self.create_voxel_place_event(command);

                    if let Some(event) = event {
                        let result = self
                            .world_event_channel
                            .send(GameCommand::ImmediateEvent(event));

                        if result.is_err() {
                            println!("Error sending voxel change event: {result:?}");
                        }
                    } else {
                        eprintln!("No item to place from PlaceBlock command");
                    }
                }
                Ok(PlayerCommand::Stop) | Err(RecvError) => {
                    // queue has closed: player should stop
                    return;
                }
            }

            let end = Instant::now();

            // number of milliseconds remaining in this loop
            let remaining_time = (end - last_loop_end).checked_sub(constants::TICK_PERIOD);

            if let Some(remaining_time) = remaining_time {
                std::thread::sleep(remaining_time);
            }

            last_loop_end = end;
        }
    }

    // NOTE: also removes the item from the player
    fn create_voxel_place_event(&mut self, command: PlaceBlockCommand) -> Option<Event> {
        let slot_idx = match command.hand {
            Hand::MainHand => self.player_state.selected_slot,
            Hand::OffHand => PLAYER_OFF_HAND_SLOT,
        };

        let stack = &mut self.player_state.slots[slot_idx];

        let placed_item = match stack.take_one() {
            ItemStack::Empty => return None,
            ItemStack::Simple(stack) => stack.item_type(),
            ItemStack::NbtItem(item) => item.item_type(),
        };

        let player_facing_normalized = self.player_state.look_direction;

        let how = BlockPlacementParameters {
            block_face: command.face,
            cursor_position_x: command.cursor_position_x,
            cursor_position_y: command.cursor_position_y,
            cursor_position_z: command.cursor_position_z,
            player_facing_normalized_x: player_facing_normalized.x,
            player_facing_normalized_y: player_facing_normalized.y,
            player_facing_normalized_z: player_facing_normalized.z,
            inside_block: false,
        };

        self.item_click_registry.get_item_click_event(
            &self.block_property_registry, &self.block_placement_registry, &self.block_state_registry,
            placed_item,
            target_block,
            how,
        );

        Some(Event::VoxelChange {
            coord: Coordinate::from(command.location),
            new_voxel: (),
        })
    }

    fn handle_event(&self, player_event: Event) {
        todo!()
    }
}

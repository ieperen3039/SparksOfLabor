use crate::entities::entity_manager::EntityManager;
use crate::game_event::{Event, ScheduledEvent};
use crate::minecraft_connection::client_connection::ClientSendCommand;
use crate::player_handler;
use crate::player_handler::PlayerHandler;
use crate::voxels::world::World;
use minecraft_vanilla::registries::Registries;
use sol_log_server::logger_mt::LoggerMt;
use sol_network_lib::constants;
use sol_network_lib::Tick;
use std::collections::BinaryHeap;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Instant;

pub struct GameLoop {
    logger: LoggerMt,
    current_tick: Tick,
    message_queue: mpsc::Receiver<GameCommand>,
    client_comm_channel: mpsc::Sender<ClientSendCommand>,
    event_queue: BinaryHeap<ScheduledEvent>,
    world: World,
    entities: EntityManager,
    player: PlayerHandler,
    registries: Registries,
}

pub enum GameCommand {
    Stop,
    ImmediateEvent(Event),
    FutureEvent(ScheduledEvent),
}

impl GameLoop {
    pub fn new(
        world: World,
        logger: LoggerMt,
        game_command_receiver: mpsc::Receiver<GameCommand>,
        client_comm_channel: mpsc::Sender<ClientSendCommand>,
        registries: Registries,
    ) -> GameLoop {
        GameLoop {
            logger,
            current_tick: 0,
            message_queue: game_command_receiver,
            client_comm_channel,
            world,
            entities: EntityManager::new(),
            player: PlayerHandler::new(todo!(), logger),
            event_queue: BinaryHeap::new(),
            registries,
        }
    }

    pub fn run(&mut self) {
        let mut last_loop_end = Instant::now();

        loop {
            self.current_tick += 1;

            // handle all incoming messages
            loop {
                match self.message_queue.try_recv() {
                    Ok(GameCommand::ImmediateEvent(event)) => self.schedule_for_this_tick(event),
                    Ok(GameCommand::FutureEvent(e)) => {
                        self.event_queue.push(e);
                    },
                    Ok(GameCommand::Stop) | Err(TryRecvError::Disconnected) => {
                        // queue has closed: game should stop
                        return;
                    },
                    Err(TryRecvError::Empty) => {
                        break;
                    },
                }
            }

            // now run every event that happened this tick
            while let Some(ScheduledEvent { tick, .. }) = self.event_queue.peek() {
                if *tick > self.current_tick {
                    break;
                }

                let game_event = self.event_queue.pop().unwrap().event;
                self.handle_event(game_event);
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

    fn schedule_for_this_tick(&mut self, event: Event) {
        self.event_queue.push(ScheduledEvent {
            tick: self.current_tick,
            event,
        })
    }

    fn handle_event(&mut self, game_event: Event) {
        let event = match game_event {
            Event::VoxelChange { .. } => { None },
            Event::VoxelUpdate { .. } => { None },
            Event::EntityUpdate { .. } => { None },
            Event::PlayerPlaceBlock(command) => {
                player_handler::handle_block_place_event(
                    command,
                    &mut self.player,
                    &self.world,
                    &self.registries,
                )
            },
        };

        if let Some(event) = event {
            self.schedule_for_this_tick(event);
        }
    }
}

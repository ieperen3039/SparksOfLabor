use crate::entities::entity_manager::EntityManager;
use crate::game_event::{Event, ScheduledEvent};
use crate::minecraft_connection::client_connection::ClientSendCommand;
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
            event_queue: BinaryHeap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut last_loop_end = Instant::now();

        loop {
            self.current_tick += 1;

            // handle all incoming messages
            loop {
                match self.message_queue.try_recv() {
                    Ok(GameCommand::ImmediateEvent(e)) => self.event_queue.push(ScheduledEvent {
                        tick: self.current_tick,
                        event: e,
                    }),
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

    fn handle_event(&self, game_event: Event) {
        todo!()
    }
}

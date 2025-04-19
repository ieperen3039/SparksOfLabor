use crate::entities::entity_manager::EntityManager;
use crate::game_event::{EventType, GameEvent};
use crate::voxels::world::World;
use sol_network_lib::constants;
use sol_network_lib::Tick;
use std::collections::BinaryHeap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Instant;

// TODO move to other file
pub enum GameCommand {
    GameStop,
    NewEvent(GameEvent),
}

pub struct GameState {
    is_stopping: bool,
    current_tick: Tick,
    message_queue_sender: Sender<GameCommand>,
    message_queue: Receiver<GameCommand>,
    event_queue: BinaryHeap<GameEvent>,
    world: World,
    entities: EntityManager,
}

impl GameState {
    pub fn build(world: World) -> GameState {
        let (sender, receiver) = std::sync::mpsc::channel();
        return GameState {
            is_stopping: false,
            current_tick: 0,
            message_queue_sender: sender,
            message_queue: receiver,
            world,
            entities: EntityManager::new(),
            event_queue: BinaryHeap::new(),
        };
    }

    pub fn run(&mut self) {
        let mut last_loop_end = Instant::now();

        loop {
            self.current_tick += 1;

            // handle all incoming messages
            loop {
                match self.message_queue.try_recv() {
                    Ok(GameCommand::GameStop) => return,
                    Ok(GameCommand::NewEvent(e)) => {
                        self.event_queue.push(e);
                    },
                    Err(TryRecvError::Disconnected) => {
                        // queue has closed: game should stop
                        return;
                    },
                    Err(TryRecvError::Empty) => {
                        break;
                    },
                }
            }

            // now run every event that happened this tick
            while let Some(GameEvent { tick, .. }) = self.event_queue.peek() {
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

    pub fn get_message_queue(&self) -> Sender<GameCommand> {
        self.message_queue_sender.clone()
    }

    fn handle_event(&self, game_event: EventType) {
        todo!()
    }
}

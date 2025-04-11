use std::cmp::Ordering;

use sol_voxel_lib::vector_alias::Position;

use crate::game_loop::Tick;

pub struct GameEvent {
    pub tick: Tick,
    pub event: EventType,
}

pub enum EventType {
    VoxelUpdate(Position),
    EntityUpdate { entity_id: u32 },
}

impl Ord for GameEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for GameEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for GameEvent {
    fn eq(&self, other: &Self) -> bool {
        self.tick == other.tick
    }
}

impl Eq for GameEvent {}

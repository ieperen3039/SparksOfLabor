use crate::player_events::PlayerPlaceBlockEvent;
use sol_network_lib::Tick;
use sol_voxel_lib::vector_alias::Coordinate;
use sol_voxel_lib::voxel::Voxel;
use std::cmp::Ordering;

pub struct ScheduledEvent {
    pub tick: Tick,
    pub event: Event,
}

pub enum Event {
    VoxelChange { coord: Coordinate, new_voxel: Voxel },
    VoxelUpdate { coord: Coordinate },
    EntityUpdate { entity_id: u32 },
    PlayerPlaceBlock(PlayerPlaceBlockEvent)
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.tick == other.tick
    }
}

impl Eq for ScheduledEvent {}

use crate::item_stack::ItemStack;
use sol_voxel_lib::vector_alias::{AxisDirection, Direction};
use std::array::from_fn;
use std::ops::RangeInclusive;

pub const PLAYER_OFF_HAND_SLOT: usize = 45;
pub const PLAYER_HOTBAR_SLOTS: RangeInclusive<usize> = 36..=44;
pub struct PlayerState {
    // 0 if the inventory window is open
    // We do not get a message when the inventory is opened,
    // so 0 may also mean no window is open at all.
    pub opened_window: u32,
    // see https://minecraft.wiki/w/File:Inventory-slots.png
    pub slots: [ItemStack; 45],
    pub selected_slot: usize,
    pub look_direction: Direction,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            opened_window: 0,
            slots: from_fn(|_| ItemStack::default()),
            selected_slot: *PLAYER_HOTBAR_SLOTS.start(),
            look_direction: AxisDirection::PosX.get_unit(),
        }
    }

    pub fn select_slot(&mut self, slot: usize) {
        assert!(PLAYER_HOTBAR_SLOTS.contains(&slot));
        self.selected_slot = slot;
    }
}

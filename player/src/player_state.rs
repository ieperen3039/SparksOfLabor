use crate::item_stack::{ItemStack, NbtItem};
use minecraft_protocol::data::items::Item;
use minecraft_protocol::nbt::NbtTag;
use minecraft_registries::item_click_registry::ItemChange;
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
    pub fn handle_item_change(&mut self, slot_idx: usize, change: ItemChange) {
        match change {
            ItemChange::Consumed => self.slots[slot_idx].remove_one(),
            ItemChange::Transformed { into } => {
                self.slots[slot_idx].remove_one();
                self.add_item(into, 1);
            }
            ItemChange::Damaged { quantity } => {
                match &mut self.slots[slot_idx] {
                    ItemStack::NbtItem(item) => {
                        self.apply_damage(item, quantity as i32);
                    },
                    _ => panic!("damage to non-nbt item"),
                }
            }
        }
    }

    pub fn add_item(&mut self, item: Item, count: usize) {
        let stack = &mut self.slots[self.selected_slot];
        if stack.is_empty() {
            *stack = ItemStack::new(item, count)
        } else {
            for stack in &mut self.slots {
                if stack.is_empty() {
                    *stack = ItemStack::new(item, count)
                }
            }
        }
    }

    /// negative `damage_quantity` results in damage being removed
    fn apply_damage(&self, item: &mut NbtItem, damage_quantity: i32) {
        match item.nbt_mut() {
            NbtTag::Compound(tags) => {
                match tags.get_mut("components") {
                    Some(NbtTag::Compound(tags)) => {
                        match tags.get_mut("damage") {
                            Some(NbtTag::Int(damage)) => {
                                *damage = i32::max(*damage + damage_quantity, 0)
                            }
                            _ => panic!("expected 'damage' tag")
                        }
                    }
                    _ => panic!("expected 'components' tag")
                }
            }
            _ => panic!("expected a compound tag at the top level")
        }
    }
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

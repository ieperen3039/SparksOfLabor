use minecraft_protocol::data::items::Item;
use minecraft_protocol::nbt::NbtTag;

pub enum ItemStack {
    Empty,
    Simple(SimpleItemStack),
    NbtItem(NbtItem),
}

pub struct SimpleItemStack {
    id: u16,
    count: u16,
}

// always one
pub struct NbtItem {
    id: u16,
    nbt: NbtTag,
}

impl ItemStack {
    pub fn new(item: Item, count: usize) -> Self {
        ItemStack::Simple(SimpleItemStack {
            id: item.id() as u16,
            count: count as u16,
        })
    }

    pub fn one_of(source: Item) -> Self {
        ItemStack::Simple(SimpleItemStack {
            id: source.id() as u16,
            count: 1,
        })
    }

    pub fn new_nbt(source: Item, nbt: NbtTag) -> Self {
        ItemStack::NbtItem(NbtItem {
            id: source.id() as u16,
            nbt,
        })
    }

    // splits one item off the stack, but returns an empty stack if no item was present
    pub fn take_one(&mut self) -> ItemStack {
        match self {
            ItemStack::Empty => ItemStack::Empty,
            ItemStack::Simple(s) => {
                if s.count == 1 {
                    std::mem::replace(self, ItemStack::Empty)
                } else {
                    ItemStack::Simple(s.take(1))
                }
            },
            ItemStack::NbtItem(_) => std::mem::replace(self, ItemStack::Empty),
        }
    }

    pub fn remove_one(&mut self) {
        self.take_one();
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, ItemStack::Empty)
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::Empty
    }
}

impl SimpleItemStack {
    pub fn take(&mut self, amount_taken: usize) -> Self {
        assert!(amount_taken <= self.count as usize);

        let count = amount_taken as u16;
        self.count -= count;
        SimpleItemStack { id: self.id, count }
    }

    pub fn try_take(&mut self, amount_taken: usize) -> Option<Self> {
        if amount_taken > self.count as usize {
            None
        } else {
            Some(self.take(amount_taken))
        }
    }

    pub fn add(&mut self, other: SimpleItemStack) {
        assert_eq!(self.id, other.id);
        self.count += other.count;
    }

    pub fn item_type(&self) -> Item {
        Item::from_id(self.id as u32)
    }

    pub fn count(&self) -> u16 {
        self.count
    }
}

impl NbtItem {
    pub fn item_type(&self) -> Item {
        Item::from_id(self.id as u32)
    }

    pub fn nbt(&self) -> &NbtTag {
        &self.nbt
    }

    pub fn nbt_mut(&mut self) -> &mut NbtTag {
        &mut self.nbt
    }
}

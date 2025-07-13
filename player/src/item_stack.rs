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
            }
            ItemStack::NbtItem(_) => std::mem::replace(self, ItemStack::Empty),
        }
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::Empty
    }
}

impl SimpleItemStack {
    pub fn new(source: minecraft_protocol::ids::items::Item, count: i32) -> Self {
        Self {
            id: source as u16,
            count: count as u16,
        }
    }

    pub fn one_of(source: minecraft_protocol::ids::items::Item) -> Self {
        Self {
            id: source as u16,
            count: 1,
        }
    }

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

    pub fn item_type(&self) -> minecraft_protocol::ids::items::Item {
        minecraft_protocol::ids::items::Item::from_id(self.id as u32).expect("Corrupt item id")
    }

    pub fn count(&self) -> u16 {
        self.count
    }
}

impl NbtItem {
    pub fn new(source: minecraft_protocol::ids::items::Item, nbt: NbtTag) -> NbtItem {
        NbtItem {
            id: source as u16,
            nbt,
        }
    }

    pub fn item_type(&self) -> minecraft_protocol::ids::items::Item {
        minecraft_protocol::ids::items::Item::from_id(self.id as u32).expect("Corrupt item id")
    }

    pub fn nbt(&self) -> &NbtTag {
        &self.nbt
    }
}

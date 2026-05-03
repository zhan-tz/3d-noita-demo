use crate::world::chunk::Material;

pub const HOTBAR_SIZE: usize = 9;
pub const BACKPACK_SIZE: usize = 27;
pub const TOTAL_SLOTS: usize = HOTBAR_SIZE + BACKPACK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    Pickaxe,
    Sword,
    Shield,
    Armor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellType {
    Fireball,
    Freeze,
    Lightning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsumableType {
    Bomb,
    HealPotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Block(Material),
    Tool(ToolType),
    Spell(SpellType),
    Consumable(ConsumableType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item_type: Option<ItemType>,
    pub count: u32,
}

impl ItemStack {
    pub fn empty() -> Self {
        ItemStack {
            item_type: None,
            count: 0,
        }
    }

    pub fn new(item_type: ItemType, count: u32) -> Self {
        ItemStack {
            item_type: Some(item_type),
            count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_type.is_none()
    }
}

impl Default for ItemStack {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    slots: [ItemStack; TOTAL_SLOTS],
    selected_slot: usize,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            slots: std::array::from_fn(|_| ItemStack::empty()),
            selected_slot: 0,
        }
    }

    /// Returns false if all 36 slots are occupied with different item types.
    pub fn add_item(&mut self, item: ItemType, count: u32) -> bool {
        for slot in &mut self.slots {
            if slot.item_type == Some(item) {
                slot.count += count;
                return true;
            }
        }
        for slot in &mut self.slots {
            if slot.is_empty() {
                *slot = ItemStack::new(item, count);
                return true;
            }
        }
        false
    }

    /// Returns false if slot is out of range or count exceeds what's available.
    pub fn remove_from_slot(&mut self, slot: usize, count: u32) -> bool {
        if slot >= TOTAL_SLOTS {
            return false;
        }
        let stack = &mut self.slots[slot];
        if stack.count < count {
            return false;
        }
        stack.count -= count;
        if stack.count == 0 {
            *stack = ItemStack::empty();
        }
        true
    }

    pub fn get_selected(&self) -> Option<&ItemStack> {
        let stack = &self.slots[self.selected_slot];
        if stack.is_empty() {
            None
        } else {
            Some(stack)
        }
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut ItemStack> {
        let slot = self.selected_slot;
        let stack = &mut self.slots[slot];
        if stack.is_empty() {
            None
        } else {
            Some(stack)
        }
    }

    pub fn select_slot(&mut self, slot: usize) {
        self.selected_slot = slot.min(HOTBAR_SIZE - 1);
    }

    pub fn selected_slot(&self) -> usize {
        self.selected_slot
    }

    pub fn is_hotbar(slot: usize) -> bool {
        slot < HOTBAR_SIZE
    }

    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        self.slots.get(index)
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        self.slots.get_mut(index)
    }

    /// Returns the Material of the placed block, or None if the slot is empty or not a Block.
    pub fn place_block(&mut self) -> Option<Material> {
        let slot = self.selected_slot;
        let stack = &self.slots[slot];
        match stack.item_type {
            Some(ItemType::Block(material)) => {
                if stack.count > 0 && self.remove_from_slot(slot, 1) {
                    Some(material)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_item_to_empty_slot() {
        let mut inv = Inventory::new();
        let added = inv.add_item(ItemType::Block(Material::Stone), 10);
        assert!(added);
        let slot = inv.get_slot(0).unwrap();
        assert_eq!(slot.item_type, Some(ItemType::Block(Material::Stone)));
        assert_eq!(slot.count, 10);
    }

    #[test]
    fn test_add_multiple_items() {
        let mut inv = Inventory::new();
        assert!(inv.add_item(ItemType::Block(Material::Stone), 10));
        assert!(inv.add_item(ItemType::Block(Material::Sand), 5));

        let slot0 = inv.get_slot(0).unwrap();
        assert_eq!(slot0.item_type, Some(ItemType::Block(Material::Stone)));
        assert_eq!(slot0.count, 10);

        let slot1 = inv.get_slot(1).unwrap();
        assert_eq!(slot1.item_type, Some(ItemType::Block(Material::Sand)));
        assert_eq!(slot1.count, 5);
    }

    #[test]
    fn test_place_block_consumes_item() {
        let mut inv = Inventory::new();
        inv.add_item(ItemType::Block(Material::Stone), 3);
        inv.select_slot(0);

        let placed = inv.place_block();
        assert_eq!(placed, Some(Material::Stone));

        let slot = inv.get_slot(0).unwrap();
        assert_eq!(slot.count, 2);
    }

    #[test]
    fn test_hotbar_selection() {
        let mut inv = Inventory::new();
        inv.add_item(ItemType::Block(Material::Stone), 10);
        inv.add_item(ItemType::Block(Material::Sand), 5);
        inv.add_item(ItemType::Tool(ToolType::Pickaxe), 1);

        // Default selection is slot 0
        assert_eq!(inv.selected_slot(), 0);
        let sel = inv.get_selected().unwrap();
        assert_eq!(sel.item_type, Some(ItemType::Block(Material::Stone)));

        // Select slot 2 (Pickaxe)
        inv.select_slot(2);
        assert_eq!(inv.selected_slot(), 2);
        let sel = inv.get_selected().unwrap();
        assert_eq!(sel.item_type, Some(ItemType::Tool(ToolType::Pickaxe)));

        // Out of range clamps to 8
        inv.select_slot(100);
        assert_eq!(inv.selected_slot(), 8);
    }

    #[test]
    fn test_remove_from_slot() {
        let mut inv = Inventory::new();
        inv.add_item(ItemType::Block(Material::Dirt), 5);

        assert!(inv.remove_from_slot(0, 3));
        assert_eq!(inv.get_slot(0).unwrap().count, 2);

        assert!(inv.remove_from_slot(0, 2));
        assert!(inv.get_slot(0).unwrap().is_empty());
    }

    #[test]
    fn test_is_hotbar() {
        assert!(Inventory::is_hotbar(0));
        assert!(Inventory::is_hotbar(8));
        assert!(!Inventory::is_hotbar(9));
        assert!(!Inventory::is_hotbar(35));
    }

    #[test]
    fn test_full_inventory() {
        let mut inv = Inventory::new();
        for slot in &mut inv.slots {
            *slot = ItemStack::new(ItemType::Block(Material::Stone), 1);
        }
        assert!(!inv.add_item(ItemType::Block(Material::Wood), 1));
        assert!(!inv.add_item(ItemType::Tool(ToolType::Pickaxe), 1));
    }
}

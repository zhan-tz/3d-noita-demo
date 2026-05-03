use crate::inventory::inventory::{ConsumableType, Inventory, ItemType, SpellType, ToolType};
use crate::world::chunk::Material;

pub struct CraftingRecipe {
    pub name: &'static str,
    pub ingredients: Vec<(ItemType, u32)>,
    pub result: ItemType,
    pub result_count: u32,
}

pub fn all_recipes() -> Vec<CraftingRecipe> {
    vec![
        CraftingRecipe {
            name: "Pickaxe",
            ingredients: vec![(ItemType::Block(Material::Wood), 3)],
            result: ItemType::Tool(ToolType::Pickaxe),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Sword",
            ingredients: vec![
                (ItemType::Block(Material::Stone), 3),
                (ItemType::Block(Material::Wood), 2),
            ],
            result: ItemType::Tool(ToolType::Sword),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Glass",
            ingredients: vec![(ItemType::Block(Material::Sand), 4)],
            result: ItemType::Block(Material::Stone),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Shield",
            ingredients: vec![
                (ItemType::Block(Material::Metal), 5),
                (ItemType::Block(Material::Wood), 2),
            ],
            result: ItemType::Tool(ToolType::Shield),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Fireball Scroll",
            ingredients: vec![
                (ItemType::Block(Material::Sand), 2),
                (ItemType::Block(Material::Wood), 1),
            ],
            result: ItemType::Spell(SpellType::Fireball),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Freeze Scroll",
            ingredients: vec![
                (ItemType::Block(Material::Ice), 2),
                (ItemType::Block(Material::Dirt), 1),
            ],
            result: ItemType::Spell(SpellType::Freeze),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Bomb",
            ingredients: vec![
                (ItemType::Block(Material::Sand), 3),
                (ItemType::Block(Material::Metal), 1),
            ],
            result: ItemType::Consumable(ConsumableType::Bomb),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Lightning Scroll",
            ingredients: vec![
                (ItemType::Block(Material::Metal), 3),
                (ItemType::Block(Material::Wood), 1),
            ],
            result: ItemType::Spell(SpellType::Lightning),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Heal Potion",
            ingredients: vec![(ItemType::Block(Material::Dirt), 3)],
            result: ItemType::Consumable(ConsumableType::HealPotion),
            result_count: 1,
        },
        CraftingRecipe {
            name: "Armor",
            ingredients: vec![(ItemType::Block(Material::Metal), 5)],
            result: ItemType::Tool(ToolType::Armor),
            result_count: 1,
        },
    ]
}

fn count_item_in_inventory(inventory: &Inventory, target: &ItemType) -> u32 {
    let mut total = 0u32;
    for i in 0..36 {
        if let Some(slot) = inventory.get_slot(i) {
            if slot.item_type.as_ref() == Some(target) {
                total += slot.count;
            }
        }
    }
    total
}

pub fn can_craft(recipe: &CraftingRecipe, inventory: &Inventory) -> bool {
    for (item_type, needed) in &recipe.ingredients {
        let have = count_item_in_inventory(inventory, item_type);
        if have < *needed {
            return false;
        }
    }
    true
}

pub fn craft(recipe: &CraftingRecipe, inventory: &mut Inventory) -> bool {
    if !can_craft(recipe, inventory) {
        return false;
    }
    // Consume ingredients
    for (item_type, needed) in &recipe.ingredients {
        let mut remaining = *needed;
        for i in 0..36 {
            if remaining == 0 {
                break;
            }
            if let Some(slot) = inventory.get_slot(i) {
                if slot.item_type.as_ref() == Some(item_type) {
                    let take = remaining.min(slot.count);
                    inventory.remove_from_slot(i, take);
                    remaining -= take;
                }
            }
        }
    }
    // Add result
    inventory.add_item(recipe.result, recipe.result_count);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_craft_with_materials() {
        let mut inv = Inventory::new();
        inv.add_item(ItemType::Block(Material::Wood), 3);
        let recipes = all_recipes();
        assert!(can_craft(&recipes[0], &inv)); // Pickaxe
    }

    #[test]
    fn test_cannot_craft_without() {
        let inv = Inventory::new();
        let recipes = all_recipes();
        for recipe in &recipes {
            assert!(!can_craft(recipe, &inv));
        }
    }

    #[test]
    fn test_crafting_consumes_materials() {
        let mut inv = Inventory::new();
        inv.add_item(ItemType::Block(Material::Wood), 5);
        let recipes = all_recipes();
        assert!(craft(&recipes[0], &mut inv)); // Pickaxe from 3 Wood
                                               // Should have 2 Wood left and 1 Pickaxe
        assert_eq!(
            count_item_in_inventory(&inv, &ItemType::Block(Material::Wood)),
            2
        );
        assert_eq!(
            count_item_in_inventory(&inv, &ItemType::Tool(ToolType::Pickaxe)),
            1
        );
    }
}

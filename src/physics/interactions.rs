use crate::world::chunk::Material;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionResult {
    None,
    Replace(Material),
    SwapBoth(Material, Material),
}

pub struct InteractionTable {
    rules: HashMap<(Material, Material), InteractionResult>,
}

impl InteractionTable {
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        rules.insert(
            (Material::Water, Material::Lava),
            InteractionResult::SwapBoth(Material::Stone, Material::Air),
        );
        rules.insert(
            (Material::Lava, Material::Water),
            InteractionResult::SwapBoth(Material::Air, Material::Stone),
        );

        rules.insert(
            (Material::Water, Material::Fire),
            InteractionResult::Replace(Material::Air),
        );
        rules.insert(
            (Material::Fire, Material::Water),
            InteractionResult::Replace(Material::Air),
        );

        rules.insert(
            (Material::Fire, Material::Wood),
            InteractionResult::Replace(Material::Fire),
        );
        rules.insert(
            (Material::Wood, Material::Fire),
            InteractionResult::Replace(Material::Fire),
        );

        rules.insert(
            (Material::Fire, Material::Dirt),
            InteractionResult::Replace(Material::Air),
        );

        rules.insert(
            (Material::Lava, Material::Wood),
            InteractionResult::Replace(Material::Fire),
        );
        rules.insert(
            (Material::Wood, Material::Lava),
            InteractionResult::Replace(Material::Fire),
        );

        rules.insert(
            (Material::Lava, Material::Ice),
            InteractionResult::SwapBoth(Material::Stone, Material::Water),
        );
        rules.insert(
            (Material::Ice, Material::Lava),
            InteractionResult::SwapBoth(Material::Water, Material::Stone),
        );

        rules.insert(
            (Material::Fire, Material::Ice),
            InteractionResult::Replace(Material::Water),
        );
        rules.insert(
            (Material::Ice, Material::Fire),
            InteractionResult::Replace(Material::Water),
        );

        rules.insert(
            (Material::Lava, Material::Sand),
            InteractionResult::Replace(Material::Stone),
        );
        rules.insert(
            (Material::Sand, Material::Lava),
            InteractionResult::Replace(Material::Stone),
        );

        InteractionTable { rules }
    }

    pub fn check(&self, mat_a: Material, mat_b: Material) -> InteractionResult {
        self.rules
            .get(&(mat_a, mat_b))
            .copied()
            .unwrap_or(InteractionResult::None)
    }
}

impl Default for InteractionTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_interactions(blocks: &[Material; 6], center: Material) -> Vec<(usize, Material)> {
    let table = InteractionTable::new();
    let mut changes = Vec::new();

    for (i, &neighbor) in blocks.iter().enumerate() {
        if neighbor == Material::Air || neighbor == center {
            continue;
        }

        match table.check(center, neighbor) {
            InteractionResult::None => {}
            InteractionResult::Replace(new_mat) => {
                changes.push((i, new_mat));
            }
            InteractionResult::SwapBoth(_, new_neighbor) => {
                changes.push((i, new_neighbor));
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_lava_interaction() {
        let table = InteractionTable::new();
        let result = table.check(Material::Water, Material::Lava);
        assert_eq!(
            result,
            InteractionResult::SwapBoth(Material::Stone, Material::Air)
        );
    }

    #[test]
    fn test_fire_wood_spread() {
        let table = InteractionTable::new();
        let result = table.check(Material::Fire, Material::Wood);
        assert_eq!(result, InteractionResult::Replace(Material::Fire));
    }

    #[test]
    fn test_lava_ice() {
        let table = InteractionTable::new();
        let result = table.check(Material::Lava, Material::Ice);
        assert_eq!(
            result,
            InteractionResult::SwapBoth(Material::Stone, Material::Water)
        );
    }

    #[test]
    fn test_fire_extinguished_by_water() {
        let table = InteractionTable::new();
        let result = table.check(Material::Water, Material::Fire);
        assert_eq!(result, InteractionResult::Replace(Material::Air));
    }

    #[test]
    fn test_no_interaction_stone_stone() {
        let table = InteractionTable::new();
        let result = table.check(Material::Stone, Material::Stone);
        assert_eq!(result, InteractionResult::None);
    }

    #[test]
    fn test_no_interaction_metal_fire() {
        let table = InteractionTable::new();
        let result = table.check(Material::Metal, Material::Fire);
        assert_eq!(result, InteractionResult::None);
    }

    #[test]
    fn test_lava_sand_glass() {
        let table = InteractionTable::new();
        let result = table.check(Material::Lava, Material::Sand);
        assert_eq!(result, InteractionResult::Replace(Material::Stone));
    }

    #[test]
    fn test_fire_ice_water() {
        let table = InteractionTable::new();
        let result = table.check(Material::Fire, Material::Ice);
        assert_eq!(result, InteractionResult::Replace(Material::Water));
    }
}

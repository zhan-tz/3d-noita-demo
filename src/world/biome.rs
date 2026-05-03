use crate::world::chunk::Material;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Surface,
    ShallowCaves,
    DeepCaves,
    LavaZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterCategory {
    Weak,
    Medium,
    Strong,
    Boss,
}

pub struct BiomeConfig {
    pub biome: Biome,
    pub stone_weight: f32,
    pub dirt_weight: f32,
    pub sand_weight: f32,
    pub water_weight: f32,
    pub lava_weight: f32,
    pub metal_weight: f32,
    pub ice_weight: f32,
    pub monster_types: Vec<MonsterCategory>,
}

pub fn get_biome(y: i32) -> Biome {
    if y >= 192 {
        Biome::Surface
    } else if y >= 128 {
        Biome::ShallowCaves
    } else if y >= 64 {
        Biome::DeepCaves
    } else {
        Biome::LavaZone
    }
}

pub fn get_biome_config(biome: Biome) -> BiomeConfig {
    match biome {
        Biome::Surface => BiomeConfig {
            biome: Biome::Surface,
            stone_weight: 0.30,
            dirt_weight: 0.35,
            sand_weight: 0.10,
            water_weight: 0.05,
            lava_weight: 0.0,
            metal_weight: 0.0,
            ice_weight: 0.05,
            monster_types: vec![MonsterCategory::Weak],
        },
        Biome::ShallowCaves => BiomeConfig {
            biome: Biome::ShallowCaves,
            stone_weight: 0.55,
            dirt_weight: 0.15,
            sand_weight: 0.05,
            water_weight: 0.10,
            lava_weight: 0.0,
            metal_weight: 0.05,
            ice_weight: 0.0,
            monster_types: vec![MonsterCategory::Weak, MonsterCategory::Medium],
        },
        Biome::DeepCaves => BiomeConfig {
            biome: Biome::DeepCaves,
            stone_weight: 0.60,
            dirt_weight: 0.0,
            sand_weight: 0.0,
            water_weight: 0.05,
            lava_weight: 0.15,
            metal_weight: 0.10,
            ice_weight: 0.0,
            monster_types: vec![MonsterCategory::Medium, MonsterCategory::Strong],
        },
        Biome::LavaZone => BiomeConfig {
            biome: Biome::LavaZone,
            stone_weight: 0.50,
            dirt_weight: 0.0,
            sand_weight: 0.0,
            water_weight: 0.0,
            lava_weight: 0.30,
            metal_weight: 0.15,
            ice_weight: 0.0,
            monster_types: vec![MonsterCategory::Strong, MonsterCategory::Boss],
        },
    }
}

pub fn get_material_for_biome(biome: Biome, noise_value: f32) -> Material {
    let config = get_biome_config(biome);
    let n = noise_value.clamp(0.0, 1.0);

    let mut cumulative = 0.0;

    cumulative += config.dirt_weight;
    if n < cumulative {
        return Material::Dirt;
    }

    cumulative += config.stone_weight;
    if n < cumulative {
        return Material::Stone;
    }

    cumulative += config.sand_weight;
    if n < cumulative {
        return Material::Sand;
    }

    cumulative += config.water_weight;
    if n < cumulative {
        return Material::Water;
    }

    cumulative += config.ice_weight;
    if n < cumulative {
        return Material::Ice;
    }

    cumulative += config.lava_weight;
    if n < cumulative {
        return Material::Lava;
    }

    cumulative += config.metal_weight;
    if n < cumulative {
        return Material::Metal;
    }

    Material::Air
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_from_depth() {
        assert_eq!(get_biome(200), Biome::Surface);
        assert_eq!(get_biome(192), Biome::Surface);
        assert_eq!(get_biome(191), Biome::ShallowCaves);
        assert_eq!(get_biome(128), Biome::ShallowCaves);
        assert_eq!(get_biome(127), Biome::DeepCaves);
        assert_eq!(get_biome(64), Biome::DeepCaves);
        assert_eq!(get_biome(63), Biome::LavaZone);
        assert_eq!(get_biome(0), Biome::LavaZone);
    }

    #[test]
    fn test_surface_has_dirt() {
        let mut dirt_count = 0;
        let samples = 1000;
        for i in 0..samples {
            let noise = i as f32 / samples as f32;
            if get_material_for_biome(Biome::Surface, noise) == Material::Dirt {
                dirt_count += 1;
            }
        }
        // Surface dirt_weight = 0.35, so roughly 350 out of 1000
        assert!(
            dirt_count > 300 && dirt_count < 400,
            "Expected ~350 dirt samples for Surface, got {}",
            dirt_count
        );
    }

    #[test]
    fn test_lava_zone_has_lava() {
        let mut lava_count = 0;
        let samples = 1000;
        for i in 0..samples {
            let noise = i as f32 / samples as f32;
            if get_material_for_biome(Biome::LavaZone, noise) == Material::Lava {
                lava_count += 1;
            }
        }
        // LavaZone lava_weight = 0.30, so roughly 300 out of 1000
        assert!(
            lava_count > 250 && lava_count < 350,
            "Expected ~300 lava samples for LavaZone, got {}",
            lava_count
        );
    }

    #[test]
    fn test_material_selection_is_deterministic() {
        let noise = 0.42;
        let m1 = get_material_for_biome(Biome::Surface, noise);
        let m2 = get_material_for_biome(Biome::Surface, noise);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_deep_caves_has_no_dirt() {
        let config = get_biome_config(Biome::DeepCaves);
        assert_eq!(config.dirt_weight, 0.0);
    }

    #[test]
    fn test_noise_zero_returns_first_material() {
        // noise=0.0 should hit the first material in the cumulative order
        let surface = get_material_for_biome(Biome::Surface, 0.0);
        assert_eq!(surface, Material::Dirt);
    }

    #[test]
    fn test_noise_one_returns_air_or_last() {
        // noise=1.0 should be past all weights → Air
        let lava_zone = get_material_for_biome(Biome::LavaZone, 1.0);
        assert_eq!(lava_zone, Material::Air);
    }
}

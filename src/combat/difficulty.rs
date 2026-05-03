use crate::world::biome::{get_biome, Biome};

pub struct DifficultyConfig {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub speed_multiplier: f32,
    pub spawn_rate: f32,
    pub environmental_damage: f32,
}

pub fn get_difficulty(depth: i32) -> DifficultyConfig {
    match get_biome(depth) {
        Biome::Surface => DifficultyConfig {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            speed_multiplier: 1.0,
            spawn_rate: 2.0,
            environmental_damage: 0.0,
        },
        Biome::ShallowCaves => DifficultyConfig {
            health_multiplier: 1.5,
            damage_multiplier: 1.5,
            speed_multiplier: 1.1,
            spawn_rate: 3.0,
            environmental_damage: 0.0,
        },
        Biome::DeepCaves => DifficultyConfig {
            health_multiplier: 2.0,
            damage_multiplier: 2.0,
            speed_multiplier: 1.2,
            spawn_rate: 4.0,
            environmental_damage: 0.5,
        },
        Biome::LavaZone => DifficultyConfig {
            health_multiplier: 3.0,
            damage_multiplier: 3.0,
            speed_multiplier: 1.3,
            spawn_rate: 5.0,
            environmental_damage: 2.0,
        },
    }
}

pub fn apply_difficulty(
    base_hp: f32,
    base_damage: f32,
    base_speed: f32,
    depth: i32,
) -> (f32, f32, f32) {
    let config = get_difficulty(depth);
    (
        base_hp * config.health_multiplier,
        base_damage * config.damage_multiplier,
        base_speed * config.speed_multiplier,
    )
}

pub fn get_depth_label(depth: i32) -> &'static str {
    match get_biome(depth) {
        Biome::Surface => "Surface",
        Biome::ShallowCaves => "Shallow Caves",
        Biome::DeepCaves => "Deep Caves",
        Biome::LavaZone => "Lava Zone",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_scales_with_depth() {
        let surface = get_difficulty(200);
        let deep = get_difficulty(50);
        assert!(deep.health_multiplier > surface.health_multiplier);
        assert!(deep.damage_multiplier > surface.damage_multiplier);
        assert!(deep.speed_multiplier > surface.speed_multiplier);
    }

    #[test]
    fn test_surface_difficulty() {
        let d = get_difficulty(250);
        assert_eq!(d.health_multiplier, 1.0);
        assert_eq!(d.damage_multiplier, 1.0);
        assert_eq!(d.environmental_damage, 0.0);
    }

    #[test]
    fn test_lava_zone_damage() {
        let d = get_difficulty(30);
        assert_eq!(d.health_multiplier, 3.0);
        assert!(d.environmental_damage > 0.0);
        assert_eq!(d.environmental_damage, 2.0);
    }
}

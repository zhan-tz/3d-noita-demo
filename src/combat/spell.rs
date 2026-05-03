use bevy::prelude::{IVec3, Vec3};

use crate::physics::explosion::ExplosionResult;
use crate::physics::material::MaterialProperties;
use crate::world::chunk::{Block, ChunkMap, Material, CHUNK_SIZE};
/// Single-sourced spell system with 8 spell types.
/// This module provides pure functions for spell behavior suitable for testing.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellType {
    Fireball,  // Projectile: ignites blocks, damages monsters
    Dig,       // Ray: destroys blocks along line
    Explosion, // Instant: triggers explosion at target
    Freeze,    // Area: Water->Ice, slows monsters
    Lightning, // Line: damages all entities along path
    Heal,      // Self: restores player HP
    Teleport,  // Self: moves player to target position
    Shield,    // Self: temporary invincibility
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpellData {
    pub spell_type: SpellType,
    pub cooldown: f32,       // seconds
    pub damage: f32,         // for offensive spells
    pub range: f32,          // max distance
    pub radius: f32,         // for area spells
    pub last_cast_time: f32, // for cooldown tracking
    pub duration: f32,       // for duration-based spells (e.g., Shield)
}

impl SpellData {
    pub fn new(spell_type: SpellType) -> Self {
        match spell_type {
            SpellType::Fireball => Self {
                spell_type,
                cooldown: 1.0,
                damage: 15.0,
                range: 50.0,
                radius: 2.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Dig => Self {
                spell_type,
                cooldown: 0.3,
                damage: 0.0,
                range: 10.0,
                radius: 1.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Explosion => Self {
                spell_type,
                cooldown: 5.0,
                damage: 50.0,
                range: 30.0,
                radius: 5.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Freeze => Self {
                spell_type,
                cooldown: 3.0,
                damage: 5.0,
                range: 20.0,
                radius: 4.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Lightning => Self {
                spell_type,
                cooldown: 2.0,
                damage: 25.0,
                range: 40.0,
                radius: 0.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Heal => Self {
                spell_type,
                cooldown: 10.0,
                damage: -30.0, // negative damage means heal
                range: 0.0,
                radius: 0.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Teleport => Self {
                spell_type,
                cooldown: 8.0,
                damage: 0.0,
                range: 50.0,
                radius: 0.0,
                last_cast_time: -1.0,
                duration: 0.0,
            },
            SpellType::Shield => Self {
                spell_type,
                cooldown: 15.0,
                damage: 0.0,
                range: 0.0,
                radius: 0.0,
                last_cast_time: -1.0,
                duration: 5.0,
            },
        }
    }

    pub fn create(spell_type: SpellType) -> Self {
        SpellData::new(spell_type)
    }
}

/// Simple 3D projectile used for spell casting visuals/logic.
pub struct Projectile {
    pub pos: Vec3,
    pub vel: Vec3,
    pub spell_type: SpellType,
    pub lifetime: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpellEffectResult {
    None,
    FireIgnited(IVec3),
    Explosion(ExplosionResult),
    DigDestroyed(IVec3),
    FreezeApplied(IVec3),
    Healed(f32),
    Teleported(IVec3),
    ShieldApplied,
}

/// Check if a spell can be cast given its last cast time and cooldown.
pub fn can_cast(spell: &SpellData, current_time: f32) -> bool {
    if spell.last_cast_time < 0.0 {
        return true;
    }
    current_time - spell.last_cast_time >= spell.cooldown
}

/// Cast a spell if possible. Updates last_cast_time and returns true on success.
pub fn cast(spell: &mut SpellData, current_time: f32) -> bool {
    if can_cast(spell, current_time) {
        spell.last_cast_time = current_time;
        true
    } else {
        false
    }
}

/// Create a simple spell effect on the world. This is intentionally pure and testable.
pub fn apply_spell_effect(
    spell_type: SpellType,
    target_pos: IVec3,
    chunk_map: &mut ChunkMap,
) -> SpellEffectResult {
    match spell_type {
        SpellType::Fireball => {
            // Ignite if the target block is flammable
            let current = chunk_map.get_block(target_pos);
            let props = MaterialProperties::get(current);
            if props.flammable && current != Material::Air {
                chunk_map.set_block(target_pos, Block::new(Material::Fire));
                SpellEffectResult::FireIgnited(target_pos)
            } else {
                SpellEffectResult::None
            }
        }
        SpellType::Explosion => {
            // Radius taken from a default explosion spell; caller may override by using
            // the Sphere radius from SpellData if needed. Here we use 5 as a reasonable default,
            // but on purpose we delegate to the existing explosion module to compute results.
            let center = target_pos;
            // Use the existing explosion logic with radius 5
            let res = crate::physics::explosion::create_explosion(chunk_map, center, 5);
            SpellEffectResult::Explosion(res)
        }
        SpellType::Dig => {
            // Simple ray along -X direction for a short range based on  range ~ 10
            let mut destroyed_pos = target_pos;
            for i in 0..10 {
                let pos = IVec3 {
                    x: (target_pos.x as i32 - i) as i32,
                    y: target_pos.y,
                    z: target_pos.z,
                };
                // Bounds check through set_block (will no-op if out of chunk)
                chunk_map.set_block(pos, Block::air());
                destroyed_pos = pos;
            }
            SpellEffectResult::DigDestroyed(destroyed_pos)
        }
        SpellType::Freeze => {
            // Turn water in a radius into ice
            let radius = 4u32; // coarse radius
            let r = radius as i32;
            for x in (target_pos.x - r)..=(target_pos.x + r) {
                for y in (target_pos.y - r)..=(target_pos.y + r) {
                    for z in (target_pos.z - r)..=(target_pos.z + r) {
                        let pos = IVec3 { x, y, z };
                        let current = chunk_map.get_block(pos);
                        if current == Material::Water {
                            chunk_map.set_block(pos, Block::new(Material::Ice));
                        }
                    }
                }
            }
            SpellEffectResult::FreezeApplied(target_pos)
        }
        SpellType::Lightning => {
            // Simplified: no world mutation; return None for now
            SpellEffectResult::None
        }
        SpellType::Heal => SpellEffectResult::Healed(30.0),
        SpellType::Teleport => SpellEffectResult::Teleported(target_pos),
        SpellType::Shield => SpellEffectResult::ShieldApplied,
    }
}

/// Update a projectile's position and lifetime. Returns true if still alive.
pub fn update_projectile(proj: &mut Projectile, dt: f32) -> bool {
    proj.pos += proj.vel * dt;
    if proj.lifetime > 0.0 {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunk::{Block, ChunkMap, Material};
    use bevy::prelude::IVec3;

    fn simple_map_with_one_block(world_pos: IVec3, mat: Material) -> ChunkMap {
        let mut map = ChunkMap::default();
        map.set_block(world_pos, Block::new(mat));
        map
    }

    #[test]
    fn test_spell_cooldown() {
        let mut spell = SpellData::create(SpellType::Fireball);

        // Initially can cast
        assert!(can_cast(&spell, 0.0));
        // Cast at time 0.0
        assert!(cast(&mut spell, 0.0));
        // Immediately trying to cast again should fail
        assert!(!can_cast(&spell, 0.0));
        // After cooldown of 1.0s, should be able to cast
        assert!(can_cast(&spell, 1.0));
        assert!(cast(&mut spell, 1.0));
    }

    #[test]
    fn test_fireball_ignites_wood() {
        let mut map = ChunkMap::default();
        let pos = IVec3 { x: 0, y: 0, z: 0 };
        map.insert_chunk(crate::world::chunk::Chunk::new(IVec3::ZERO));
        map.set_block(pos, Block::new(Material::Wood));

        let res = apply_spell_effect(SpellType::Fireball, pos, &mut map);
        // Should ignite wood into Fire
        assert!(matches!(res, SpellEffectResult::FireIgnited(p) if p == pos));
        assert_eq!(map.get_block(pos), Material::Fire);
    }

    #[test]
    fn test_explosion_spell() {
        let mut map = ChunkMap::default();
        let center = IVec3 { x: 8, y: 8, z: 8 };
        map.insert_chunk(crate::world::chunk::Chunk::new(IVec3::ZERO));
        // Fill around center with Stone so explosion has something to affect
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    let w = IVec3 {
                        x: center.x + dx,
                        y: center.y + dy,
                        z: center.z + dz,
                    };
                    map.set_block(w, Block::new(Material::Stone));
                }
            }
        }
        if let SpellEffectResult::Explosion(res) =
            apply_spell_effect(SpellType::Explosion, center, &mut map)
        {
            // Explosion should report some affected chunks
            assert!(!res.affected_chunks.is_empty());
            // Center should be likely turned into Air due to destruction
            assert_eq!(map.get_block(center), Material::Air);
        } else {
            panic!("Explosion did not produce ExplosionResult");
        }
    }
}

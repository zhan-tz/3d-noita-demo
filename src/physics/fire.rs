use crate::physics::material::MaterialProperties;
use crate::world::chunk::{Block, Chunk, Material, CHUNK_SIZE};

#[derive(Debug, Clone)]
pub struct FireDelta {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub new_block: Block,
}

const UP_SPREAD_PROB: u32 = 70;
const DIAG_UP_SPREAD_PROB: u32 = 40;
const HORIZ_SPREAD_PROB: u32 = 20;
const FIRE_INITIAL_LIFETIME: f32 = 10.0;
const HEAT_AMOUNT: f32 = 5.0;
const MAX_TEMPERATURE: f32 = 100.0;

const NEIGHBORS: [(i32, i32, i32); 6] = [
    (0, 1, 0),
    (0, -1, 0),
    (-1, 0, 0),
    (1, 0, 0),
    (0, 0, -1),
    (0, 0, 1),
];

const DIAG_UP: [(i32, i32, i32); 4] = [(-1, 1, 0), (1, 1, 0), (0, 1, -1), (0, 1, 1)];

const HORIZONTAL: [(i32, i32, i32); 4] = [(-1, 0, 0), (1, 0, 0), (0, 0, -1), (0, 0, 1)];

/// Deterministic hash that incorporates position and the fire's current lifetime.
/// Because lifetime decreases each step, the hash varies over time and spread
/// eventually succeeds at every position.
fn deterministic_hash(x: usize, y: usize, z: usize, lifetime: f32) -> u32 {
    let t = lifetime.to_bits() as u64;
    let hash = (x as u64).wrapping_mul(73856093)
        ^ (y as u64).wrapping_mul(19349663)
        ^ (z as u64).wrapping_mul(83492791)
        ^ t;
    (hash % 100) as u32
}

#[inline]
fn in_bounds(x: i32, y: i32, z: i32) -> bool {
    x >= 0
        && x < CHUNK_SIZE as i32
        && y >= 0
        && y < CHUNK_SIZE as i32
        && z >= 0
        && z < CHUNK_SIZE as i32
}

fn try_spread(
    chunk: &Chunk,
    src_x: usize,
    src_y: usize,
    src_z: usize,
    directions: &[(i32, i32, i32)],
    probability: u32,
    lifetime: f32,
    changes: &mut Vec<(usize, usize, usize, Block)>,
) {
    for &(dx, dy, dz) in directions {
        let nx = src_x as i32 + dx;
        let ny = src_y as i32 + dy;
        let nz = src_z as i32 + dz;
        if !in_bounds(nx, ny, nz) {
            continue;
        }
        let nxi = nx as usize;
        let nyi = ny as usize;
        let nzi = nz as usize;
        let neighbor_mat = chunk.get_material(nxi, nyi, nzi);
        if MaterialProperties::get(neighbor_mat).flammable {
            let hash = deterministic_hash(nxi, nyi, nzi, lifetime);
            if hash < probability {
                changes.push((
                    nxi,
                    nyi,
                    nzi,
                    Block::with_temperature(Material::Fire, FIRE_INITIAL_LIFETIME),
                ));
            }
        }
    }
}

/// Enhanced fire/gas simulation step.
///
/// Lifetime is stored in `Block.temperature` (starts at ~10, decrements per step).
/// When lifetime ≤ 0 the fire becomes Air. Adjacent Water extinguishes fire.
/// Fire spreads to flammable neighbors with directional probability:
/// up 70%, diagonal-up 40%, horizontal 20%.
/// Non-fire, non-liquid neighbors are heated each step.
pub fn simulate_fire_step(chunk: &mut Chunk) -> Vec<FireDelta> {
    let mut deltas = Vec::new();
    let mut changes: Vec<(usize, usize, usize, Block)> = Vec::new();
    let mut heated: Vec<(usize, usize, usize, f32)> = Vec::new();

    let mut fire_positions = Vec::new();
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get_material(x, y, z) == Material::Fire {
                    fire_positions.push((x, y, z));
                }
            }
        }
    }

    for (x, y, z) in fire_positions {
        let block = chunk.get_block(x, y, z);
        if block.material != Material::Fire {
            continue;
        }

        let mut extinguished = false;
        for &(dx, dy, dz) in &NEIGHBORS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;
            if in_bounds(nx, ny, nz)
                && chunk.get_material(nx as usize, ny as usize, nz as usize) == Material::Water
            {
                changes.push((x, y, z, Block::air()));
                extinguished = true;
                break;
            }
        }
        if extinguished {
            continue;
        }

        let new_lifetime = block.temperature - 1.0;

        if new_lifetime <= 0.0 {
            changes.push((x, y, z, Block::air()));
            continue;
        }

        changes.push((
            x,
            y,
            z,
            Block::with_temperature(Material::Fire, new_lifetime),
        ));

        try_spread(
            chunk,
            x,
            y,
            z,
            &[(0, 1, 0)],
            UP_SPREAD_PROB,
            new_lifetime,
            &mut changes,
        );
        try_spread(
            chunk,
            x,
            y,
            z,
            &DIAG_UP,
            DIAG_UP_SPREAD_PROB,
            new_lifetime,
            &mut changes,
        );
        try_spread(
            chunk,
            x,
            y,
            z,
            &HORIZONTAL,
            HORIZ_SPREAD_PROB,
            new_lifetime,
            &mut changes,
        );

        for &(dx, dy, dz) in &NEIGHBORS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;
            if !in_bounds(nx, ny, nz) {
                continue;
            }
            let nxi = nx as usize;
            let nyi = ny as usize;
            let nzi = nz as usize;
            let neighbor = chunk.get_block(nxi, nyi, nzi);
            if neighbor.material == Material::Fire || neighbor.material.is_liquid() {
                continue;
            }
            let heat = if dy == 1 {
                HEAT_AMOUNT + 3.0
            } else {
                HEAT_AMOUNT
            };
            let new_temp = (neighbor.temperature + heat).min(MAX_TEMPERATURE);
            heated.push((nxi, nyi, nzi, new_temp));
        }
    }

    for (x, y, z, new_block) in changes {
        chunk.set_block(x, y, z, new_block);
        deltas.push(FireDelta { x, y, z, new_block });
    }

    for (x, y, z, new_temp) in heated {
        let block = chunk.get_block(x, y, z);
        if block.material != Material::Fire {
            let heated_block = Block {
                material: block.material,
                temperature: new_temp,
            };
            chunk.set_block(x, y, z, heated_block);
            deltas.push(FireDelta {
                x,
                y,
                z,
                new_block: heated_block,
            });
        }
    }

    deltas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunk::Chunk;
    use bevy::prelude::IVec3;

    #[test]
    fn test_fire_spreads_to_wood() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_material(x, 0, z, Material::Stone);
            }
        }
        for y in 1..=5 {
            chunk.set_block(8, y, 8, Block::new(Material::Wood));
        }
        chunk.set_block(
            8,
            1,
            8,
            Block::with_temperature(Material::Fire, FIRE_INITIAL_LIFETIME),
        );
        chunk.dirty = false;

        for _ in 0..100 {
            let _ = simulate_fire_step(&mut chunk);
        }

        for y in 1..=5 {
            let mat = chunk.get_material(8, y, 8);
            assert_ne!(
                mat,
                Material::Wood,
                "Wood at y={} should have been consumed by fire",
                y
            );
        }
    }

    #[test]
    fn test_fire_lifetime_expires() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_block(8, 8, 8, Block::with_temperature(Material::Fire, 3.0));
        chunk.dirty = false;

        for _ in 0..3 {
            let _ = simulate_fire_step(&mut chunk);
        }

        assert_eq!(
            chunk.get_material(8, 8, 8),
            Material::Air,
            "Fire should have expired after lifetime ran out"
        );
    }

    #[test]
    fn test_fire_extinguished_by_water() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_block(8, 8, 8, Block::with_temperature(Material::Fire, 10.0));
        chunk.set_block(9, 8, 8, Block::new(Material::Water));
        chunk.dirty = false;

        let _ = simulate_fire_step(&mut chunk);

        assert_eq!(
            chunk.get_material(8, 8, 8),
            Material::Air,
            "Fire should be extinguished by adjacent water"
        );
    }

    #[test]
    fn test_fire_heats_air() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_block(8, 8, 8, Block::with_temperature(Material::Fire, 10.0));
        chunk.dirty = false;

        let _ = simulate_fire_step(&mut chunk);

        let above = chunk.get_block(8, 9, 8);
        assert_eq!(above.material, Material::Air);
        assert!(
            above.temperature > 20.0,
            "Air above fire should be heated above default 20.0, got {}",
            above.temperature
        );
    }
}

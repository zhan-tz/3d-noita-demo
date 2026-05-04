use crate::physics::material::{MaterialBehavior, MaterialProperties};
use crate::world::chunk::{Block, Chunk, Material, CHUNK_SIZE};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct BlockDelta {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub new_block: Block,
}

pub struct SimulationResult {
    pub deltas: HashMap<IVec3, Vec<BlockDelta>>,
}

pub fn simulate_step(chunks: &mut [&mut Chunk]) -> SimulationResult {
    let mut result = SimulationResult {
        deltas: HashMap::new(),
    };

    for chunk in chunks.iter_mut() {
        let chunk_pos = chunk.position;
        let mut deltas = Vec::new();
        let mut processed: HashSet<(usize, usize, usize)> = HashSet::new();

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let x_range: Box<dyn Iterator<Item = usize>> = if z % 2 == 0 {
                    Box::new(0..CHUNK_SIZE)
                } else {
                    Box::new((0..CHUNK_SIZE).rev())
                };

                for x in x_range {
                    if processed.contains(&(x, y, z)) {
                        continue;
                    }

                    let mat = chunk.get_material(x, y, z);
                    if mat == Material::Air {
                        continue;
                    }

                    let props = MaterialProperties::get(mat);
                    if !props.is_mobile() {
                        continue;
                    }

                    if props.lifetime.is_some() {
                        let hash = (x * 7 + y * 13 + z * 23) % 100;
                        if hash < 10 {
                            chunk.set_material(x, y, z, Material::Air);
                            deltas.push(BlockDelta {
                                x,
                                y,
                                z,
                                new_block: Block::air(),
                            });
                            continue;
                        }
                    }

                    let moved = match props.behavior {
                        MaterialBehavior::FallingSolid => {
                            try_move_falling_solid(chunk, x, y, z, &props, &mut processed)
                        }
                        MaterialBehavior::Liquid => {
                            try_move_liquid(chunk, x, y, z, &props, &mut processed)
                        }
                        MaterialBehavior::Gas => {
                            try_move_gas(chunk, x, y, z, &props, &mut processed)
                        }
                        MaterialBehavior::Static => false,
                    };

                    if moved {
                        deltas.push(BlockDelta {
                            x,
                            y,
                            z,
                            new_block: chunk.get_block(x, y, z),
                        });
                    }
                }
            }
        }

        if !deltas.is_empty() {
            result.deltas.insert(chunk_pos, deltas);
        }
    }

    result
}

fn try_move_falling_solid(
    chunk: &mut Chunk,
    x: usize,
    y: usize,
    z: usize,
    props: &MaterialProperties,
    processed: &mut HashSet<(usize, usize, usize)>,
) -> bool {
    let mat = chunk.get_material(x, y, z);

    if y > 0 {
        let below = chunk.get_material(x, y - 1, z);
        if below == Material::Air {
            move_block(chunk, x, y, z, x, y - 1, z, mat, processed);
            return true;
        }
        if below.is_liquid() {
            let below_props = MaterialProperties::get(below);
            if props.density > below_props.density {
                swap_blocks(chunk, x, y, z, x, y - 1, z, mat, below, processed);
                return true;
            }
        }
    }

    if y > 0 {
        let diags: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for &(dx, dz) in &diags {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if nx >= 0 && nx < CHUNK_SIZE as i32 && nz >= 0 && nz < CHUNK_SIZE as i32 {
                let nxi = nx as usize;
                let nzi = nz as usize;
                let diag_below = chunk.get_material(nxi, y - 1, nzi);
                let diag_level = chunk.get_material(nxi, y, nzi);
                if (diag_level == Material::Air || diag_level.is_liquid())
                    && can_displace(mat, diag_below, props)
                {
                    let displaced = if diag_below.is_liquid() {
                        diag_below
                    } else {
                        Material::Air
                    };
                    if displaced != Material::Air {
                        swap_blocks(chunk, x, y, z, nxi, y - 1, nzi, mat, displaced, processed);
                    } else {
                        move_block(chunk, x, y, z, nxi, y - 1, nzi, mat, processed);
                    }
                    return true;
                }
            }
        }
    }

    false
}

fn try_move_liquid(
    chunk: &mut Chunk,
    x: usize,
    y: usize,
    z: usize,
    props: &MaterialProperties,
    processed: &mut HashSet<(usize, usize, usize)>,
) -> bool {
    let mat = chunk.get_material(x, y, z);

    if y > 0 {
        let below = chunk.get_material(x, y - 1, z);
        if below == Material::Air {
            move_block(chunk, x, y, z, x, y - 1, z, mat, processed);
            return true;
        }
        if below.is_liquid() && below != mat {
            let below_props = MaterialProperties::get(below);
            if props.density > below_props.density {
                swap_blocks(chunk, x, y, z, x, y - 1, z, mat, below, processed);
                return true;
            }
        }
    }

    if y > 0 {
        let diags: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for &(dx, dz) in &diags {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if nx >= 0 && nx < CHUNK_SIZE as i32 && nz >= 0 && nz < CHUNK_SIZE as i32 {
                let nxi = nx as usize;
                let nzi = nz as usize;
                if chunk.get_material(nxi, y - 1, nzi) == Material::Air
                    && chunk.get_material(nxi, y, nzi) == Material::Air
                {
                    move_block(chunk, x, y, z, nxi, y - 1, nzi, mat, processed);
                    return true;
                }
            }
        }
    }

    let horiz: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for &(dx, dz) in &horiz {
        let nx = x as i32 + dx;
        let nz = z as i32 + dz;
        if nx >= 0 && nx < CHUNK_SIZE as i32 && nz >= 0 && nz < CHUNK_SIZE as i32 {
            if chunk.get_material(nx as usize, y, nz as usize) == Material::Air {
                move_block(chunk, x, y, z, nx as usize, y, nz as usize, mat, processed);
                return true;
            }
        }
    }

    false
}

fn try_move_gas(
    chunk: &mut Chunk,
    x: usize,
    y: usize,
    z: usize,
    _props: &MaterialProperties,
    processed: &mut HashSet<(usize, usize, usize)>,
) -> bool {
    let mat = chunk.get_material(x, y, z);

    if y < CHUNK_SIZE - 1 {
        let above = chunk.get_material(x, y + 1, z);
        if above == Material::Air {
            move_block(chunk, x, y, z, x, y + 1, z, mat, processed);
            return true;
        }
    }

    let horiz: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for &(dx, dz) in &horiz {
        let nx = x as i32 + dx;
        let nz = z as i32 + dz;
        if nx >= 0 && nx < CHUNK_SIZE as i32 && nz >= 0 && nz < CHUNK_SIZE as i32 {
            if chunk.get_material(nx as usize, y, nz as usize) == Material::Air {
                move_block(chunk, x, y, z, nx as usize, y, nz as usize, mat, processed);
                return true;
            }
        }
    }

    false
}

fn can_displace(mover: Material, target: Material, mover_props: &MaterialProperties) -> bool {
    if target == Material::Air {
        return true;
    }
    if target == mover {
        return false;
    }
    let target_props = MaterialProperties::get(target);
    if target_props.behavior == MaterialBehavior::Liquid
        && mover_props.density > target_props.density
    {
        return true;
    }
    false
}

#[inline]
fn move_block(
    chunk: &mut Chunk,
    sx: usize,
    sy: usize,
    sz: usize,
    dx: usize,
    dy: usize,
    dz: usize,
    mat: Material,
    processed: &mut HashSet<(usize, usize, usize)>,
) {
    chunk.set_material(dx, dy, dz, mat);
    chunk.set_material(sx, sy, sz, Material::Air);
    processed.insert((dx, dy, dz));
}

#[inline]
fn swap_blocks(
    chunk: &mut Chunk,
    sx: usize,
    sy: usize,
    sz: usize,
    dx: usize,
    dy: usize,
    dz: usize,
    mat_a: Material,
    mat_b: Material,
    processed: &mut HashSet<(usize, usize, usize)>,
) {
    chunk.set_material(dx, dy, dz, mat_a);
    chunk.set_material(sx, sy, sz, mat_b);
    processed.insert((dx, dy, dz));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sand_falls() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_material(8, 8, 8, Material::Sand);
        chunk.dirty = false;

        let mut refs = [&mut chunk];
        let result = simulate_step(&mut refs);
        assert!(
            result.deltas.contains_key(&IVec3::ZERO)
                || chunk.get_material(8, 7, 8) == Material::Sand
        );
    }

    #[test]
    fn test_sand_on_floor_stays() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_material(x, 1, z, Material::Stone);
            }
        }
        chunk.set_material(8, 2, 8, Material::Sand);

        let mut refs = [&mut chunk];
        let _ = simulate_step(&mut refs);
        assert_eq!(chunk.get_material(8, 2, 8), Material::Sand);
    }

    #[test]
    fn test_water_flows() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_material(x, 0, z, Material::Stone);
            }
        }
        chunk.set_material(8, 1, 8, Material::Water);

        let mut refs = [&mut chunk];
        let result = simulate_step(&mut refs);

        assert!(result.deltas.contains_key(&IVec3::ZERO));
        assert_ne!(chunk.get_material(8, 1, 8), Material::Water);

        let mut found = false;
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if chunk.get_material(x, 1, z) == Material::Water {
                    found = true;
                }
            }
        }
        assert!(found, "Water should be conserved");
    }

    #[test]
    fn test_sand_sinks_through_water() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_material(x, 0, z, Material::Stone);
            }
        }
        chunk.set_material(8, 1, 8, Material::Water);
        chunk.set_material(8, 2, 8, Material::Sand);

        for _ in 0..5 {
            let mut refs = [&mut chunk];
            let _ = simulate_step(&mut refs);
        }

        assert_eq!(
            chunk.get_material(8, 1, 8),
            Material::Sand,
            "Sand should sink to y=1"
        );
    }
}

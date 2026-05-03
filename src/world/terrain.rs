use crate::world::chunk::{Chunk, Material, CHUNK_SIZE};
use bevy::prelude::*;

pub const SURFACE_Y: i32 = 128;
pub const TERRAIN_HEIGHT_SCALE: f32 = 30.0;
pub const CAVE_THRESHOLD: f64 = 0.3;

pub struct SimpleNoise {
    seed: u32,
}

impl SimpleNoise {
    pub fn new(seed: u32) -> Self {
        SimpleNoise { seed }
    }

    pub fn noise_2d(&self, x: f32, z: f32) -> f64 {
        let mut value = 0.0_f64;
        let mut amplitude = 1.0_f64;
        let mut frequency = 1.0_f64;
        let mut max_value = 0.0_f64;

        for _octave in 0..5 {
            let nx = (x as f64) * frequency * 0.01;
            let nz = (z as f64) * frequency * 0.01;
            value += self.hash_noise_2d(nx, nz) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value / max_value
    }

    pub fn noise_3d(&self, x: f32, y: f32, z: f32) -> f64 {
        let mut value = 0.0_f64;
        let mut amplitude = 1.0_f64;
        let mut frequency = 1.0_f64;
        let mut max_value = 0.0_f64;

        for _octave in 0..3 {
            let nx = (x as f64) * frequency * 0.05;
            let ny = (y as f64) * frequency * 0.05;
            let nz = (z as f64) * frequency * 0.05;
            value += self.hash_noise_3d(nx, ny, nz) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value / max_value
    }

    fn hash(&self, x: i64, y: i64, z: i64) -> f64 {
        let mut h = self.seed as i64;
        h = h.wrapping_mul(374761393);
        h = h.wrapping_add(x);
        h = h.wrapping_mul(668265263);
        h = h.wrapping_add(y);
        h = h.wrapping_mul(1274126177);
        h = h.wrapping_add(z);
        h = h.wrapping_mul(1911520717);
        ((h & 0xFFFF) as f64 / 0xFFFF as f64) * 2.0 - 1.0
    }

    fn hash_noise_2d(&self, x: f64, z: f64) -> f64 {
        let ix = x.floor() as i64;
        let iz = z.floor() as i64;
        let fx = x - ix as f64;
        let fz = z - iz as f64;

        let v00 = self.hash(ix, 0, iz);
        let v10 = self.hash(ix + 1, 0, iz);
        let v01 = self.hash(ix, 0, iz + 1);
        let v11 = self.hash(ix + 1, 0, iz + 1);

        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sz = fz * fz * (3.0 - 2.0 * fz);

        let v0 = v00 + sx * (v10 - v00);
        let v1 = v01 + sx * (v11 - v01);
        v0 + sz * (v1 - v0)
    }

    fn hash_noise_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let ix = x.floor() as i64;
        let iy = y.floor() as i64;
        let iz = z.floor() as i64;
        let fx = x - ix as f64;
        let fy = y - iy as f64;
        let fz = z - iz as f64;

        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sy = fy * fy * (3.0 - 2.0 * fy);
        let sz = fz * fz * (3.0 - 2.0 * fz);

        let v000 = self.hash(ix, iy, iz);
        let v100 = self.hash(ix + 1, iy, iz);
        let v010 = self.hash(ix, iy + 1, iz);
        let v110 = self.hash(ix + 1, iy + 1, iz);
        let v001 = self.hash(ix, iy, iz + 1);
        let v101 = self.hash(ix + 1, iy, iz + 1);
        let v011 = self.hash(ix, iy + 1, iz + 1);
        let v111 = self.hash(ix + 1, iy + 1, iz + 1);

        let v00 = v000 + sx * (v100 - v000);
        let v10 = v010 + sx * (v110 - v010);
        let v01 = v001 + sx * (v101 - v001);
        let v11 = v011 + sx * (v111 - v011);

        let v0 = v00 + sy * (v10 - v00);
        let v1 = v01 + sy * (v11 - v01);

        v0 + sz * (v1 - v0)
    }
}

pub struct TerrainGenerator {
    seed: u32,
    height_noise: SimpleNoise,
    cave_noise: SimpleNoise,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let height_noise = SimpleNoise::new(seed);
        let cave_noise = SimpleNoise::new(seed.wrapping_add(1));
        Self {
            seed,
            height_noise,
            cave_noise,
        }
    }

    pub fn generate_chunk(&mut self, chunk_pos: IVec3) -> Chunk {
        let mut chunk = Chunk::new(chunk_pos);

        let world_base_x = chunk_pos.x * CHUNK_SIZE as i32;
        let world_base_y = chunk_pos.y * CHUNK_SIZE as i32;
        let world_base_z = chunk_pos.z * CHUNK_SIZE as i32;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = (world_base_x + x as i32) as f32;
                let world_z = (world_base_z + z as i32) as f32;

                let noise_val = self.height_noise.noise_2d(world_x, world_z);
                let surface_y = (SURFACE_Y as f32 + noise_val as f32 * TERRAIN_HEIGHT_SCALE) as i32;

                for y in 0..CHUNK_SIZE {
                    let world_y = world_base_y + y as i32;

                    let material = if world_y > surface_y {
                        Material::Air
                    } else if world_y == surface_y {
                        Material::Dirt
                    } else if world_y > surface_y - 4 {
                        Material::Dirt
                    } else {
                        Material::Stone
                    };

                    let final_material = if material != Material::Air {
                        let world_yf = world_y as f32;
                        let cave_val = self.cave_noise.noise_3d(world_x, world_yf, world_z);
                        if cave_val > CAVE_THRESHOLD {
                            Material::Air
                        } else {
                            material
                        }
                    } else {
                        material
                    };

                    if final_material != Material::Air {
                        chunk.set_material(x, y, z, final_material);
                    }
                }
            }
        }

        chunk.dirty = true;
        chunk
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut gen1 = TerrainGenerator::new(42);
        let mut gen2 = TerrainGenerator::new(42);
        let chunk1 = gen1.generate_chunk(IVec3::new(0, 8, 0));
        let chunk2 = gen2.generate_chunk(IVec3::new(0, 8, 0));

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(
                        chunk1.get_material(x, y, z),
                        chunk2.get_material(x, y, z),
                        "Mismatch at ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_surface_has_air() {
        let mut gen = TerrainGenerator::new(42);
        let chunk = gen.generate_chunk(IVec3::new(0, 12, 0));
        let air_count = count_material(&chunk, Material::Air);
        assert!(
            air_count > CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE / 2,
            "High chunk should have significant air, got {} air blocks",
            air_count
        );
    }

    #[test]
    fn test_underground_has_stone() {
        let mut gen = TerrainGenerator::new(42);
        let chunk = gen.generate_chunk(IVec3::new(0, 0, 0));
        let stone_count = count_material(&chunk, Material::Stone);
        assert!(
            stone_count > CHUNK_SIZE * CHUNK_SIZE,
            "Deep chunk should have stone, got {} stone blocks",
            stone_count
        );
    }

    #[test]
    fn test_caves_exist() {
        let mut gen = TerrainGenerator::new(42);
        let mut found_cave = false;
        for cx in -2..=2 {
            for cz in -2..=2 {
                let chunk = gen.generate_chunk(IVec3::new(cx, 6, cz));
                for x in 2..CHUNK_SIZE - 2 {
                    for y in 2..CHUNK_SIZE - 2 {
                        for z in 2..CHUNK_SIZE - 2 {
                            if chunk.get_material(x, y, z) == Material::Air {
                                let neighbors_air = [
                                    chunk.get_material(x - 1, y, z) == Material::Air,
                                    chunk.get_material(x + 1, y, z) == Material::Air,
                                    chunk.get_material(x, y - 1, z) == Material::Air,
                                    chunk.get_material(x, y + 1, z) == Material::Air,
                                ];
                                if neighbors_air.iter().filter(|&&a| a).count() < 3 {
                                    found_cave = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(
            found_cave,
            "Should find at least one cave in generated terrain"
        );
    }

    #[test]
    fn test_different_seeds_differ() {
        let mut gen1 = TerrainGenerator::new(42);
        let mut gen2 = TerrainGenerator::new(99);
        let chunk1 = gen1.generate_chunk(IVec3::new(0, 8, 0));
        let chunk2 = gen2.generate_chunk(IVec3::new(0, 8, 0));

        let mut differences = 0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if chunk1.get_material(x, y, z) != chunk2.get_material(x, y, z) {
                        differences += 1;
                    }
                }
            }
        }
        assert!(
            differences > 0,
            "Different seeds should produce different terrain"
        );
    }

    fn count_material(chunk: &Chunk, material: Material) -> usize {
        let mut count = 0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if chunk.get_material(x, y, z) == material {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

use crate::world::chunk::{Chunk, Material, CHUNK_SIZE};

pub struct GhostCellData {
    pub pos_x: [Material; CHUNK_SIZE * CHUNK_SIZE],
    pub neg_x: [Material; CHUNK_SIZE * CHUNK_SIZE],
    pub pos_y: [Material; CHUNK_SIZE * CHUNK_SIZE],
    pub neg_y: [Material; CHUNK_SIZE * CHUNK_SIZE],
    pub pos_z: [Material; CHUNK_SIZE * CHUNK_SIZE],
    pub neg_z: [Material; CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for GhostCellData {
    fn default() -> Self {
        GhostCellData {
            pos_x: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
            neg_x: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
            pos_y: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
            neg_y: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
            pos_z: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
            neg_z: [Material::Air; CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl GhostCellData {
    pub fn get_neighbor_material(&self, local_x: i32, local_y: i32, local_z: i32) -> Material {
        if local_x == CHUNK_SIZE as i32 {
            let idx = local_y as usize * CHUNK_SIZE + local_z as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.pos_x[idx];
            }
        }
        if local_x == -1 {
            let idx = local_y as usize * CHUNK_SIZE + local_z as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.neg_x[idx];
            }
        }
        if local_y == CHUNK_SIZE as i32 {
            let idx = local_x as usize * CHUNK_SIZE + local_z as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.pos_y[idx];
            }
        }
        if local_y == -1 {
            let idx = local_x as usize * CHUNK_SIZE + local_z as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.neg_y[idx];
            }
        }
        if local_z == CHUNK_SIZE as i32 {
            let idx = local_x as usize * CHUNK_SIZE + local_y as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.pos_z[idx];
            }
        }
        if local_z == -1 {
            let idx = local_x as usize * CHUNK_SIZE + local_y as usize;
            if idx < CHUNK_SIZE * CHUNK_SIZE {
                return self.neg_z[idx];
            }
        }
        Material::Air
    }
}

pub fn sync_ghost_cells(_chunk: &Chunk, neighbors: &[Option<&Chunk>; 6]) -> GhostCellData {
    let mut ghost = GhostCellData::default();

    if let Some(nbr) = neighbors[0] {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                ghost.pos_x[y * CHUNK_SIZE + z] = nbr.get_material(0, y, z);
            }
        }
    }

    if let Some(nbr) = neighbors[1] {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                ghost.neg_x[y * CHUNK_SIZE + z] = nbr.get_material(CHUNK_SIZE - 1, y, z);
            }
        }
    }

    if let Some(nbr) = neighbors[2] {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                ghost.pos_y[x * CHUNK_SIZE + z] = nbr.get_material(x, 0, z);
            }
        }
    }

    if let Some(nbr) = neighbors[3] {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                ghost.neg_y[x * CHUNK_SIZE + z] = nbr.get_material(x, CHUNK_SIZE - 1, z);
            }
        }
    }

    if let Some(nbr) = neighbors[4] {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                ghost.pos_z[x * CHUNK_SIZE + y] = nbr.get_material(x, y, 0);
            }
        }
    }

    if let Some(nbr) = neighbors[5] {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                ghost.neg_z[x * CHUNK_SIZE + y] = nbr.get_material(x, y, CHUNK_SIZE - 1);
            }
        }
    }

    ghost
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec3;

    #[test]
    fn test_ghost_cell_sync() {
        let mut chunk_a = Chunk::new(IVec3::new(0, 0, 0));
        let mut chunk_b = Chunk::new(IVec3::new(1, 0, 0));

        chunk_a.set_material(15, 5, 5, Material::Sand);
        chunk_b.set_material(0, 5, 5, Material::Water);

        let neighbors: [Option<&Chunk>; 6] = [Some(&chunk_b), None, None, None, None, None];
        let ghost = sync_ghost_cells(&chunk_a, &neighbors);

        assert_eq!(ghost.get_neighbor_material(16, 5, 5), Material::Water);
    }

    #[test]
    fn test_no_neighbor_returns_air() {
        let chunk = Chunk::new(IVec3::ZERO);
        let neighbors: [Option<&Chunk>; 6] = [None; 6];
        let ghost = sync_ghost_cells(&chunk, &neighbors);

        assert_eq!(ghost.get_neighbor_material(16, 5, 5), Material::Air);
        assert_eq!(ghost.get_neighbor_material(-1, 5, 5), Material::Air);
    }
}

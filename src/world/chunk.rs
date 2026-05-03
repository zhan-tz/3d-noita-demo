use bevy::prelude::*;
use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_F: f32 = 16.0;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Sand = 3,
    Water = 4,
    Lava = 5,
    Wood = 6,
    Metal = 7,
    Ice = 8,
    Fire = 9,
}

impl Material {
    pub fn is_solid(self) -> bool {
        matches!(
            self,
            Material::Stone
                | Material::Dirt
                | Material::Sand
                | Material::Wood
                | Material::Metal
                | Material::Ice
        )
    }

    pub fn is_liquid(self) -> bool {
        matches!(self, Material::Water | Material::Lava)
    }

    pub fn is_gas(self) -> bool {
        matches!(self, Material::Fire)
    }

    pub fn is_air(self) -> bool {
        self == Material::Air
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub material: Material,
    pub temperature: f32,
}

impl Block {
    pub fn air() -> Self {
        Block {
            material: Material::Air,
            temperature: 20.0,
        }
    }

    pub fn new(material: Material) -> Self {
        Block {
            material,
            temperature: 20.0,
        }
    }

    pub fn with_temperature(material: Material, temperature: f32) -> Self {
        Block {
            material,
            temperature,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::air()
    }
}

#[inline]
fn chunk_index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

pub struct Chunk {
    blocks: Box<[Block; CHUNK_VOLUME]>,
    pub position: IVec3,
    pub dirty: bool,
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        let blocks = Box::new([Block::air(); CHUNK_VOLUME]);
        Chunk {
            blocks,
            position,
            dirty: false,
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[chunk_index(x, y, z)]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.blocks[chunk_index(x, y, z)] = block;
            self.dirty = true;
        }
    }

    pub fn get_material(&self, x: usize, y: usize, z: usize) -> Material {
        self.blocks[chunk_index(x, y, z)].material
    }

    pub fn set_material(&mut self, x: usize, y: usize, z: usize, material: Material) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.blocks[chunk_index(x, y, z)].material = material;
            self.dirty = true;
        }
    }

    pub fn world_to_local(world_pos: IVec3) -> (IVec3, IVec3) {
        let chunk_pos = IVec3::new(
            world_pos.x.div_euclid(CHUNK_SIZE as i32),
            world_pos.y.div_euclid(CHUNK_SIZE as i32),
            world_pos.z.div_euclid(CHUNK_SIZE as i32),
        );
        let local_pos = IVec3::new(
            world_pos.x.rem_euclid(CHUNK_SIZE as i32),
            world_pos.y.rem_euclid(CHUNK_SIZE as i32),
            world_pos.z.rem_euclid(CHUNK_SIZE as i32),
        );
        (chunk_pos, local_pos)
    }
}

pub struct ChunkMap {
    chunks: HashMap<IVec3, Chunk>,
}

impl ChunkMap {
    pub fn new() -> Self {
        ChunkMap {
            chunks: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, pos: IVec3) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn remove_chunk(&mut self, pos: IVec3) -> Option<Chunk> {
        self.chunks.remove(&pos)
    }

    pub fn get_block(&self, world_pos: IVec3) -> Material {
        let (chunk_pos, local_pos) = Chunk::world_to_local(world_pos);
        match self.chunks.get(&chunk_pos) {
            Some(chunk) => chunk.get_material(
                local_pos.x as usize,
                local_pos.y as usize,
                local_pos.z as usize,
            ),
            None => Material::Air,
        }
    }

    pub fn set_block(&mut self, world_pos: IVec3, block: Block) {
        let (chunk_pos, local_pos) = Chunk::world_to_local(world_pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_block(
                local_pos.x as usize,
                local_pos.y as usize,
                local_pos.z as usize,
                block,
            );
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.chunks.values()
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_chunk() {
        let chunk = Chunk::new(IVec3::ZERO);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(chunk.get_material(x, y, z), Material::Air);
                }
            }
        }
    }

    #[test]
    fn test_set_get_block() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_material(0, 0, 0, Material::Sand);
        assert_eq!(chunk.get_material(0, 0, 0), Material::Sand);
    }

    #[test]
    fn test_set_get_corner() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_material(15, 15, 15, Material::Water);
        assert_eq!(chunk.get_material(15, 15, 15), Material::Water);
    }

    #[test]
    fn test_chunk_map_insert_get() {
        let mut map = ChunkMap::new();
        let chunk = Chunk::new(IVec3::new(0, 0, 0));
        map.insert_chunk(chunk);
        assert!(map.get_chunk(IVec3::ZERO).is_some());
        assert!(map.get_chunk(IVec3::new(1, 0, 0)).is_none());
    }

    #[test]
    fn test_chunk_map_world_coords() {
        let mut map = ChunkMap::new();
        let mut chunk = Chunk::new(IVec3::new(1, 0, 0));
        chunk.set_material(1, 0, 0, Material::Stone);
        map.insert_chunk(chunk);
        assert_eq!(map.get_block(IVec3::new(17, 0, 0)), Material::Stone);
    }

    #[test]
    fn test_dirty_flag() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        assert!(!chunk.dirty);
        chunk.set_material(0, 0, 0, Material::Sand);
        assert!(chunk.dirty);
    }

    #[test]
    fn test_negative_world_coords() {
        let (chunk_pos, local_pos) = Chunk::world_to_local(IVec3::new(-1, 0, 0));
        assert_eq!(chunk_pos, IVec3::new(-1, 0, 0));
        assert_eq!(local_pos, IVec3::new(15, 0, 0));
    }
}

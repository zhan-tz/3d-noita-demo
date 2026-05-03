use crate::world::chunk::{Chunk, Material, CHUNK_SIZE};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

pub fn material_color(mat: Material) -> Color {
    match mat {
        Material::Air => Color::srgba(0.0, 0.0, 0.0, 0.0),
        Material::Stone => Color::srgb(0.5, 0.5, 0.5),
        Material::Dirt => Color::srgb(0.6, 0.4, 0.2),
        Material::Sand => Color::srgb(0.9, 0.85, 0.5),
        Material::Water => Color::srgb(0.2, 0.4, 0.8),
        Material::Lava => Color::srgb(0.9, 0.3, 0.0),
        Material::Wood => Color::srgb(0.55, 0.35, 0.15),
        Material::Metal => Color::srgb(0.7, 0.7, 0.75),
        Material::Ice => Color::srgb(0.7, 0.9, 1.0),
        Material::Fire => Color::srgb(1.0, 0.6, 0.0),
    }
}

const FACE_DIRS: [(i32, i32, i32); 6] = [
    (1, 0, 0),
    (-1, 0, 0),
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, 1),
    (0, 0, -1),
];

fn face_vertices(x: f32, y: f32, z: f32, dx: i32, dy: i32, dz: i32) -> [[f32; 3]; 4] {
    match (dx, dy, dz) {
        (1, 0, 0) => [
            [x + 1.0, y, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y, z + 1.0],
        ],
        (-1, 0, 0) => [
            [x, y, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x, y + 1.0, z],
            [x, y, z],
        ],
        (0, 1, 0) => [
            [x, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z],
            [x, y + 1.0, z],
        ],
        (0, -1, 0) => [
            [x, y, z],
            [x + 1.0, y, z],
            [x + 1.0, y, z + 1.0],
            [x, y, z + 1.0],
        ],
        (0, 0, 1) => [
            [x + 1.0, y, z + 1.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x, y + 1.0, z + 1.0],
            [x, y, z + 1.0],
        ],
        (0, 0, -1) => [
            [x, y, z],
            [x, y + 1.0, z],
            [x + 1.0, y + 1.0, z],
            [x + 1.0, y, z],
        ],
        _ => [[0.0; 3]; 4],
    }
}

fn face_normal(dx: i32, dy: i32, dz: i32) -> [f32; 3] {
    [dx as f32, dy as f32, dz as f32]
}

pub fn generate_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let material = chunk.get_material(x, y, z);
                if material == Material::Air {
                    continue;
                }

                let color = material_color(material);
                let srgba = color.to_srgba();
                let color_array: [f32; 4] = [srgba.red, srgba.green, srgba.blue, srgba.alpha];

                for &(dx, dy, dz) in &FACE_DIRS {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;

                    let neighbor_is_transparent = if nx < 0
                        || nx >= CHUNK_SIZE as i32
                        || ny < 0
                        || ny >= CHUNK_SIZE as i32
                        || nz < 0
                        || nz >= CHUNK_SIZE as i32
                    {
                        true
                    } else {
                        let neighbor = chunk.get_material(nx as usize, ny as usize, nz as usize);
                        neighbor == Material::Air || neighbor == Material::Water
                    };

                    if neighbor_is_transparent {
                        let vert_count = positions.len() as u32;
                        let verts = face_vertices(x as f32, y as f32, z as f32, dx, dy, dz);
                        for v in &verts {
                            positions.push(*v);
                            normals.push(face_normal(dx, dy, dz));
                            colors.push(color_array);
                        }

                        indices.extend_from_slice(&[
                            vert_count,
                            vert_count + 1,
                            vert_count + 2,
                            vert_count,
                            vert_count + 2,
                            vert_count + 3,
                        ]);
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunk::Block;

    #[test]
    fn test_all_air_chunk_empty_mesh() {
        let chunk = Chunk::new(IVec3::ZERO);
        let mesh = generate_chunk_mesh(&chunk);
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 0, "All-air chunk should have empty mesh");
    }

    #[test]
    fn test_single_block_has_6_faces() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_block(8, 8, 8, Block::new(Material::Stone));

        let mesh = generate_chunk_mesh(&chunk);
        let indices = mesh.indices().unwrap();
        assert_eq!(
            indices.len(),
            36,
            "Single block in air should have 6 faces (36 indices)"
        );
    }

    #[test]
    fn test_adjacent_blocks_share_face() {
        let mut chunk = Chunk::new(IVec3::ZERO);
        chunk.set_material(8, 8, 8, Material::Stone);
        chunk.set_material(9, 8, 8, Material::Stone);

        let mesh = generate_chunk_mesh(&chunk);
        let indices = mesh.indices().unwrap();
        assert_eq!(
            indices.len(),
            60,
            "Two adjacent blocks should have 10 faces (60 indices)"
        );
    }
}

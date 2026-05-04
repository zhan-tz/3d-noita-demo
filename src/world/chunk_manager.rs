use bevy::prelude::*;
use std::collections::HashSet;

use crate::rendering::chunk_renderer::generate_chunk_mesh;
use crate::world::chunk::{ChunkMap, CHUNK_SIZE, CHUNK_SIZE_F};
use crate::world::terrain::TerrainGenerator;

pub const RENDER_DISTANCE: i32 = 4;

#[derive(Component)]
pub struct ChunkEntity {
    pub position: IVec3,
}

#[derive(Resource)]
pub struct ChunkWorldState {
    pub chunks: ChunkMap,
    pub terrain_gen: TerrainGenerator,
}

#[derive(Component)]
pub struct NeedsMesh;

pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkWorldState {
            chunks: ChunkMap::new(),
            terrain_gen: TerrainGenerator::new(42),
        })
        .add_systems(
            Update,
            (
                load_chunks_around_player,
                spawn_chunk_meshes,
                despawn_far_chunks,
            ),
        );
    }
}

fn load_chunks_around_player(
    camera: Query<&Transform, With<Camera>>,
    mut world_state: ResMut<ChunkWorldState>,
    mut commands: Commands,
    existing_chunks: Query<&ChunkEntity>,
) {
    let Ok(cam_transform) = camera.get_single() else {
        return;
    };

    let player_chunk_x = (cam_transform.translation.x / CHUNK_SIZE_F).floor() as i32;
    let player_chunk_y = (cam_transform.translation.y / CHUNK_SIZE_F).floor() as i32;
    let player_chunk_z = (cam_transform.translation.z / CHUNK_SIZE_F).floor() as i32;

    let existing_positions: HashSet<IVec3> = existing_chunks.iter().map(|ce| ce.position).collect();

    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dy in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = IVec3::new(
                    player_chunk_x + dx,
                    player_chunk_y + dy,
                    player_chunk_z + dz,
                );

                if existing_positions.contains(&chunk_pos) {
                    continue;
                }
                if world_state.chunks.get_chunk(chunk_pos).is_some() {
                    continue;
                }

                let chunk = world_state.terrain_gen.generate_chunk(chunk_pos);
                world_state.chunks.insert_chunk(chunk);

                commands.spawn((
                    ChunkEntity {
                        position: chunk_pos,
                    },
                    NeedsMesh,
                ));
            }
        }
    }
}

fn spawn_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    needs_mesh: Query<(Entity, &ChunkEntity), With<NeedsMesh>>,
    world_state: Res<ChunkWorldState>,
) {
    for (entity, chunk_entity) in needs_mesh.iter() {
        if let Some(chunk) = world_state.chunks.get_chunk(chunk_entity.position) {
            let mesh = generate_chunk_mesh(chunk);
            let mesh_handle = meshes.add(mesh);
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            });

            let world_pos = IVec3::new(
                chunk_entity.position.x * CHUNK_SIZE as i32,
                chunk_entity.position.y * CHUNK_SIZE as i32,
                chunk_entity.position.z * CHUNK_SIZE as i32,
            );

            commands
                .entity(entity)
                .insert((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(world_pos.as_vec3()),
                ))
                .remove::<NeedsMesh>();
        }
    }
}

fn despawn_far_chunks(
    camera: Query<&Transform, With<Camera>>,
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkEntity)>,
) {
    let Ok(cam_transform) = camera.get_single() else {
        return;
    };

    let player_pos = cam_transform.translation;
    let max_distance = (RENDER_DISTANCE + 2) as f32 * CHUNK_SIZE_F;

    for (entity, chunk_entity) in chunks.iter() {
        let chunk_world_pos =
            chunk_entity.position.as_vec3() * CHUNK_SIZE_F + Vec3::splat(CHUNK_SIZE_F / 2.0);
        let distance = player_pos.distance(chunk_world_pos);

        if distance > max_distance {
            commands.entity(entity).despawn();
        }
    }
}

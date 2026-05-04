use bevy::prelude::*;

use crate::player::controller::{PlayerSettings, Velocity};
use crate::world::chunk::Material;
use crate::world::chunk_manager::ChunkWorldState;

pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.0;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_collision);
    }
}

fn is_solid_at(world_pos: IVec3, world_state: &ChunkWorldState) -> bool {
    let mat = world_state.chunks.get_block(world_pos);
    mat != Material::Air && mat != Material::Water
}

fn check_aabb_collision(position: Vec3, world_state: &ChunkWorldState) -> bool {
    let min_x = (position.x - PLAYER_HALF_WIDTH).floor() as i32;
    let max_x = (position.x + PLAYER_HALF_WIDTH).floor() as i32;
    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT).floor() as i32;
    let min_z = (position.z - PLAYER_HALF_WIDTH).floor() as i32;
    let max_z = (position.z + PLAYER_HALF_WIDTH).floor() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if is_solid_at(IVec3::new(x, y, z), world_state) {
                    return true;
                }
            }
        }
    }
    false
}

fn apply_collision(
    world_state: Res<ChunkWorldState>,
    settings: Res<PlayerSettings>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Camera>>,
    time: Res<Time>,
) {
    if settings.is_flying {
        return;
    }

    let Ok((mut transform, mut velocity)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    velocity.value.y += settings.gravity * dt;

    let pos = transform.translation;

    let new_pos_x = Vec3::new(pos.x + velocity.value.x * dt, pos.y, pos.z);
    if !check_aabb_collision(new_pos_x, &world_state) {
        transform.translation.x = new_pos_x.x;
    } else {
        velocity.value.x = 0.0;
    }

    let pos = transform.translation;
    let new_pos_y = Vec3::new(pos.x, pos.y + velocity.value.y * dt, pos.z);
    if !check_aabb_collision(new_pos_y, &world_state) {
        transform.translation.y = new_pos_y.y;
    } else {
        if velocity.value.y < 0.0 {
            transform.translation.y = pos.y.floor() + 0.01;
        }
        velocity.value.y = 0.0;
    }

    let pos = transform.translation;
    let new_pos_z = Vec3::new(pos.x, pos.y, pos.z + velocity.value.z * dt);
    if !check_aabb_collision(new_pos_z, &world_state) {
        transform.translation.z = new_pos_z.z;
    } else {
        velocity.value.z = 0.0;
    }
}

pub fn is_ground_beneath(translation: Vec3, world_state: &ChunkWorldState) -> bool {
    let foot_block_y = (translation.y - 0.05).floor() as i32;
    let check_pos = IVec3::new(
        translation.x.floor() as i32,
        foot_block_y,
        translation.z.floor() as i32,
    );
    let mat = world_state.chunks.get_block(check_pos);
    mat != Material::Air && mat != Material::Water
}

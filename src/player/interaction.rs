use bevy::prelude::*;

use crate::player::controller::{MouseLocked, PlayerCamera};
use crate::world::chunk::{Block, Material};
use crate::world::chunk_manager::ChunkWorldState;

#[derive(Resource)]
pub struct InteractionState {
    pub selected_material: Material,
    pub reach_distance: f32,
}

impl Default for InteractionState {
    fn default() -> Self {
        InteractionState {
            selected_material: Material::Stone,
            reach_distance: 8.0,
        }
    }
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InteractionState::default())
            .add_systems(Startup, setup_crosshair)
            .add_systems(Update, (block_interaction, select_material));
    }
}

fn setup_crosshair(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            margin: UiRect::new(Val::Px(-2.0), Val::Px(-2.0), Val::Px(-2.0), Val::Px(-2.0)),
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

fn raycast_voxels(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    world_state: &ChunkWorldState,
) -> Option<(IVec3, IVec3)> {
    let dir = direction.normalize();

    let mut pos = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let step = IVec3::new(
        if dir.x > 0.0 { 1 } else { -1 },
        if dir.y > 0.0 { 1 } else { -1 },
        if dir.z > 0.0 { 1 } else { -1 },
    );

    let mut t_max = Vec3::new(
        if dir.x != 0.0 {
            let next_x = if dir.x > 0.0 {
                (pos.x + 1) as f32 - origin.x
            } else {
                origin.x - pos.x as f32
            };
            next_x / dir.x.abs()
        } else {
            f32::MAX
        },
        if dir.y != 0.0 {
            let next_y = if dir.y > 0.0 {
                (pos.y + 1) as f32 - origin.y
            } else {
                origin.y - pos.y as f32
            };
            next_y / dir.y.abs()
        } else {
            f32::MAX
        },
        if dir.z != 0.0 {
            let next_z = if dir.z > 0.0 {
                (pos.z + 1) as f32 - origin.z
            } else {
                origin.z - pos.z as f32
            };
            next_z / dir.z.abs()
        } else {
            f32::MAX
        },
    );

    let t_delta = Vec3::new(
        if dir.x != 0.0 {
            1.0 / dir.x.abs()
        } else {
            f32::MAX
        },
        if dir.y != 0.0 {
            1.0 / dir.y.abs()
        } else {
            f32::MAX
        },
        if dir.z != 0.0 {
            1.0 / dir.z.abs()
        } else {
            f32::MAX
        },
    );

    let mut last_normal = IVec3::ZERO;
    let mut distance = 0.0f32;

    while distance < max_distance {
        let material = world_state.chunks.get_block(pos);
        if material != Material::Air {
            return Some((pos, last_normal));
        }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            distance = t_max.x;
            t_max.x += t_delta.x;
            last_normal = IVec3::new(-step.x, 0, 0);
            pos.x += step.x;
        } else if t_max.y < t_max.z {
            distance = t_max.y;
            t_max.y += t_delta.y;
            last_normal = IVec3::new(0, -step.y, 0);
            pos.y += step.y;
        } else {
            distance = t_max.z;
            t_max.z += t_delta.z;
            last_normal = IVec3::new(0, 0, -step.z);
            pos.z += step.z;
        }
    }

    None
}

fn block_interaction(
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut world_state: ResMut<ChunkWorldState>,
    interaction_state: Res<InteractionState>,
    mouse_locked: Res<MouseLocked>,
) {
    if !mouse_locked.0 {
        return;
    }

    let Ok(cam_transform) = camera.get_single() else {
        return;
    };

    let origin = cam_transform.translation;
    let direction = cam_transform.forward();

    if let Some((hit_pos, face_normal)) = raycast_voxels(
        origin,
        direction.into(),
        interaction_state.reach_distance,
        &world_state,
    ) {
        if mouse_button.just_pressed(MouseButton::Left) {
            world_state.chunks.set_block(hit_pos, Block::air());
        }

        if mouse_button.just_pressed(MouseButton::Right) {
            let place_pos = hit_pos + face_normal;
            let current = world_state.chunks.get_block(place_pos);
            if current == Material::Air {
                world_state
                    .chunks
                    .set_block(place_pos, Block::new(interaction_state.selected_material));
            }
        }
    }
}

fn select_material(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_state: ResMut<InteractionState>,
) {
    let materials = [
        Material::Stone,
        Material::Dirt,
        Material::Sand,
        Material::Water,
        Material::Lava,
        Material::Wood,
        Material::Metal,
        Material::Ice,
        Material::Fire,
    ];

    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            interaction_state.selected_material = materials[i];
        }
    }
}

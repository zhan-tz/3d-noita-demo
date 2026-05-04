use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::physics::collision::is_ground_beneath;
use crate::world::chunk_manager::ChunkWorldState;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Debug, Reflect)]
pub struct MoveSpeed {
    pub walk: f32,
    pub fly: f32,
}

impl Default for MoveSpeed {
    fn default() -> Self {
        MoveSpeed {
            walk: 5.0,
            fly: 15.0,
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct LookSpeed {
    pub sensitivity: f32,
}

impl Default for LookSpeed {
    fn default() -> Self {
        LookSpeed { sensitivity: 0.002 }
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct Velocity {
    pub value: Vec3,
}

#[derive(Resource, Debug)]
pub struct PlayerSettings {
    pub gravity: f32,
    pub jump_speed: f32,
    pub is_flying: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        PlayerSettings {
            gravity: -20.0,
            jump_speed: 8.0,
            is_flying: true,
        }
    }
}

#[derive(Resource)]
pub struct MouseLocked(pub bool);

impl Default for MouseLocked {
    fn default() -> Self {
        MouseLocked(false)
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerSettings::default())
            .insert_resource(MouseLocked::default())
            .add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    player_look,
                    player_fly_movement,
                    player_jump,
                    toggle_fly_mode,
                    mouse_lock_system,
                )
                    .chain(),
            );
    }
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 150.0, 8.0).looking_at(Vec3::new(8.0, 130.0, 24.0), Vec3::Y),
        Player,
        PlayerCamera,
        MoveSpeed::default(),
        LookSpeed::default(),
        Velocity::default(),
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<PlayerSettings>,
    speed: Query<&MoveSpeed, With<Player>>,
    mut velocity: Query<&mut Velocity, With<Player>>,
    camera: Query<&Transform, With<PlayerCamera>>,
) {
    let Ok(move_speed) = speed.get_single() else {
        return;
    };
    let Ok(cam_transform) = camera.get_single() else {
        return;
    };
    let Ok(mut vel) = velocity.get_single_mut() else {
        return;
    };

    let speed = if settings.is_flying {
        move_speed.fly
    } else {
        move_speed.walk
    };
    let _delta = time.delta_secs();

    let forward = cam_transform.forward();
    let right = cam_transform.right();

    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward_xz;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward_xz;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right_xz;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right_xz;
    }

    if settings.is_flying {
        if keyboard.pressed(KeyCode::Space) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            direction.y -= 1.0;
        }
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
    }

    if settings.is_flying {
        vel.value = direction * speed;
    } else {
        vel.value.x = direction.x * speed;
        vel.value.z = direction.z * speed;
    }
}

fn player_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_locked: Res<MouseLocked>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
    look_speed: Query<&LookSpeed, With<PlayerCamera>>,
) {
    if !mouse_locked.0 {
        return;
    }

    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };
    let Ok(speed) = look_speed.get_single() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let yaw = -delta.x * speed.sensitivity;
    let pitch = -delta.y * speed.sensitivity;

    transform.rotate_y(yaw);

    let current_pitch = transform.rotation.to_euler(EulerRot::YXZ).1;
    let new_pitch = (current_pitch + pitch).clamp(
        -std::f32::consts::FRAC_PI_2 + 0.01,
        std::f32::consts::FRAC_PI_2 - 0.01,
    );

    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, new_pitch, 0.0);
}

fn player_fly_movement(
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    velocity: Query<&Velocity, With<Player>>,
    mut transform: Query<&mut Transform, With<PlayerCamera>>,
) {
    if !settings.is_flying {
        return;
    }

    let Ok(vel) = velocity.get_single() else {
        return;
    };
    let Ok(mut tf) = transform.get_single_mut() else {
        return;
    };
    tf.translation += vel.value * time.delta_secs();
}

fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<PlayerSettings>,
    world_state: Res<ChunkWorldState>,
    mut velocity: Query<&mut Velocity, With<Player>>,
    transform: Query<&Transform, With<PlayerCamera>>,
) {
    if settings.is_flying {
        return;
    }
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(mut vel) = velocity.get_single_mut() else {
        return;
    };
    let Ok(tf) = transform.get_single() else {
        return;
    };

    if is_ground_beneath(tf.translation, &world_state) {
        vel.value.y = settings.jump_speed;
    }
}

fn toggle_fly_mode(keyboard: Res<ButtonInput<KeyCode>>, mut settings: ResMut<PlayerSettings>) {
    if keyboard.just_pressed(KeyCode::F3) {
        settings.is_flying = !settings.is_flying;
    }
}

fn mouse_lock_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_locked: ResMut<MouseLocked>,
    mut windows: Query<&mut Window>,
) {
    if mouse_button.just_pressed(MouseButton::Left) && !mouse_locked.0 {
        mouse_locked.0 = true;
        for mut window in windows.iter_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) && mouse_locked.0 {
        mouse_locked.0 = false;
        for mut window in windows.iter_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

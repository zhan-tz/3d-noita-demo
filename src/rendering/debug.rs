use bevy::prelude::*;

use crate::combat::difficulty::get_depth_label;
use crate::game::{GameState, PlayerHealth};
use crate::world::biome::get_biome;
use crate::world::chunk_manager::ChunkEntity;

#[derive(Resource, Default)]
pub struct DebugState {
    pub visible: bool,
    pub fps: f32,
    pub frame_count: u32,
    pub timer: Timer,
}

#[derive(Component)]
struct DebugText;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugState {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            ..default()
        })
        .add_systems(OnEnter(GameState::Playing), spawn_debug_text)
        .add_systems(OnExit(GameState::Playing), despawn_debug_text)
        .add_systems(
            Update,
            (toggle_debug, update_fps, update_debug_text)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn spawn_debug_text(mut commands: Commands) {
    commands.spawn((
        DebugText,
        Text::new(""),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(90.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn despawn_debug_text(mut commands: Commands, query: Query<Entity, With<DebugText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<DebugState>) {
    if keys.just_pressed(KeyCode::F3) {
        state.visible = !state.visible;
    }
}

fn update_fps(time: Res<Time>, mut state: ResMut<DebugState>) {
    state.frame_count += 1;
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
        state.fps = state.frame_count as f32 / state.timer.elapsed_secs().max(0.001);
        state.frame_count = 0;
        state.timer.reset();
    }
}

fn update_debug_text(
    state: Res<DebugState>,
    mut text_query: Query<&mut Text, With<DebugText>>,
    player_query: Query<&Transform, With<PlayerHealth>>,
    chunk_query: Query<&ChunkEntity>,
) {
    if !state.visible {
        for mut text in text_query.iter_mut() {
            text.0 = String::new();
        }
        return;
    }

    let player_pos = player_query.iter().next();
    let chunk_count = chunk_query.iter().count();

    let (coords_line, biome_line) = if let Some(transform) = player_pos {
        let pos = transform.translation;
        let y = pos.y as i32;
        let biome = get_biome(y);
        let depth_label = get_depth_label(y);
        (
            format!("Pos: {:.1}, {:.1}, {:.1}", pos.x, pos.y, pos.z),
            format!("Biome: {:?} ({})", biome, depth_label),
        )
    } else {
        ("Pos: --".to_string(), "Biome: --".to_string())
    };

    let content = format!(
        "FPS: {:.0}\n{}\nChunks: {}\n{}",
        state.fps, coords_line, chunk_count, biome_line
    );

    for mut text in text_query.iter_mut() {
        text.0 = content.clone();
    }
}

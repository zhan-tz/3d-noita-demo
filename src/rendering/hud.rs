use bevy::prelude::*;

use crate::game::{GameState, PlayerDepth, PlayerHealth};

#[derive(Component)]
struct HUDHealthText;

#[derive(Component)]
struct HUDDepthText;

#[derive(Component)]
struct HUDCrosshair;

#[derive(Component)]
struct HUDMarker;

pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), hud_spawn)
            .add_systems(OnExit(GameState::Playing), hud_despawn)
            .add_systems(Update, hud_update.run_if(in_state(GameState::Playing)));
    }
}

fn hud_spawn(mut commands: Commands) {
    commands.spawn((HUDMarker,));

    commands.spawn((
        HUDHealthText,
        Text::new("HP: 100"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        HUDDepthText,
        Text::new("Depth: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        HUDCrosshair,
        Text::new("+"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

fn hud_despawn(mut commands: Commands, query: Query<Entity, With<HUDMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn hud_update(
    health_query: Query<&PlayerHealth>,
    depth_query: Query<&PlayerDepth>,
    mut health_text: Query<&mut Text, With<HUDHealthText>>,
    mut depth_text: Query<&mut Text, With<HUDDepthText>>,
) {
    let health = health_query.iter().next();
    let depth = depth_query.iter().next();

    if let Some(h) = health {
        for mut text in health_text.iter_mut() {
            text.0 = format!("HP: {:.0}", h.hp);
        }
    }
    if let Some(d) = depth {
        for mut text in depth_text.iter_mut() {
            text.0 = format!("Depth: {}", d.depth);
        }
    }
}

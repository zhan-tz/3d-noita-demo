use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
    Victory,
}

#[derive(Component)]
pub struct PlayerHealth {
    pub hp: f32,
    pub max_hp: f32,
}

#[derive(Component)]
pub struct PlayerDepth {
    pub depth: i32,
}

#[derive(Component)]
struct MenuMarker;

#[derive(Component)]
struct HUDMarker;

#[derive(Resource, Default)]
pub struct WorldState {
    pub depth: i32,
    pub score: i32,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldState::default())
            .add_systems(OnEnter(GameState::MainMenu), menu_enter)
            .add_systems(OnExit(GameState::MainMenu), menu_exit)
            .add_systems(OnEnter(GameState::Playing), playing_enter)
            .add_systems(OnEnter(GameState::GameOver), gameover_enter)
            .add_systems(OnEnter(GameState::Victory), victory_enter)
            .add_systems(Update, menu_input.run_if(in_state(GameState::MainMenu)))
            .add_systems(
                Update,
                (
                    player_health_check,
                    player_depth_update,
                    environmental_damage,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, gameover_input.run_if(in_state(GameState::GameOver)))
            .add_systems(Update, victory_input.run_if(in_state(GameState::Victory)));
    }
}

fn menu_enter(mut commands: Commands) {
    commands.spawn((MenuMarker,));
}

fn menu_exit(mut commands: Commands, query: Query<Entity, With<MenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn playing_enter(mut commands: Commands) {
    commands.insert_resource(WorldState { depth: 0, score: 0 });
    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        PlayerHealth {
            hp: 100.0,
            max_hp: 100.0,
        },
        PlayerDepth { depth: 0 },
        Name::new("PlayerHealth"),
    ));
}

fn gameover_enter(mut commands: Commands) {
    commands.spawn((HUDMarker,));
}

fn victory_enter(mut commands: Commands) {
    commands.spawn((HUDMarker,));
}

fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(GameState::Playing);
    }
}

fn gameover_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next.set(GameState::MainMenu);
    }
}

fn victory_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next.set(GameState::MainMenu);
    }
}

fn player_health_check(mut next: ResMut<NextState<GameState>>, query: Query<&PlayerHealth>) {
    for health in query.iter() {
        if health.hp <= 0.0 {
            next.set(GameState::GameOver);
        }
    }
}

fn player_depth_update(mut query: Query<(&Transform, &mut PlayerDepth)>) {
    for (transform, mut depth) in query.iter_mut() {
        depth.depth = transform.translation.y as i32;
    }
}

fn environmental_damage(mut query: Query<(&PlayerDepth, &mut PlayerHealth)>) {
    for (depth, mut health) in query.iter_mut() {
        let dmg = (depth.depth.max(0) as f32) * 0.05;
        if dmg > 0.0 {
            health.hp = (health.hp - dmg).max(0.0);
        }
    }
}

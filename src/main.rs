mod combat;
mod inventory;
mod physics;
mod player;
mod rendering;
mod world;

use bevy::prelude::*;
mod game;
use game::{GamePlugin, GameState};
use physics::collision::CollisionPlugin;
use player::controller::PlayerPlugin;
use player::InteractionPlugin;
use world::chunk_manager::ChunkManagerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins(GamePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ChunkManagerPlugin)
        .add_plugins(CollisionPlugin)
        .add_plugins(InteractionPlugin)
        .run();
}

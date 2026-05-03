mod world;
mod physics;
mod player;
mod combat;
mod rendering;
mod inventory;

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}

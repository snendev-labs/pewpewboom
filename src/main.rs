use bevy::{
    prelude::{App, Commands, Startup},
    DefaultPlugins,
};

use pewpewboom::{tilemap::Tilemap, PewPewBoomPlugins};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PewPewBoomPlugins);
    app.add_systems(Startup, spawn_camera);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(pewpewboom::camera::PlayerCamera);
    commands.spawn(Tilemap::bundle());
}

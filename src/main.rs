use bevy::prelude::*;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

#[derive(Component)]
struct Paddle {
    move_up: KeyCode,
    move_down: KeyCode,
    
}


fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, setup_camera);
    app.run();
}

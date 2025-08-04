use bevy::prelude::*;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

#[derive(Component)]
struct Paddle {
    move_up: KeyCode,
    move_down: KeyCode,
}

fn spawn_players(mut commands: Commands) {
    commands.spawn((Sprite {
        color: Color::BLACK,
        custom_size: Some(Vec2::new(700., 500.)),
        ..default()
    },));

    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(10.0, 150.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-300.0, 0.0, 0.0)),
        Paddle {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
        },
    ));

    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(10.0, 150.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(300.0, 0.0, 0.0)),
        Paddle {
            move_up: KeyCode::ArrowUp,
            move_down: KeyCode::ArrowDown,
        },
    ));
}

fn move_paddle(mut paddles: Query<(&mut Transform, &Paddle)>, input: Res<ButtonInput<KeyCode>>) {
    for (mut pos, settings) in &mut paddles {
        if input.pressed(settings.move_up) {
            pos.translation.y += 5.0;
        }

        if input.pressed(settings.move_down) {
            pos.translation.y -= 5.0;
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (setup_camera, spawn_players));
    app.add_systems(Update, move_paddle);
    app.run();
}

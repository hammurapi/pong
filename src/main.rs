use std::f32::consts::PI;

use bevy::{audio::Volume, prelude::*, text::cosmic_text::rustybuzz::script::PHOENICIAN};

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

fn move_paddle(
    mut paddles: Query<(&mut Transform, &Paddle)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut pos, settings) in &mut paddles {
        if input.pressed(settings.move_up) {
            pos.translation.y += 100.0 * time.delta_secs();
            pos.translation.y = pos.translation.y.clamp(-250.0 + 75.0, 250.0 - 75.0);
        }

        if input.pressed(settings.move_down) {
            pos.translation.y -= 100.0 * time.delta_secs();
            pos.translation.y = pos.translation.y.clamp(-250.0 + 75.0, 250.0 - 75.0);
        }
    }
}

#[derive(Component)]
struct Ball(Vec2);

fn spawn_ball(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        Ball(Vec2::new(-100.0, 0.0)),
    ));
}

fn move_ball(mut ball: Query<(&mut Transform, &Ball)>, time: Res<Time>) {
    for (mut pos, ball) in &mut ball {
        pos.translation += ball.0.extend(0.) * time.delta_secs();
    }
}

const BWIDTH: f32 = 25.;
const PWIDTH: f32 = 10.;
const PHIGTH: f32 = 150.;
const MAXBOUNCEANGLE: f32 = std::f32::consts::FRAC_PI_8; // 22.5 degrees

#[derive(Resource)]
struct GameAudio {
    bounce_sound: Handle<AudioSource>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bounce_sound = asset_server.load("sounds/surprise-sound-effect-99300.ogg");
    commands.insert_resource(GameAudio { bounce_sound });
}

fn ball_collide(
    mut balls: Query<(&Transform, &mut Ball)>,
    paddles: Query<&Transform, With<Paddle>>,
    mut commands: Commands,
    audio: Res<GameAudio>,
) {
    for (ball, mut velocity) in &mut balls {
        if ball.translation.y.abs() + BWIDTH / 2. > 250. {
            velocity.0.y *= -1.;
        }

        for paddle in &paddles {
            if ball.translation.x - BWIDTH / 2. < paddle.translation.x + PWIDTH / 2.
                && ball.translation.y - BWIDTH / 2. < paddle.translation.y + PHIGTH / 2.
                && ball.translation.x + BWIDTH / 2. > paddle.translation.x - PWIDTH / 2.
                && ball.translation.y + BWIDTH / 2. > paddle.translation.y - PHIGTH / 2.
            {
                let intersection_y = (ball.translation.y - paddle.translation.y);
                let normalized_intersect_y = intersection_y / (PHIGTH / 2. + BWIDTH / 2.);

                let abs_speed = velocity.0.length();
                let mut bounce_angle = (velocity.0.x / abs_speed).acos();
                dbg!(bounce_angle.to_degrees());
                dbg!((normalized_intersect_y * MAXBOUNCEANGLE).to_degrees());

                bounce_angle += PI;

                velocity.0 = Vec2::new(
                    abs_speed * bounce_angle.cos(),
                    abs_speed * bounce_angle.sin(),
                );

                // Play bounce sound
                commands.spawn((
                    AudioPlayer::new(audio.bounce_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(
        Startup,
        (setup_camera, spawn_players, spawn_ball, load_sounds),
    );
    app.add_systems(Update, (move_paddle, move_ball, ball_collide));
    app.run();
}

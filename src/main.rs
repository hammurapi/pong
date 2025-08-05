use bevy::{audio::Volume, prelude::*, text::cosmic_text::rustybuzz::script::PHOENICIAN};
use rand;
use std::f32::consts::PI;

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
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return; // Don't move paddles if game is over
    }
    
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

fn move_ball(
    mut ball: Query<(&mut Transform, &Ball)>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return; // Don't move ball if game is over
    }
    
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
    paddle_bounce: Handle<AudioSource>,
    wall_bounce: Handle<AudioSource>,
}

#[derive(Resource)]
struct Score {
    left: u32,
    right: u32,
}

#[derive(Resource)]
struct GameState {
    game_over: bool,
    winner: Option<String>,
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paddle_bounce = asset_server
        .load("sounds/funny-sound-effect-for-quotjack-in-the-boxquot-sound-ver3-110925.ogg");
    let wall_bounce = asset_server.load("sounds/surprise-sound-effect-99300.ogg");
    commands.insert_resource(GameAudio {
        paddle_bounce,
        wall_bounce,
    });
}

fn spawn_ui(mut commands: Commands) {
    // Create a container that fills the screen
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(Val::Percent(15.0)),
            ..default()
        },
    )).with_children(|parent| {
        // Score text as a child
        parent.spawn((
            Text::new("0 - 0"),
            TextLayout::new_with_justify(JustifyText::Center),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)), // White with 40% opacity
        ));
    });
}

fn check_game_over(
    score: Res<Score>,
    mut game_state: ResMut<GameState>,
) {
    if !game_state.game_over {
        if score.left >= 10 {
            game_state.game_over = true;
            game_state.winner = Some("Left Player".to_string());
        } else if score.right >= 10 {
            game_state.game_over = true;
            game_state.winner = Some("Right Player".to_string());
        }
    }
}

fn update_score_display(
    score: Res<Score>,
    game_state: Res<GameState>,
    mut text_query: Query<&mut Text>,
) {
    if score.is_changed() || game_state.is_changed() {
        for mut text in &mut text_query {
            if game_state.game_over {
                if let Some(winner) = &game_state.winner {
                    text.0 = format!("{} WINS!\n{} - {}", winner, score.left, score.right);
                }
            } else {
                text.0 = format!("{} - {}", score.left, score.right);
            }
        }
    }
}

fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameState>,
    mut exit: EventWriter<AppExit>,
) {
    if game_state.game_over && input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn check_ball_out_of_bounds(
    mut balls: Query<(Entity, &Transform, &mut Ball)>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return; // Don't check bounds if game is over
    }
    
    for (entity, ball_transform, mut velocity) in &mut balls {
        // Check if ball went off left or right side
        if ball_transform.translation.x < -350.0 {
            // Right player scores
            score.right += 1;
            reset_ball(entity, &mut velocity, &mut commands);
        } else if ball_transform.translation.x > 350.0 {
            // Left player scores
            score.left += 1;
            reset_ball(entity, &mut velocity, &mut commands);
        }
    }
}

fn reset_ball(entity: Entity, velocity: &mut Ball, commands: &mut Commands) {
    // Reset ball to center
    commands
        .entity(entity)
        .insert(Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)));

    // Give ball random direction (left or right)
    let direction = if rand::random::<bool>() { -1.0 } else { 1.0 };
    velocity.0 = Vec2::new(direction * 100.0, 0.0);
}

fn ball_collide(
    mut balls: Query<(&Transform, &mut Ball)>,
    paddles: Query<&Transform, With<Paddle>>,
    mut commands: Commands,
    audio: Res<GameAudio>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return; // Don't process collisions if game is over
    }
    
    for (ball, mut velocity) in &mut balls {
        if ball.translation.y.abs() + BWIDTH / 2. > 250. {
            velocity.0.y *= -1.;

            // Play bounce sound
            commands.spawn((
                AudioPlayer::new(audio.wall_bounce.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }

        for paddle in &paddles {
            if ball.translation.x - BWIDTH / 2. < paddle.translation.x + PWIDTH / 2.
                && ball.translation.y - BWIDTH / 2. < paddle.translation.y + PHIGTH / 2.
                && ball.translation.x + BWIDTH / 2. > paddle.translation.x - PWIDTH / 2.
                && ball.translation.y + BWIDTH / 2. > paddle.translation.y - PHIGTH / 2.
            {
                let intersection_y = ball.translation.y - paddle.translation.y;
                let normalized_intersect_y = (intersection_y / (PHIGTH / 2.)).clamp(-1.0, 1.0);

                let abs_speed = velocity.0.length();

                // Calculate bounce angle based on where ball hits paddle
                let bounce_angle = normalized_intersect_y * MAXBOUNCEANGLE;

                // Determine direction based on which side of paddle was hit
                let direction = -paddle.translation.x.signum();
                velocity.0 = Vec2::new(
                    direction * abs_speed * bounce_angle.cos(),
                    abs_speed * bounce_angle.sin(),
                );

                // Play bounce sound
                commands.spawn((
                    AudioPlayer::new(audio.paddle_bounce.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.insert_resource(Score { left: 0, right: 0 });
    app.insert_resource(GameState { 
        game_over: false, 
        winner: None 
    });
    app.add_systems(
        Startup,
        (
            setup_camera,
            spawn_players,
            spawn_ball,
            spawn_ui,
            load_sounds,
        ),
    );
    app.add_systems(
        Update,
        (
            move_paddle,
            move_ball,
            ball_collide,
            check_ball_out_of_bounds,
            check_game_over,
            update_score_display,
            game_over_input,
        ),
    );
    app.run();
}

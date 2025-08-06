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
        Visibility::Hidden, // Start hidden since game starts in StartScreen phase
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
        Visibility::Hidden, // Start hidden since game starts in StartScreen phase
    ));
}

fn move_paddle(
    mut paddles: Query<(&mut Transform, &Paddle)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
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

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct StartScreenText;

fn spawn_ball(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(25.0, 25.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        Ball(Vec2::new(-100.0, 0.0)),
        Visibility::Hidden, // Start hidden since game starts in StartScreen phase
    ));
}

fn move_ball(
    mut ball: Query<(&mut Transform, &Ball)>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
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

#[derive(Resource, PartialEq, Eq, Clone, Copy)]
enum GamePhase {
    StartScreen,
    Playing,
}

#[derive(Resource)]
struct GameState {
    phase: GamePhase,
    winner: Option<String>,
}

#[derive(Resource)]
struct GameTimer {
    elapsed: f32,
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
    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(Val::Percent(15.0)),
            ..default()
        },))
        .with_children(|parent| {
            // Score text as a child
            parent.spawn((
                Text::new("0 - 0"),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)), // White with 40% opacity
                ScoreText,
                Visibility::Hidden, // Start hidden since game starts in StartScreen phase
            ));
        });
}

fn spawn_start_screen(mut commands: Commands) {
    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PONG\nPress any key to start\nPress ESC to exit"),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                StartScreenText,
            ));
        });
}

fn start_screen_input(
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    start_screen_nodes: Query<Entity, (With<Node>, With<Children>, Without<ScoreText>)>,
    children_query: Query<&Children>,
    start_screen_text_query: Query<Entity, With<StartScreenText>>,
) {
    if game_state.phase == GamePhase::StartScreen {
        if input.just_pressed(KeyCode::Escape) {
            exit.write(AppExit::Success);
        } else if input.get_just_pressed().next().is_some() {
            game_state.phase = GamePhase::Playing;
            game_state.winner = None;
            // Despawn only the start screen nodes that contain StartScreenText
            for node_entity in &start_screen_nodes {
                if let Ok(children) = children_query.get(node_entity) {
                    for child in children {
                        if start_screen_text_query.contains(*child) {
                            commands.entity(node_entity).despawn();
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn check_game_over(score: Res<Score>, mut game_state: ResMut<GameState>, mut commands: Commands) {
    if game_state.phase == GamePhase::Playing {
        if score.left >= 10 {
            game_state.phase = GamePhase::StartScreen;
            game_state.winner = Some("Left Player".to_string());
            commands.insert_resource(Score { left: 0, right: 0 });
            spawn_start_screen(commands);
        } else if score.right >= 10 {
            game_state.phase = GamePhase::StartScreen;
            game_state.winner = Some("Right Player".to_string());
            commands.insert_resource(Score { left: 0, right: 0 });
            spawn_start_screen(commands);
        }
    }
}

fn update_score_display(
    score: Res<Score>,
    game_state: Res<GameState>,
    mut score_text_query: Query<(&mut Text, &mut Visibility), With<ScoreText>>,
    mut start_screen_text_query: Query<&mut Text, (With<StartScreenText>, Without<ScoreText>)>,
) {
    // Update score text during gameplay
    if game_state.phase == GamePhase::Playing {
        for (mut text, mut visibility) in &mut score_text_query {
            text.0 = format!("{} - {}", score.left, score.right);
            *visibility = Visibility::Visible;
        }
    } else {
        // Hide score text during start screen
        for (_, mut visibility) in &mut score_text_query {
            *visibility = Visibility::Hidden;
        }
    }

    // Update start screen text
    if game_state.phase == GamePhase::StartScreen {
        for mut text in &mut start_screen_text_query {
            if let Some(winner) = &game_state.winner {
                text.0 = format!(
                    "{} WINS!\nPress any key to restart\nPress ESC to exit",
                    winner
                );
            } else {
                text.0 = "PONG\nPress any key to start\nPress ESC to exit".to_string();
            }
        }
    }
}

fn update_game_visibility(
    game_state: Res<GameState>,
    mut paddle_query: Query<&mut Visibility, (With<Paddle>, Without<Ball>)>,
    mut ball_query: Query<&mut Visibility, (With<Ball>, Without<Paddle>)>,
) {
    let visibility = if game_state.phase == GamePhase::Playing {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Update paddle visibility
    for mut paddle_visibility in &mut paddle_query {
        *paddle_visibility = visibility;
    }

    // Update ball visibility
    for mut ball_visibility in &mut ball_query {
        *ball_visibility = visibility;
    }
}

fn check_ball_out_of_bounds(
    mut balls: Query<(Entity, &Transform, &mut Ball)>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    game_state: Res<GameState>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
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

fn increase_ball_speed(
    mut balls: Query<&mut Ball>,
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    game_state: Res<GameState>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
    }

    game_timer.elapsed += time.delta_secs();

    // Increase speed every 10 seconds
    if game_timer.elapsed >= 10.0 {
        game_timer.elapsed = 0.0;

        for mut ball in &mut balls {
            // Increase speed by 10% each time
            ball.0 *= 1.1;
        }
    }
}

fn reset_ball(entity: Entity, velocity: &mut Ball, commands: &mut Commands) {
    // Reset ball to center
    commands
        .entity(entity)
        .insert(Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)));

    // Give ball random direction (left or right) with base speed
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
    if game_state.phase != GamePhase::Playing {
        return;
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
        phase: GamePhase::StartScreen,
        winner: None,
    });
    app.insert_resource(GameTimer { elapsed: 0.0 });
    app.add_systems(
        Startup,
        (
            setup_camera,
            spawn_players,
            spawn_ball,
            spawn_ui,
            load_sounds,
            spawn_start_screen,
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
            update_game_visibility,
            start_screen_input,
            increase_ball_speed,
        ),
    );
    app.run();
}

use bevy::{
    prelude::*,
    //sprite::collide_aabb::{collide, Collision},
    time::FixedTimestep
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// PADDLE DATA
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const PADDLE_SPEED : f32 = 500.0;
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

// PROJECTILE DATA
const PROJECTILE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 20.0);
const PROJECTILE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.0, 1.0);

// x coordinates
const LEFT_WALL: f32 = -600.;
const RIGHT_WALL: f32 = 600.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

// UI DATA
const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn main()  {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpaceInvader)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_system(bevy::window::close_on_esc)
        .run();
}

pub struct SpaceInvader;

impl Plugin for SpaceInvader{
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_player)
                .with_system(system_spawn_projectile)
                .with_system(system_apply_velocity)
                .with_system(system_lifetime)
        );
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile;

#[derive(Component, Deref, DerefMut)]
struct LifeTime(f32);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn()
        .insert(Player)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        });

     // Scoreboard
    commands.spawn_bundle(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/dogica.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/dogica.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );
}

fn move_player(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    let mut player_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    let new_player_position = player_transform.translation.x + direction * PADDLE_SPEED * TIME_STEP;

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL +  PADDLE_SIZE.x / 2.0;
    let right_bound = RIGHT_WALL - PADDLE_SIZE.x / 2.0;
    let _upper_bound = TOP_WALL;
    let _lower_bound = BOTTOM_WALL;

    player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
}

fn system_spawn_projectile(mut commands: Commands, keyboard_input: Res<Input<KeyCode>>, query: Query<&Transform, With<Player>>, projectile_query: Query<&Projectile>) {
    let player_transform = query.single();
    
    if projectile_query.is_empty() && keyboard_input.pressed(KeyCode::Space) {
        commands
        .spawn()
        .insert(Projectile)
        .insert(Velocity(INITIAL_BALL_DIRECTION.normalize()))
        .insert(LifeTime(3.0))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: player_transform.translation,
                scale: PROJECTILE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PROJECTILE_COLOR,
                ..default()
            },
            ..default()
        });
    }
}

fn system_apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

fn system_lifetime(mut commands: Commands, mut query: Query<(Entity, &mut LifeTime)>) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= TIME_STEP;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}


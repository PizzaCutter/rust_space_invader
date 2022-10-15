use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    time::FixedTimestep
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// PADDLE DATA
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const PADDLE_SPEED : f32 = 500.0;
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const PADDLE_BOTTOM_OFFSET : f32 = 25.0;

// PROJECTILE DATA
const PROJECTILE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 20.0);
const PROJECTILE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const PROJECITLE_SPEED: f32 = 1000.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.0, 1.0);

// x coordinates
const LEFT_WALL: f32 = -600.;
const RIGHT_WALL: f32 = 600.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

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
        .run();
}

pub struct SpaceInvader;

impl Plugin for SpaceInvader{
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(setup)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(move_player.before(check_for_collisions))
                .with_system(system_spawn_projectile.before(check_for_collisions))
                .with_system(system_apply_velocity.before(check_for_collisions))
                .with_system(check_for_collisions.before(system_lifetime))
                .with_system(system_lifetime)
                .with_system(play_collision_sound.after(system_lifetime))
        )
        .add_system(bevy::window::close_on_esc);
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

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

struct CollisionSound(Handle<AudioSource>);

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;

        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));


    // Spawning player
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, BOTTOM_WALL + PADDLE_BOTTOM_OFFSET, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        });

    // Walls
    commands.spawn_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn_bundle(WallBundle::new(WallLocation::Bottom));
    commands.spawn_bundle(WallBundle::new(WallLocation::Top));

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
        .insert(Velocity(INITIAL_BALL_DIRECTION.normalize() * PROJECITLE_SPEED))
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
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
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

fn check_for_collisions(
    mut _commands: Commands,
    mut projectile_query: Query<(&Transform, &mut LifeTime), With<Projectile>>,
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>
) {
    if projectile_query.is_empty() {
        return;
    }

    let (projectile_transform, mut lifetime) = projectile_query.single_mut();
    let projectile_size = projectile_transform.scale.truncate();

    for (_collider_entity, transform) in &collider_query {
        let collision = collide(
            projectile_transform.translation,
            projectile_size,
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(collision) = collision {
            // Sends a collision event so that other systems ca react to the collision
            collision_events.send_default();
            
            // reflect the ball when it collides
            let mut destroy_projectile = false;

            // only reflect if the ball's velocity is going in the opposite direction of the collision
            match collision {
                Collision::Left => destroy_projectile = true,
                Collision::Right => destroy_projectile = true,
                Collision::Top => destroy_projectile = true,
                Collision::Bottom => destroy_projectile = true,
                Collision::Inside => { /* do nothing */}
            }

            if destroy_projectile {
                lifetime.0 = 0.0;
            }
        }
    }

}

fn play_collision_sound(
    collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame
        collision_events.clear();
        audio.play(sound.0.clone());
    }
}


use rand::prelude::*;
use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Speed {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct PlayerControlled;

#[derive(Component)]
struct AIControlled {
    update_timer: Timer,
    update_freq_min: f32,
    update_freq_max: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_movement, ai_movement, apply_moves).chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            // mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            mesh: meshes
                .add(shape::RegularPolygon::new(10.0, 3).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
            ..default()
        },
        Velocity { x: 0.0, y: 0.0 },
        Speed { x: 500.0, y: 500.0 },
        PlayerControlled,
    ));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
            ..default()
        },
        Velocity { x: 330.0, y: 330.0 },
        Speed { x: 333.0, y: 333.0 },
        AIControlled {
            update_timer: Timer::new(
                Duration::from_secs_f32(rng.gen_range(0.333..1.8)),
                TimerMode::Once,
            ),
            update_freq_min: 0.333,
            update_freq_max: 1.5,
        },
    ));
}

fn ai_movement(time: Res<Time>, mut query: Query<(&mut Velocity, &mut AIControlled, &Speed)>) {
    let mut rng = rand::thread_rng();
    for (mut velocity, mut bot, speed) in query.iter_mut() {
        bot.update_timer.tick(time.delta());

        if bot.update_timer.just_finished() {
            let x: f32 = rng.gen_range(-1.0..1.0);
            let y: f32 = rng.gen_range(-1.0..1.0);
            velocity.x = speed.x * x.round();
            velocity.y = speed.y * y.round();

            let range = bot.update_freq_min..bot.update_freq_max;
            bot.update_timer
                .set_duration(Duration::from_secs_f32(rng.gen_range(range)));
            bot.update_timer.reset();
        }
    }
}

fn keyboard_movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &Speed), With<PlayerControlled>>,
) {
    for (mut velocity, speed) in query.iter_mut() {
        for key in input.get_just_pressed() {
            match key {
                KeyCode::Up => velocity.y = speed.y,
                KeyCode::Down => velocity.y = -speed.y,
                KeyCode::Right => velocity.x = speed.x,
                KeyCode::Left => velocity.x = -speed.x,
                _ => continue,
            }
        }

        for key in input.get_just_released() {
            match key {
                KeyCode::Up | KeyCode::Down => {
                    if input.any_pressed([KeyCode::Up, KeyCode::Down]) == false {
                        velocity.y = 0.0;
                    }
                }
                KeyCode::Left | KeyCode::Right => {
                    if input.any_pressed([KeyCode::Left, KeyCode::Right]) == false {
                        velocity.x = 0.0;
                    }
                }
                _ => continue,
            }
        }
    }
}

fn apply_moves(
    time: Res<Time>,
    window: Query<&Window>,
    mut query: Query<(&mut Transform, &mut Velocity)>,
) {
    let win = window.single();
    let min_x = -win.resolution.width() / 2.0;
    let max_x = win.resolution.width() / 2.0;
    let min_y = -win.resolution.height() / 2.0;
    let max_y = win.resolution.height() / 2.0;
    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();

        transform.translation.x = transform.translation.x.clamp(min_x, max_x);
        if [min_x, max_x].contains(&transform.translation.x) {
            velocity.x = -velocity.x
        }

        transform.translation.y = transform.translation.y.clamp(min_y, max_y);
        if [min_y, max_y].contains(&transform.translation.y) {
            velocity.y = -velocity.y;
        }
    }
}

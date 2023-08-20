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

#[derive(Clone, Copy)]
struct Screen {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Screen {
    fn from(win: &Window) -> Self {
        Self {
            min_x: -win.resolution.width() / 2.0,
            max_x: win.resolution.width() / 2.0,
            min_y: -win.resolution.height() / 2.0,
            max_y: win.resolution.height() / 2.0,
        }
    }

    fn contains(&self, position: &Vec3) -> bool {
        (self.min_x..self.max_x).contains(&position.x)
            && (self.min_y..self.max_y).contains(&position.y)
    }
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
    window: Query<&Window>,
) {
    let mut rng = rand::thread_rng();
    let screen = Screen::from(window.single());
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
        Speed { x: 666.0, y: 600.0 },
        PlayerControlled,
    ));
    for _ in 0..25 {
        let mut x: f32 = rng.gen_range(-1000.0..1000.0);
        let mut y: f32 = rng.gen_range(-1000.0..1000.0);
        while screen.contains(&Vec3::from([x, y, 0.0])) {
            x = rng.gen_range(-1000.0..1000.0);
            y = rng.gen_range(-1000.0..1000.0);
        }
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(10.0, 3).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
                transform: Transform::from_translation(Vec3::from([x, y, 0.])),
                ..default()
            },
            Velocity { x: 330.0, y: 330.0 },
            Speed { x: 666.0, y: 600.0 },
            AIControlled {
                update_timer: Timer::new(
                    Duration::from_secs_f32(rng.gen_range(0.25..1.8)),
                    TimerMode::Once,
                ),
                update_freq_min: 0.25,
                update_freq_max: 1.8,
            },
        ));
    }
}

fn ai_movement(
    time: Res<Time>,
    window: Query<&Window>,
    mut query: Query<(&mut Velocity, &mut AIControlled, &Transform, &Speed)>,
) {
    let mut rng = rand::thread_rng();
    let screen = Screen::from(&window.single());
    for (mut velocity, mut bot, transform, speed) in query.iter_mut() {
        bot.update_timer.tick(time.delta());

        if bot.update_timer.just_finished() {
            if screen.contains(&transform.translation) {
                // onscreen bots pick a new random direction
                let x: f32 = rng.gen_range(-1.0..1.0);
                let y: f32 = rng.gen_range(-1.0..1.0);
                velocity.x = speed.x * x.round();
                velocity.y = speed.y * y.round();
            } else {
                // offscreen bots should try to get to 0,0
                velocity.x = speed.x;
                if transform.translation.x > 0.0 {
                    velocity.x = -velocity.x;
                }

                velocity.y = speed.y;
                if transform.translation.y > 0.0 {
                    velocity.y = -velocity.y;
                }
            }

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
    let screen = Screen::from(win);
    for (mut transform, mut velocity) in query.iter_mut() {
        let mut position = &mut transform.translation;

        let was_onscreen = screen.contains(&position);

        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();

        // things that are offscreen before the movement or onscreen after the movement don't need
        // to be contained.
        if !was_onscreen || screen.contains(&position) {
            continue;
        }

        position.x = position.x.clamp(screen.min_x, screen.max_x);
        if position.x == screen.min_x {
            position.x = screen.min_x + 1.0;
            velocity.x = -velocity.x
        } else if position.x == screen.max_x {
            position.x = screen.max_x - 1.0;
            velocity.x = -velocity.x
        }

        position.y = position.y.clamp(screen.min_y, screen.max_y);
        if position.y == screen.min_y {
            position.y = screen.min_y + 1.0;
            velocity.y = -velocity.y
        } else if position.y == screen.max_y {
            position.y = screen.max_y - 1.0;
            velocity.y = -velocity.y
        }
    }
}

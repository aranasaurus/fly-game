use rand::prelude::*;
use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

const MAX_FLIES: usize = 42;

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
struct PlayerControlled {
    bounce_timer: Timer,
}

#[derive(Component)]
struct AIControlled {
    update_timer: Timer,
    update_freq_min: f32,
    update_freq_max: f32,
}

#[derive(Resource)]
struct Spawner(Timer);

impl Spawner {
    fn new() -> Self {
        Spawner(Timer::from_seconds(3.0, TimerMode::Repeating))
    }
}

impl Default for Spawner {
    fn default() -> Self {
        Self::new()
    }
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
        .init_resource::<Spawner>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_movement, ai_movement, apply_moves).chain(),
        )
        .add_systems(Update, spawn_flies)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            // mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            mesh: meshes
                .add(shape::RegularPolygon::new(10.0, 3).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            transform: Transform::from_translation(Vec3::new(
                rng.gen_range(-300.0..300.0),
                rng.gen_range(-300.0..300.0),
                0.,
            )),
            ..default()
        },
        Velocity { x: 0.0, y: 0.0 },
        Speed { x: 666.0, y: 600.0 },
        PlayerControlled {
            bounce_timer: Timer::from_seconds(0.1, TimerMode::Once),
        },
    ));
}

fn spawn_flies(
    time: Res<Time>,
    window: Query<&Window>,
    ai_query: Query<&AIControlled>,
    mut spawner: ResMut<Spawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let screen = Screen::from(window.single());

    spawner.0.tick(time.delta());

    if ai_query.iter().count() >= MAX_FLIES {
        return;
    }

    if spawner.0.just_finished() {
        for _ in 0..3 {
            let mut x: f32 = rng.gen_range(-1500.0..1500.0);
            let mut y: f32 = rng.gen_range(-1500.0..1500.0);
            while screen.contains(&Vec3::from([x, y, 0.0])) {
                x = rng.gen_range(-1500.0..1500.0);
                y = rng.gen_range(-1500.0..1500.0);
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
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Speed, &mut PlayerControlled)>,
) {
    for (mut velocity, speed, mut player) in query.iter_mut() {
        for key in input.get_just_pressed() {
            match key {
                KeyCode::Up => velocity.y = speed.y,
                KeyCode::Down => velocity.y = -speed.y,
                KeyCode::Right => velocity.x = speed.x,
                KeyCode::Left => velocity.x = -speed.x,
                _ => continue,
            }
        }

        // when players hit the screen edge they bounce off, but if they're still holding the
        // button down after the bounce_timer finishes, we need to reset the velocity according to
        // the held down button.
        player.bounce_timer.tick(time.delta());
        if player.bounce_timer.just_finished() {
            for key in input.get_pressed() {
                match key {
                    KeyCode::Up => velocity.y = speed.y,
                    KeyCode::Down => velocity.y = -speed.y,
                    KeyCode::Right => velocity.x = speed.x,
                    KeyCode::Left => velocity.x = -speed.x,
                    _ => continue,
                }
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
    mut query: Query<(&mut Transform, &mut Velocity, Entity)>,
    mut players: Query<&mut PlayerControlled>,
) {
    let win = window.single();
    let screen = Screen::from(win);
    for (mut transform, mut velocity, entity) in query.iter_mut() {
        let mut position = &mut transform.translation;

        let was_onscreen = screen.contains(&position);

        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();

        // things that are offscreen before the movement or onscreen after the movement don't need
        // to be contained.
        if !was_onscreen || screen.contains(&position) {
            continue;
        }

        // bounce off the screen edges
        let mut bounced = false;
        position.x = position.x.clamp(screen.min_x, screen.max_x);
        if position.x == screen.min_x {
            position.x = screen.min_x + 1.0;
            velocity.x = -velocity.x;
            bounced = true;
        } else if position.x == screen.max_x {
            position.x = screen.max_x - 1.0;
            velocity.x = -velocity.x;
            bounced = true;
        }

        position.y = position.y.clamp(screen.min_y, screen.max_y);
        if position.y == screen.min_y {
            position.y = screen.min_y + 1.0;
            velocity.y = -velocity.y;
            bounced = true;
        } else if position.y == screen.max_y {
            position.y = screen.max_y - 1.0;
            velocity.y = -velocity.y;
            bounced = true;
        }

        if bounced {
            if let Ok(mut player) = players.get_mut(entity) {
                player.bounce_timer.reset();
                player.bounce_timer.unpause();
            }
        }
    }
}

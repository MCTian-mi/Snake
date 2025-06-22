use bevy::prelude::*;
use rand::Rng;
use std::mem::swap;

// Constants
const OBJECT_SIZE: f32 = 32.0;
const WINDOW_WIDTH: f32 = 16.0 * OBJECT_SIZE;
const WINDOW_HEIGHT: f32 = 16.0 * OBJECT_SIZE;
const SNAKE_SPEED: f32 = OBJECT_SIZE * 1.;

const TITLE: &str = "Snake";

/// Structs and Components
#[derive(Clone, PartialEq, Eq, Default, Debug)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<&KeyCode> for Direction {
    type Error = ();

    fn try_from(key: &KeyCode) -> std::result::Result<Self, Self::Error> {
        match key {
            KeyCode::ArrowUp | KeyCode::KeyW => Ok(Direction::Up),
            KeyCode::ArrowDown | KeyCode::KeyS => Ok(Direction::Down),
            KeyCode::ArrowLeft | KeyCode::KeyA => Ok(Direction::Left),
            KeyCode::ArrowRight | KeyCode::KeyD => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

impl Direction {
    fn is_opposite(&self, other: &Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

impl SnakeHead {
    fn turn(&mut self, new_direction: Direction) {
        if !self.direction.is_opposite(&new_direction) {
            self.turn_unchecked(new_direction)
        }
    }

    fn turn_unchecked(&mut self, new_direction: Direction) {
        self.direction = new_direction;
    }
}

#[derive(Component)]
struct SnakeBody;

#[derive(Component)]
struct Orb;

trait Moveable {
    fn step(&mut self);
    fn clone_translation(&self) -> Vec3;
}

impl Moveable for (Mut<'_, Transform>, Mut<'_, SnakeHead>) {
    fn step(&mut self) {
        let Vec3 { x, y, z: _ } = &mut self.0.translation;
        let direction = &self.1.direction;

        match direction {
            Direction::Up => {
                *y += SNAKE_SPEED;
                if *y > WINDOW_HEIGHT / 2. {
                    *y -= WINDOW_HEIGHT;
                }
            }
            Direction::Down => {
                *y -= SNAKE_SPEED;
                if *y < -WINDOW_HEIGHT / 2. {
                    *y += WINDOW_HEIGHT;
                }
            }
            Direction::Left => {
                *x -= SNAKE_SPEED;
                if *x < -WINDOW_WIDTH / 2. {
                    *x += WINDOW_WIDTH;
                }
            }
            Direction::Right => {
                *x += SNAKE_SPEED;
                if *x > WINDOW_WIDTH / 2. {
                    *x -= WINDOW_WIDTH;
                }
            }
        }
    }

    fn clone_translation(&self) -> Vec3 {
        self.0.translation.clone()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: TITLE.to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (move_snake, check_collisions))
        .insert_resource(Time::<Fixed>::from_seconds(0.25))
        .add_systems(Update, handle_input)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // Spawns snake head
    commands.spawn((
        Sprite::from_image(asset_server.load("textures/head.png")),
        SnakeHead {
            direction: Direction::Up,
        },
    ));

    // Spawns snake body
    spawn_snake_body(
        &mut commands,
        &asset_server,
        Vec3::new(0., -OBJECT_SIZE, 0.),
    );

    // Spawns orb
    commands.spawn((
        Sprite::from_image(asset_server.load("textures/orb.png")),
        Transform::from_translation(get_random_position()),
        Orb,
    ));
}

fn check_collisions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut orb_query: Query<&mut Transform, (With<Orb>, Without<SnakeHead>, Without<SnakeBody>)>,
    mut snake_head_query: Query<(&mut Transform, &mut SnakeHead, &mut Sprite)>,
    snake_body_query: Query<(&mut Transform, Entity), (With<SnakeBody>, Without<SnakeHead>)>,
) {
    let orb_translation = &orb_query.single_mut().unwrap().translation;
    let mut snake_head_moveable = snake_head_query.single_mut().unwrap();
    let snake_head_translation = &snake_head_moveable.0.translation;

    let mut crashed = false;
    for (body_transform, _) in &snake_body_query {
        if snake_head_translation == &body_transform.translation {
            crashed = true;
        }
    }

    if crashed {
        snake_body_query.iter().for_each(|(_, body)| {
            commands.entity(body).despawn();
        });
        snake_head_moveable.2.image = asset_server.load("textures/head_2.png");
    }

    if orb_translation == snake_head_translation {
        let orb_position = get_random_position();

        orb_query.single_mut().unwrap().translation = orb_position;

        spawn_snake_body(&mut commands, &asset_server, *snake_head_translation);
    }
}

fn spawn_snake_body(commands: &mut Commands, asset_server: &Res<AssetServer>, translation: Vec3) {
    commands.spawn((
        Sprite::from_image(asset_server.load("textures/body.png")),
        Transform::from_translation(translation),
        SnakeBody,
    ));
}

fn move_snake(
    mut snake_head_query: Query<(&mut Transform, &mut SnakeHead)>,
    mut snake_body_query: Query<&mut Transform, (With<SnakeBody>, Without<SnakeHead>)>,
) {
    let mut snake_moveable = snake_head_query.single_mut().unwrap();

    let mut last_translation = snake_moveable.clone_translation();

    snake_moveable.step();

    snake_body_query.iter_mut().for_each(|mut body_transform| {
        swap(&mut body_transform.translation, &mut last_translation);
    });
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut snake_head_query: Query<(&mut Transform, &mut SnakeHead)>,
) {
    keyboard_input.get_pressed().for_each(|key| {
        snake_head_query.iter_mut().for_each(|(_, mut snake_head)| {
            if let Ok(direction) = Direction::try_from(key) {
                snake_head.turn(direction);
            }
        });
    });
}

fn get_random_position() -> Vec3 {
    let mut rng = rand::rng();
    let x = (rng.random_range(0..(WINDOW_WIDTH / OBJECT_SIZE) as i32)
        - (((WINDOW_WIDTH / OBJECT_SIZE) as i32) / 2)) as f32
        * OBJECT_SIZE;
    let y = (rng.random_range(0..(WINDOW_HEIGHT / OBJECT_SIZE) as i32)
        - (((WINDOW_HEIGHT / OBJECT_SIZE) as i32) / 2)) as f32
        * OBJECT_SIZE;
    Vec3::new(x, y, 0.)
}

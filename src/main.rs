use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer, window::PrimaryWindow};
use rand::prelude::random;

const SNAKE_HEAD_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const FOOD_COLOR: Color = Color::srgb(1., 0., 1.);
const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(Resource, Default)]
struct LastTailPosition(Option<Position>);

#[derive(Event)]
struct GameOverEvent;

#[derive(Event)]
struct SpawnEvent;

#[derive(Event)]
struct GrowthEvent;

#[derive(Component)]
struct SnakeSegment;

#[derive(Resource, Default)]
struct SnakeSegments(Vec<Entity>);

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Component)]
struct Food;

fn spawn_food(
    mut growth_reader: EventReader<GrowthEvent>,
    mut spawn_reader: EventReader<SpawnEvent>,
    mut commands: Commands,
) {
    if spawn_reader.read().next().is_some() || growth_reader.read().next().is_some() {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: FOOD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(Food)
            .insert(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .insert(Size::square(0.8));
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (500., 500.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_systems(
            Startup,
            (setup_camera, emit_spawn_signal, spawn_snake).chain(),
        )
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .add_systems(
            FixedUpdate,
            (snake_movement.run_if(on_timer(Duration::from_secs_f32(0.090))),),
        )
        .add_systems(
            Update,
            (
                snake_movement_input.before(snake_movement),
                snake_eating.after(snake_movement),
                snake_growth.after(snake_eating),
                spawn_food,
                game_over.after(snake_movement)
            ),
        )
        .add_event::<GrowthEvent>()
        .add_event::<SpawnEvent>()
        .add_event::<GameOverEvent>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn emit_spawn_signal(mut growth_writer: EventWriter<SpawnEvent>) {
    growth_writer.send(SpawnEvent);
}

fn spawn_snake(
    mut spawn_reader: EventReader<SpawnEvent>,
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
) {
    if spawn_reader.read().next().is_some() {
        *segments = SnakeSegments(vec![
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: SNAKE_HEAD_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(10., 10., 10.),
                        ..default()
                    },
                    ..default()
                })
                .insert(SnakeHead {
                    direction: Direction::Up,
                })
                .insert(SnakeSegment)
                .insert(Position { x: 3, y: 3 })
                .insert(Size::square(0.8))
                .id(),
            spawn_segment(commands, Position { x: 3, y: 2 }),
        ])
    }
}

fn size_scaling(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Size, &mut Transform)>,
) {
    let window = windows.get_single().unwrap();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Query<&Window>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(position: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        position / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_single().unwrap();
    for (position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(position.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(
                position.y as f32,
                window.height() as f32,
                ARENA_HEIGHT as f32,
            ),
            0.0,
        );
    }
}

fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut head_positions: Query<&mut SnakeHead>,
) {
    if let Some(mut head) = head_positions.iter_mut().next() {
        let direction: Direction =
            if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
                Direction::Up
            } else if keyboard_input.pressed(KeyCode::ArrowDown)
                || keyboard_input.pressed(KeyCode::KeyS)
            {
                Direction::Down
            } else if keyboard_input.pressed(KeyCode::ArrowRight)
                || keyboard_input.pressed(KeyCode::KeyD)
            {
                Direction::Right
            } else if keyboard_input.pressed(KeyCode::ArrowLeft)
                || keyboard_input.pressed(KeyCode::KeyA)
            {
                Direction::Left
            } else {
                head.direction
            };
        if direction != head.direction.opposite() {
            head.direction = direction;
        }
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
        let mut head_position = positions.get_mut(head_entity).unwrap();
        match head.direction {
            Direction::Up => head_position.y += 1,
            Direction::Down => head_position.y -= 1,
            Direction::Right => head_position.x += 1,
            Direction::Left => head_position.x -= 1,
        }
        if head_position.x < 0
            || head_position.y < 0
            || head_position.x as u32 >= ARENA_WIDTH
            || head_position.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }
        if segment_positions.contains(&head_position) {
            game_over_writer.send(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(position, segment)| *positions.get_mut(*segment).unwrap() = *position);
    }
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    head_positions.iter().for_each(|head_position| {
        food_positions.iter().for_each(|(entity, food_position)| {
            if food_position == head_position {
                commands.entity(entity).despawn();
                growth_writer.send(GrowthEvent);
            }
        })
    });
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.read().next().is_some() {
        segments
            .0
            .push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut game_over_reader: EventReader<GameOverEvent>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if game_over_reader.read().next().is_some() {
        for entity in food.iter().chain(segments.iter()) {
            commands.entity(entity).despawn();
        }
        // TODO: show game over screen
    }
}

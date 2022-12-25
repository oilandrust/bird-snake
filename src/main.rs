#[allow(unused)]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use level::{parse_level, Cell, Level, LEVELS};
use std::collections::VecDeque;

mod level;

#[derive(Component)]
struct Snake {
    parts: VecDeque<(IVec2, IVec2)>,
}

impl Snake {
    fn head_position(&self) -> IVec2 {
        self.parts[0].0
    }

    fn len(&self) -> usize {
        self.parts.len()
    }
}

#[derive(Component, Default)]
struct MoveCommand {
    velocity: f32,
    anim_offset: f32,
}

#[derive(Component)]
struct SnakePart(usize);

#[derive(Component)]
struct GravityFall {
    velocity: f32,
    accumulated_distance: f32,
}

const GRID_TO_WORLD_UNIT: f32 = 25.;
const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
const SNAKE_START_VELOCITY: f32 = 1.0;

const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

fn setup_system(mut commands: Commands) {
    let level = parse_level(LEVELS[0]).unwrap();

    // Spawn the snake
    {
        let start_parts = &level.initial_snake;

        for (index, part) in start_parts.iter().enumerate() {
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::GRAY,
                        custom_size: Some(SNAKE_SIZE),
                        ..default()
                    },
                    transform: Transform {
                        translation: to_world(part.0).extend(0.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(SnakePart(index));
        }

        commands.spawn(Snake {
            parts: VecDeque::from(start_parts.clone()),
        });
    }

    // Spawn the ground sprites
    for (cell, position) in level.grid.iter() {
        if cell != Cell::Wall {
            continue;
        }

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(GRID_CELL_SIZE),
                ..default()
            },
            transform: Transform {
                translation: to_world(position).extend(0.0),
                ..default()
            },
            ..default()
        });
    }

    // Spawn level goal sprite.
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::LIME_GREEN,
            custom_size: Some(GRID_CELL_SIZE),
            ..default()
        },
        transform: Transform {
            translation: to_world(level.goal_position).extend(0.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            level.grid.width as f32 * GRID_TO_WORLD_UNIT * 0.5,
            level.grid.height as f32 * GRID_TO_WORLD_UNIT * 0.5,
            0.0,
        ),
        ..default()
    });

    commands.insert_resource(level);
}

fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * GRID_TO_WORLD_UNIT
}

fn snake_movement_control_system(
    keyboard: Res<Input<KeyCode>>,
    level: Res<Level>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake), Without<MoveCommand>>,
) {
    let Ok((snake_entity, mut snake)) = query.get_single_mut() else {
        return;
    };

    // TODO: Use last pressed instead of any pressed.
    let new_direction = if keyboard.any_pressed(MOVE_UP_KEYS) {
        Some(IVec2::Y)
    } else if keyboard.any_pressed(MOVE_LEFT_KEYS) {
        Some(IVec2::NEG_X)
    } else if keyboard.any_pressed(MOVE_DOWN_KEYS) {
        Some(IVec2::NEG_Y)
    } else if keyboard.any_pressed(MOVE_RIGHT_KEYS) {
        Some(IVec2::X)
    } else {
        None
    };

    let Some(direction) = new_direction else {
        return;
    };

    // Check that we have enough parts to go up.
    if direction == IVec2::Y {
        let distance_to_ground = level.get_distance_to_ground(snake.head_position());
        if distance_to_ground >= snake.len() as i32 {
            return;
        }
    }

    let new_position = snake.parts[0].0 + direction;

    // Check for collition with self.
    if snake.parts.iter().any(|part| part.0 == new_position) || !level.grid.is_empty(new_position) {
        return;
    }

    // Finaly move the snake forward.
    snake.parts.push_front((new_position, direction));
    snake.parts.pop_back();

    // Smooth move animation starts.
    commands.entity(snake_entity).insert(MoveCommand {
        velocity: SNAKE_START_VELOCITY,
        anim_offset: GRID_TO_WORLD_UNIT,
    });

    // Check if snake is on the ground and spawn gravity fall if not.
    let on_the_ground = !snake
        .parts
        .iter()
        .all(|(position, _)| level.get_distance_to_ground(*position) > 1);

    if !on_the_ground {
        commands.entity(snake_entity).insert(GravityFall {
            velocity: SNAKE_START_VELOCITY,
            accumulated_distance: 0.0,
        });

        for (position, _) in snake.parts.iter_mut() {
            *position += IVec2::NEG_Y;
        }
    }
}

fn gravity_system(mut commands: Commands, mut query: Query<(Entity, &mut GravityFall)>) {
    let Ok((snake_entity, mut gravity_fall)) = query.get_single_mut() else {
        return;
    };

    gravity_fall.accumulated_distance += gravity_fall.velocity;
    if gravity_fall.accumulated_distance > GRID_TO_WORLD_UNIT {
        commands.entity(snake_entity).remove::<GravityFall>();
    }
}

fn snake_smooth_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MoveCommand)>,
) {
    let Ok((entity, mut move_command)) = query.get_single_mut() else {
        return;
    };

    move_command.anim_offset -= move_command.velocity;
    if move_command.anim_offset < 0.0 {
        commands.entity(entity).remove::<MoveCommand>();
    }
}

type WithMoveOrGravity = Or<(With<MoveCommand>, With<GravityFall>)>;

fn update_sprite_positions_system(
    snake_query: Query<(&Snake, Option<&MoveCommand>, Option<&GravityFall>), WithMoveOrGravity>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite, &SnakePart)>,
) {
    let Ok((snake, move_command, gravity_fall)) = snake_query.get_single() else {
        return;
    };

    for (mut transform, mut sprite, part) in sprite_query.iter_mut() {
        let direction = snake.parts[part.0].1;
        let mut part_position = to_world(snake.parts[part.0].0);

        // Move sprite with move anim.
        if let Some(move_command) = move_command {
            part_position -= move_command.anim_offset * direction.as_vec2();

            // Extend sprites at a turn to cover the gaps. Reset normal size otherwize.
            if part.0 < snake.parts.len() - 1 && direction != snake.parts[part.0 + 1].1 {
                let size_offset =
                    direction.as_vec2() * (GRID_TO_WORLD_UNIT - move_command.anim_offset);
                sprite.custom_size = Some(SNAKE_SIZE + size_offset.abs());
                part_position -= size_offset * 0.5;
            } else {
                sprite.custom_size = Some(SNAKE_SIZE);
            }
        }

        // Move sprite with gravity fall anim.
        if let Some(gravity_fall) = gravity_fall {
            part_position += (GRID_TO_WORLD_UNIT - gravity_fall.accumulated_distance) * Vec2::Y;
        }

        transform.translation.x = part_position.x;
        transform.translation.y = part_position.y;
    }
}

fn debug_draw_grid_system(mut lines: ResMut<DebugLines>) {
    for j in -10..=10 {
        let y = j as f32 * GRID_TO_WORLD_UNIT;
        let start = Vec3::new(-10. * GRID_TO_WORLD_UNIT, y, 0.);
        let end = Vec3::new(10. * GRID_TO_WORLD_UNIT, y, 0.);
        lines.line_colored(
            start,
            end,
            0.,
            if j == 0 { Color::RED } else { Color::BLACK },
        );
    }

    for i in -10..=10 {
        let x = i as f32 * GRID_TO_WORLD_UNIT;
        let start = Vec3::new(x, -10. * GRID_TO_WORLD_UNIT, 0.);
        let end = Vec3::new(x, 10. * GRID_TO_WORLD_UNIT, 0.);
        lines.line_colored(
            start,
            end,
            0.,
            if i == 0 { Color::RED } else { Color::BLACK },
        );
    }
}

fn debug_draw_snake_system(mut lines: ResMut<DebugLines>, query: Query<&Snake>) {
    let Ok(snake) = query.get_single() else {
        return;
    };

    let grid = snake.parts[0].0;
    let world_grid = to_world(grid);
    let world_grid = Vec3::new(world_grid.x, world_grid.y, 1.0);

    lines.line_colored(
        world_grid + Vec3::new(5.0, 5.0, 1.0),
        world_grid + Vec3::new(-5.0, -5.0, 1.0),
        0.,
        Color::BLUE,
    );

    lines.line_colored(
        world_grid + Vec3::new(-5.0, 5.0, 1.0),
        world_grid + Vec3::new(5.0, -5.0, 1.0),
        0.,
        Color::BLUE,
    );
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BEIGE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Snake".to_string(),
                width: 640.0,
                height: 420.0,
                ..default()
            },
            ..default()
        }))
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup_system)
        .add_system(bevy::window::close_on_esc)
        .add_system(snake_movement_control_system)
        .add_system(gravity_system.after(snake_movement_control_system))
        .add_system(snake_smooth_movement_system.after(gravity_system))
        .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system)
        // .add_system_to_stage(CoreStage::Last, debug_draw_grid_system)
        // .add_system_to_stage(CoreStage::Last, debug_draw_snake_system)
        .run();
}

use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use std::collections::VecDeque;

#[derive(Component)]
struct Snake {
    direction: IVec2,
    velocity: f32,
    move_offset: f32,
    parts: VecDeque<(IVec2, IVec2)>,
}

#[derive(Component)]
struct DirectionChange {
    new_direction: IVec2,
}

#[derive(Component)]
struct SnakePart(usize);

const SNAKE_WIDTH: f32 = 13.;
const SNAKE_START_VELOCITY: f32 = 1.0;

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });

    let start_parts: Vec<(IVec2, IVec2)> = (0..6).map(|i| (IVec2::new(-i, 0), IVec2::X)).collect();

    for (index, part) in start_parts.iter().enumerate() {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(SNAKE_WIDTH, SNAKE_WIDTH)),
                    ..default()
                },
                transform: Transform {
                    translation: part.0.as_vec2().extend(0.0),
                    ..default()
                },
                ..default()
            })
            .insert(SnakePart(index));
    }

    commands.spawn(Snake {
        direction: IVec2::X,
        velocity: SNAKE_START_VELOCITY,
        move_offset: 0.0,
        parts: VecDeque::from(start_parts),
    });
}

fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * SNAKE_WIDTH
}

fn snake_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake, Option<&DirectionChange>)>,
) {
    let Ok((entity, mut snake, direction_change)) = query.get_single_mut() else {
        return;
    };

    // parts
    // velocity
    // move_offset
    // direction

    let head_position = snake.parts[0].0;
    let mut new_offset = snake.move_offset - snake.velocity;

    if new_offset < 0.0 {
        if let Some(change) = direction_change {
            snake.direction = change.new_direction;
            commands.entity(entity).remove::<DirectionChange>();
        }
        let new_position = head_position + snake.direction;
        let part_direction = snake.direction;
        snake.parts.push_front((new_position, part_direction));
        snake.parts.pop_back();
        new_offset = SNAKE_WIDTH;
    }

    snake.move_offset = new_offset;
}

fn update_sprite_positions_system(
    snake_query: Query<&Snake>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite, &SnakePart)>,
) {
    let Ok(snake) = snake_query.get_single() else {
        return;
    };

    // parts
    // move_offset

    for (mut transform, mut sprite, part) in sprite_query.iter_mut() {
        let direction = snake.parts[part.0].1;
        let mut part_position =
            to_world(snake.parts[part.0].0) - snake.move_offset * direction.as_vec2();

        if part.0 < snake.parts.len() - 1 && direction != snake.parts[part.0 + 1].1 {
            // Extend sprites at a turn to cover the gaps.
            let size_offset = direction.as_vec2() * (SNAKE_WIDTH - snake.move_offset);
            sprite.custom_size = Some(Vec2::new(SNAKE_WIDTH, SNAKE_WIDTH) + size_offset.abs());
            part_position -= size_offset * 0.5;
        } else {
            sprite.custom_size = Some(Vec2::new(SNAKE_WIDTH, SNAKE_WIDTH));
        }

        transform.translation.x = part_position.x;
        transform.translation.y = part_position.y;
    }
}

fn keyboard_control_system(
    keyboard: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, &Snake)>,
) {
    let Ok((snake_entity, snake)) = query.get_single_mut() else {
        return;
    };

    // direction

    const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
    const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
    const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
    const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

    // TODO: handle multiple just pressed.
    let new_direction = if keyboard.any_just_pressed(MOVE_UP_KEYS) {
        Some(IVec2::Y)
    } else if keyboard.any_just_pressed(MOVE_LEFT_KEYS) {
        Some(IVec2::NEG_X)
    } else if keyboard.any_just_pressed(MOVE_DOWN_KEYS) {
        Some(IVec2::NEG_Y)
    } else if keyboard.any_just_pressed(MOVE_RIGHT_KEYS) {
        Some(IVec2::X)
    } else {
        None
    };

    if let Some(direction) = new_direction {
        if direction != -snake.direction {
            commands
                .entity(snake_entity)
                .remove::<DirectionChange>()
                .insert(DirectionChange {
                    new_direction: direction,
                });
        }
    }
}

fn debug_draw_grid_system(mut lines: ResMut<DebugLines>) {
    for j in -10..=10 {
        let y = j as f32 * SNAKE_WIDTH;
        let start = Vec3::new(-10. * SNAKE_WIDTH, y, 0.);
        let end = Vec3::new(10. * SNAKE_WIDTH, y, 0.);
        lines.line_colored(
            start,
            end,
            0.,
            if j == 0 { Color::RED } else { Color::BLACK },
        );
    }

    for i in -10..=10 {
        let x = i as f32 * SNAKE_WIDTH;
        let start = Vec3::new(x, -10. * SNAKE_WIDTH, 0.);
        let end = Vec3::new(x, 10. * SNAKE_WIDTH, 0.);
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
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(setup_system)
        .add_system(bevy::window::close_on_esc)
        .add_system(keyboard_control_system)
        .add_system(snake_movement_system.after(keyboard_control_system))
        .add_system(update_sprite_positions_system.after(snake_movement_system))
        .add_system(debug_draw_grid_system)
        .add_system(debug_draw_snake_system)
        .run();
}

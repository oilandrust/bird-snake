#[allow(unused)]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use level::{Cell, Level, LEVELS};
use std::collections::VecDeque;

use bevy_egui::{egui, EguiContext, EguiPlugin};

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

    fn is_standing(&self) -> bool {
        (self.parts.front().unwrap().0.y - self.parts.back().unwrap().0.y)
            == (self.len() - 1) as i32
    }

    fn occupies_position(&self, position: IVec2) -> bool {
        self.parts.iter().any(|part| part.0 == position)
    }

    fn fall_one_unit(&mut self) {
        for (position, _) in self.parts.iter_mut() {
            *position += IVec2::NEG_Y;
        }
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
    relative_y: f32,
}

mod game_constants {
    use bevy::prelude::*;

    pub const GRID_TO_WORLD_UNIT: f32 = 25.;
    pub const SNAKE_SIZE: Vec2 = Vec2::splat(GRID_TO_WORLD_UNIT);
    pub const GRID_CELL_SIZE: Vec2 = SNAKE_SIZE;
    pub const MOVE_START_VELOCITY: f32 = 4.0;
    pub const JUMP_START_VELOCITY: f32 = 65.0;
    pub const GRAVITY: f32 = 300.0;
}

pub const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
pub const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
pub const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
pub const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

#[derive(Resource)]
struct GameConstants {
    move_velocity: f32,
    jump_velocity: f32,
    gravity: f32,
}

fn setup_system(mut commands: Commands) {
    let level = Level::parse(LEVELS[0]).unwrap();

    // Spawn the snake
    {
        let start_parts = &level.initial_snake;

        for (index, part) in start_parts.iter().enumerate() {
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::GRAY,
                        custom_size: Some(game_constants::SNAKE_SIZE),
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
    for (position, cell) in level.grid.iter() {
        if cell != Cell::Wall {
            continue;
        }

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(game_constants::GRID_CELL_SIZE),
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
            custom_size: Some(game_constants::GRID_CELL_SIZE),
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
            level.grid.width() as f32 * game_constants::GRID_TO_WORLD_UNIT * 0.5,
            level.grid.height() as f32 * game_constants::GRID_TO_WORLD_UNIT * 0.5,
            0.0,
        ),
        ..default()
    });

    commands.insert_resource(level);
    commands.insert_resource(GameConstants {
        move_velocity: game_constants::MOVE_START_VELOCITY,
        jump_velocity: game_constants::JUMP_START_VELOCITY,
        gravity: game_constants::GRAVITY,
    })
}

fn to_world(position: IVec2) -> Vec2 {
    (position.as_vec2() + 0.5) * game_constants::GRID_TO_WORLD_UNIT
}

fn min_distance_to_ground(level: &Level, snake: &Snake) -> i32 {
    snake
        .parts
        .iter()
        .map(|(position, _)| level.get_distance_to_ground(*position))
        .min()
        .unwrap()
}

type WithoutMoveOrFall = (Without<MoveCommand>, Without<GravityFall>);

fn snake_movement_control_system(
    keyboard: Res<Input<KeyCode>>,
    level: Res<Level>,
    constants: Res<GameConstants>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake), WithoutMoveOrFall>,
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
    // TODO_MAYBE: This could be done with a move up then fall.
    if direction == IVec2::Y && snake.is_standing() {
        commands.entity(snake_entity).insert(GravityFall {
            velocity: constants.jump_velocity,
            relative_y: 0.0,
        });
        return;
    }

    let new_position = snake.parts[0].0 + direction;

    // Check for collition with self.
    if snake.occupies_position(new_position) || !level.grid.is_empty(new_position) {
        return;
    }

    // Finaly move the snake forward.
    snake.parts.push_front((new_position, direction));
    snake.parts.pop_back();

    // Smooth move animation starts.
    commands.entity(snake_entity).insert(MoveCommand {
        velocity: constants.move_velocity,
        anim_offset: game_constants::GRID_TO_WORLD_UNIT,
    });
}

fn gravity_system(
    time: Res<Time>,
    constants: Res<GameConstants>,
    level: Res<Level>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake, Option<&mut GravityFall>)>,
) {
    let Ok((snake_entity, mut snake, gravity_fall)) = query.get_single_mut() else {
        return;
    };

    match gravity_fall {
        Some(mut gravity_fall) => {
            gravity_fall.velocity -= constants.gravity * time.delta_seconds();
            gravity_fall.relative_y += gravity_fall.velocity * time.delta_seconds();

            // When relative y is 0, the sprites are aligned with the actual position.
            if gravity_fall.relative_y < 0.0 {
                // keep falling..
                if min_distance_to_ground(&level, &snake) > 1 {
                    gravity_fall.relative_y = game_constants::GRID_TO_WORLD_UNIT;

                    snake.fall_one_unit();
                } else {
                    // ..or stop falling animation.
                    commands.entity(snake_entity).remove::<GravityFall>();
                }
            }
        }
        None => {
            // Check if snake is on the ground and spawn gravity fall if not.
            if min_distance_to_ground(&level, &snake) > 1 {
                commands.entity(snake_entity).insert(GravityFall {
                    velocity: 0.0,
                    relative_y: game_constants::GRID_TO_WORLD_UNIT,
                });

                snake.fall_one_unit();
            }
        }
    }
}

fn snake_smooth_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MoveCommand)>,
) {
    let Ok((entity, mut move_command)) = query.get_single_mut() else {
        return;
    };

    move_command.anim_offset -= move_command.velocity + time.delta_seconds();
    if move_command.anim_offset < 0.0 {
        commands.entity(entity).remove::<MoveCommand>();
    }
}

fn update_sprite_positions_system(
    snake_query: Query<(&Snake, Option<&MoveCommand>, Option<&GravityFall>)>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite, &SnakePart)>,
) {
    let Ok((snake, move_command, gravity_fall)) = snake_query.get_single() else {
        return;
    };

    for (mut transform, mut sprite, part) in sprite_query.iter_mut() {
        let mut part_position = to_world(snake.parts[part.0].0);

        // Move sprite with move anim.
        if let Some(move_command) = move_command {
            let direction = snake.parts[part.0].1;
            part_position -= move_command.anim_offset * direction.as_vec2();

            // Extend sprites at a turn to cover the gaps. Reset normal size otherwize.
            if part.0 < snake.parts.len() - 1 && direction != snake.parts[part.0 + 1].1 {
                let size_offset = direction.as_vec2()
                    * (game_constants::GRID_TO_WORLD_UNIT - move_command.anim_offset);
                sprite.custom_size = Some(game_constants::SNAKE_SIZE + size_offset.abs());
                part_position -= size_offset * 0.5;
            } else {
                sprite.custom_size = Some(game_constants::SNAKE_SIZE);
            }
        } else {
            sprite.custom_size = Some(game_constants::SNAKE_SIZE);
        }

        // Move sprite with gravity fall anim.
        if let Some(gravity_fall) = gravity_fall {
            part_position += gravity_fall.relative_y * Vec2::Y;
        }

        transform.translation.x = part_position.x;
        transform.translation.y = part_position.y;
    }
}

fn debug_draw_grid_system(level: Res<Level>, mut lines: ResMut<DebugLines>) {
    for j in 0..=level.grid.height() {
        let y = j as f32 * game_constants::GRID_TO_WORLD_UNIT;
        let start = Vec3::new(0., y, 0.);
        let end = Vec3::new(
            level.grid.width() as f32 * game_constants::GRID_TO_WORLD_UNIT,
            y,
            0.,
        );
        lines.line_colored(
            start,
            end,
            0.,
            if j == 0 { Color::RED } else { Color::BLACK },
        );
    }

    for i in 0..=level.grid.width() {
        let x = i as f32 * game_constants::GRID_TO_WORLD_UNIT;
        let start = Vec3::new(x, 0., 0.);
        let end = Vec3::new(
            x,
            level.grid.height() as f32 * game_constants::GRID_TO_WORLD_UNIT,
            0.,
        );
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

    for position in &snake.parts {
        let world_grid = to_world(position.0);
        let world_grid = Vec3::new(world_grid.x, world_grid.y, 0.0);

        lines.line_colored(
            world_grid + Vec3::new(5.0, 5.0, 0.0),
            world_grid + Vec3::new(-5.0, -5.0, 0.0),
            0.,
            Color::BLUE,
        );

        lines.line_colored(
            world_grid + Vec3::new(-5.0, 5.0, 0.0),
            world_grid + Vec3::new(5.0, -5.0, 0.0),
            0.,
            Color::BLUE,
        );
    }
}

fn dev_ui_system(mut egui_context: ResMut<EguiContext>, mut game_constants: ResMut<GameConstants>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut game_constants.move_velocity, 0.0..=10.0).text("Move Velocity"),
        );
        ui.add(
            egui::Slider::new(&mut game_constants.jump_velocity, 0.0..=300.0).text("Jump Velocity"),
        );
        ui.add(egui::Slider::new(&mut game_constants.gravity, 0.0..=900.0).text("Gravity"));
    });
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
        .add_plugin(EguiPlugin)
        .add_startup_system(setup_system)
        .add_system(bevy::window::close_on_esc)
        .add_system(snake_movement_control_system)
        .add_system(gravity_system.after(snake_movement_control_system))
        .add_system(snake_smooth_movement_system.after(gravity_system))
        .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system)
        .add_system_to_stage(CoreStage::Last, debug_draw_grid_system)
        .add_system_to_stage(CoreStage::Last, debug_draw_snake_system)
        .add_system(dev_ui_system)
        .run();
}

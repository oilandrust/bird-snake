#[allow(unused)]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPlugin;
use dev_tools::DevToolsPlugin;
use game_constants::*;
use level::{Cell, Level, LEVELS};
use movement::*;
use snake::*;

mod dev_tools;
mod game_constants;
mod level;
mod movement;
mod snake;

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

        commands.spawn(Snake::from_parts(start_parts.clone()));
    }

    // Spawn the ground sprites
    for (position, cell) in level.grid.iter() {
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
            level.grid.width() as f32 * GRID_TO_WORLD_UNIT * 0.5,
            level.grid.height() as f32 * GRID_TO_WORLD_UNIT * 0.5,
            0.0,
        ),
        ..default()
    });

    commands.insert_resource(level);
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
        .add_plugin(EguiPlugin)
        .add_plugin(GameConstantsPlugin)
        .add_plugin(DevToolsPlugin)
        .add_startup_system(setup_system)
        .add_system(bevy::window::close_on_esc)
        .add_system(snake_movement_control_system)
        .add_system(gravity_system.after(snake_movement_control_system))
        .add_system(snake_smooth_movement_system.after(gravity_system))
        .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system)
        .run();
}

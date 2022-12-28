use bevy::{app::AppExit, prelude::*};

use crate::{
    game_constants_pluggin::{to_world, GRID_CELL_SIZE, GRID_TO_WORLD_UNIT},
    level::{Cell, Level, LEVELS},
    movement_pluggin::snake_movement_control_system,
    snake::{spawn_snake_system, Snake, SpawnSnakeEvent},
};

pub struct StartLevelEvent(pub usize);
pub struct ClearLevelEvent;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct Food(pub IVec2);

#[derive(Resource)]
pub struct CurrentLevelId(usize);

pub struct LevelPluggin;

static LOAD_LEVEL_STAGE: &str = "LoadLevelStage";

impl Plugin for LevelPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartLevelEvent>()
            .add_event::<ClearLevelEvent>()
            .add_stage_before(
                CoreStage::PreUpdate,
                LOAD_LEVEL_STAGE,
                SystemStage::single_threaded(),
            )
            .add_system_to_stage(LOAD_LEVEL_STAGE, load_level_system)
            .add_system_to_stage(CoreStage::PreUpdate, spawn_level_entities_system)
            .add_system_to_stage(CoreStage::PreUpdate, spawn_snake_system)
            .add_system(check_for_level_completion_system.after(snake_movement_control_system))
            .add_system_to_stage(CoreStage::Last, clear_level_system);
    }
}

fn load_level_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEvent>,
    mut spawn_snake_event: EventWriter<SpawnSnakeEvent>,
) {
    let Some(event) = event_start_level.iter().next() else {
        return;
    };

    let next_level_index = event.0;
    let level = Level::parse(LEVELS[next_level_index]).unwrap();

    commands.insert_resource(level);
    commands.insert_resource(CurrentLevelId(next_level_index));

    spawn_snake_event.send(SpawnSnakeEvent);
}

fn spawn_level_entities_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEvent>,
    level: Res<Level>,
) {
    if event_start_level.iter().next().is_none() {
        return;
    }

    // Spawn the ground sprites
    for (position, cell) in level.grid.iter() {
        if cell != Cell::Wall {
            continue;
        }

        commands
            .spawn(SpriteBundle {
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
            })
            .insert(LevelEntity);
    }

    // Spawn the food sprites.
    for position in &level.food_positions {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(GRID_CELL_SIZE),
                    ..default()
                },
                transform: Transform {
                    translation: to_world(*position).extend(0.0),
                    ..default()
                },
                ..default()
            })
            .insert(Food(*position))
            .insert(LevelEntity);
    }

    // Spawn level goal sprite.
    commands
        .spawn(SpriteBundle {
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
        })
        .insert(LevelEntity);

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(
                level.grid.width() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                level.grid.height() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                0.0,
            ),
            ..default()
        })
        .insert(LevelEntity);
}

pub fn clear_level_system(
    mut event_clear_level: EventReader<ClearLevelEvent>,
    mut commands: Commands,
    query: Query<Entity, With<LevelEntity>>,
) {
    if event_clear_level.iter().next().is_none() {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn check_for_level_completion_system(
    level: Res<Level>,
    level_id: Res<CurrentLevelId>,
    mut event_start_level: EventWriter<StartLevelEvent>,
    mut event_clear_level: EventWriter<ClearLevelEvent>,
    mut exit: EventWriter<AppExit>,
    mut query: Query<&Snake>,
) {
    let Ok(snake) = query.get_single_mut() else {
        return;
    };

    if level.goal_position != snake.head_position() {
        return;
    }

    if level_id.0 == LEVELS.len() - 1 {
        exit.send(AppExit);
    } else {
        event_clear_level.send(ClearLevelEvent);
        event_start_level.send(StartLevelEvent(level_id.0 + 1));
    }
}

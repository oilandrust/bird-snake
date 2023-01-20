use bevy::{app::AppExit, prelude::*};

use crate::{
    commands::SnakeCommands,
    game_constants_pluggin::{
        to_world, BRIGHT_COLOR_PALETTE, DARK_COLOR_PALETTE, GRID_CELL_SIZE, GRID_TO_WORLD_UNIT,
        WALL_COLOR,
    },
    level_instance::{LevelEntityType, LevelInstance},
    level_template::{Cell, LevelTemplate},
    levels::LEVELS,
    movement_pluggin::{GravityFall, SnakeReachGoalEvent},
    snake_pluggin::{Active, DespawnSnakePartsEvent, SelectedSnake, Snake, SpawnSnakeEvent},
    test_levels::TEST_LEVELS,
    undo::SnakeHistory,
};

pub struct StartLevelEventWithIndex(pub usize);
pub struct StartTestLevelEventWithIndex(pub usize);
pub struct StartLevelEventWithLevel(pub String);
pub struct ClearLevelEvent;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Clone, Copy)]
pub struct Food(pub IVec2);

#[derive(Component, Clone, Copy)]
pub struct Spike(pub IVec2);

#[derive(Resource)]
pub struct CurrentLevelId(pub usize);

pub struct LevelPluggin;

pub static LOAD_LEVEL_STAGE: &str = "LoadLevelStage";

impl Plugin for LevelPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartLevelEventWithIndex>()
            .add_event::<StartTestLevelEventWithIndex>()
            .add_event::<StartLevelEventWithLevel>()
            .add_event::<ClearLevelEvent>()
            .add_stage_before(
                CoreStage::PreUpdate,
                LOAD_LEVEL_STAGE,
                SystemStage::single_threaded(),
            )
            .add_system_to_stage(LOAD_LEVEL_STAGE, load_level_with_index_system)
            .add_system_to_stage(LOAD_LEVEL_STAGE, load_test_level_with_index_system)
            .add_system_to_stage(
                LOAD_LEVEL_STAGE,
                load_level_system
                    .after(load_level_with_index_system)
                    .after(load_test_level_with_index_system),
            )
            .add_system_to_stage(CoreStage::PreUpdate, spawn_level_entities_system)
            .add_system_to_stage(CoreStage::PostUpdate, check_for_level_completion_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                snake_exit_level_system.after(check_for_level_completion_system),
            )
            .add_system_to_stage(CoreStage::Last, clear_level_system);
    }
}

fn load_level_with_index_system(
    mut commands: Commands,
    mut event_start_level_with_index: EventReader<StartLevelEventWithIndex>,
    mut event_start_level: EventWriter<StartLevelEventWithLevel>,
) {
    let Some(event) = event_start_level_with_index.iter().next() else {
        return;
    };

    let next_level_index = event.0;
    event_start_level.send(StartLevelEventWithLevel(
        LEVELS[next_level_index].to_owned(),
    ));

    commands.insert_resource(CurrentLevelId(next_level_index));
}

fn load_test_level_with_index_system(
    mut commands: Commands,
    mut event_start_level_with_index: EventReader<StartTestLevelEventWithIndex>,
    mut event_start_level: EventWriter<StartLevelEventWithLevel>,
) {
    let Some(event) = event_start_level_with_index.iter().next() else {
        return;
    };

    let next_level_index = event.0;
    event_start_level.send(StartLevelEventWithLevel(
        TEST_LEVELS[next_level_index].to_owned(),
    ));

    commands.insert_resource(CurrentLevelId(next_level_index));
}

pub fn load_level_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEventWithLevel>,
    mut spawn_snake_event: EventWriter<SpawnSnakeEvent>,
) {
    let Some(event) = event_start_level.iter().next() else {
        return;
    };

    let level = LevelTemplate::parse(&event.0).unwrap();

    commands.insert_resource(SnakeHistory::default());
    commands.insert_resource(level);
    commands.insert_resource(LevelInstance::new());

    spawn_snake_event.send(SpawnSnakeEvent);
}

fn spawn_level_entities_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEventWithLevel>,
    level_template: Res<LevelTemplate>,
    mut level_instance: ResMut<LevelInstance>,
) {
    if event_start_level.iter().next().is_none() {
        return;
    }

    // Spawn the ground sprites
    for (position, cell) in level_template.grid.iter() {
        if cell != Cell::Wall {
            continue;
        }

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: WALL_COLOR,
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

        level_instance.mark_position_occupied(position, LevelEntityType::Wall);
    }

    // Spawn the food sprites.
    for position in &level_template.food_positions {
        spawn_food(&mut commands, position, &mut level_instance);
    }

    // Spawn the spikes sprites.
    for position in &level_template.spike_positions {
        spawn_spike(&mut commands, position, &mut level_instance);
    }

    // Spawn level goal sprite.
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: BRIGHT_COLOR_PALETTE[8],
                custom_size: Some(GRID_CELL_SIZE),
                ..default()
            },
            transform: Transform {
                translation: to_world(level_template.goal_position).extend(0.0),
                ..default()
            },
            ..default()
        })
        .insert(LevelEntity);

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(
                level_template.grid.width() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                level_template.grid.height() as f32 * GRID_TO_WORLD_UNIT * 0.5,
                50.0,
            ),
            ..default()
        })
        .insert(LevelEntity);
}

pub fn spawn_spike(commands: &mut Commands, position: &IVec2, level_instance: &mut LevelInstance) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: DARK_COLOR_PALETTE[3],
                custom_size: Some(0.5 * GRID_CELL_SIZE),
                ..default()
            },
            transform: Transform {
                translation: to_world(*position).extend(0.0),
                ..default()
            },
            ..default()
        })
        .insert(Spike(*position))
        .insert(LevelEntity);

    level_instance.mark_position_occupied(*position, LevelEntityType::Spike);
}

pub fn spawn_food(commands: &mut Commands, position: &IVec2, level_instance: &mut LevelInstance) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: BRIGHT_COLOR_PALETTE[3],
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

    level_instance.mark_position_occupied(*position, LevelEntityType::Food);
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

    commands.remove_resource::<LevelInstance>();
    commands.remove_resource::<SnakeHistory>();
}

pub fn check_for_level_completion_system(
    level: Res<LevelTemplate>,
    mut snake_reach_goal_event: EventWriter<SnakeReachGoalEvent>,
    snakes_query: Query<&Snake, With<Active>>,
) {
    let snake_at_exit = snakes_query
        .iter()
        .find(|snake| level.goal_position == snake.head_position());
    if snake_at_exit.is_none() {
        return;
    }

    snake_reach_goal_event.send(SnakeReachGoalEvent(snake_at_exit.unwrap().index()));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn snake_exit_level_system(
    mut history: ResMut<SnakeHistory>,
    mut level_instance: ResMut<LevelInstance>,
    level_id: Res<CurrentLevelId>,
    mut snake_reach_goal_event: EventReader<SnakeReachGoalEvent>,
    mut event_start_level: EventWriter<StartLevelEventWithIndex>,
    mut event_clear_level: EventWriter<ClearLevelEvent>,
    mut event_despawn_snake_parts: EventWriter<DespawnSnakePartsEvent>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    snakes_query: Query<
        (Entity, &Snake, Option<&GravityFall>, Option<&SelectedSnake>),
        With<Active>,
    >,
) {
    let Some(reach_goal_event) = snake_reach_goal_event.iter().next() else {
        return;
    };

    // If there is only on snake left, exit level.
    if snakes_query.iter().len() == 1 {
        if level_id.0 == LEVELS.len() - 1 {
            exit.send(AppExit);
        } else {
            event_clear_level.send(ClearLevelEvent);
            event_start_level.send(StartLevelEventWithIndex(level_id.0 + 1));
        }
        return;
    }

    let snake_at_exit = snakes_query
        .iter()
        .find(|(_, snake, _, _)| snake.index() == reach_goal_event.0);

    let Some((entity, snake, fall, selected)) = snake_at_exit else {
        return;
    };

    commands
        .entity(entity)
        .remove::<SelectedSnake>()
        .remove::<Active>()
        .remove::<GravityFall>();

    event_despawn_snake_parts.send(DespawnSnakePartsEvent(snake.index()));

    SnakeCommands::new(level_instance.as_mut(), history.as_mut()).exit_level(snake, entity, fall);

    //   Select another snake if the snake was selected.
    if selected.is_some() {
        let other_snake = snakes_query
            .iter()
            .find(|(other_entity, _, _, _)| entity != *other_entity);

        if let Some((next_snake_entity, _, _, _)) = other_snake {
            commands.entity(next_snake_entity).insert(SelectedSnake);
        }
    }
}

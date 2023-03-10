use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use bevy_tweening::{
    component_animator_system, AnimationSystem, Animator, EaseFunction, Lens, Tween,
};
use iyes_loopless::prelude::{ConditionSet, IntoConditionalSystem};
use rand::prelude::*;

use crate::{
    gameplay::commands::SnakeCommands,
    gameplay::game_constants_pluggin::*,
    gameplay::level_pluggin::Food,
    gameplay::snake_pluggin::{
        respawn_snake_on_fall_system, Active, SelectedSnake, Snake, SpawnSnakeEvent,
    },
    gameplay::undo::{keyboard_undo_system, undo_event_system, SnakeHistory, UndoEvent},
    level::{level_instance::LevelInstance, level_template::LevelTemplate},
    Assets, GameState,
};

use super::{
    level_pluggin::Goal,
    snake_pluggin::{DespawnSnakePartEvent, PartClipper, SnakeEye, SnakePart, SnakePartBundle},
};

const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

#[derive(Component, Default)]
pub struct MoveCommand {
    velocity: f32,
    pub lerp_time: f32,
}

#[derive(Component, Default)]
pub struct PushedAnim {
    pub direction: Vec2,
    velocity: f32,
    pub lerp_time: f32,
}

#[derive(Component, Copy, Clone)]
pub struct GravityFall {
    velocity: f32,
    pub relative_y: f32,
    pub grid_distance: i32,
}

#[derive(Component, Clone)]
pub struct LevelExitAnim {
    pub distance_to_move: i32,
    pub initial_snake_position: Vec<(IVec2, IVec2)>,
}

#[derive(Component)]
pub struct PartGrowAnim {
    pub grow_factor: f32,
}

struct GrowPartLens;

impl Lens<PartGrowAnim> for GrowPartLens {
    fn lerp(&mut self, target: &mut PartGrowAnim, ratio: f32) {
        target.grow_factor = ratio;
    }
}

pub struct MovementPluggin;

pub struct MoveCommandEvent(pub IVec2);

pub struct SnakeMovedEvent;

pub struct SnakeReachGoalEvent(pub Entity);

pub struct SnakeExitedLevelEvent;

const KEYBOARD_INPUT: &str = "KEYBOARD_INPUT";
const UNDO: &str = "UNDO";
const SNAKE_MOVEMENT: &str = "SNAKE_MOVEMENT";
const SNAKE_GROW: &str = "SNAKE_GROW";
const SNAKE_FALL: &str = "SNAKE_FALL";
const SMOOTH_MOVEMENT: &str = "SMOOTH_MOVEMENT";

impl Plugin for MovementPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSnakeEvent>()
            .add_event::<SnakeMovedEvent>()
            .add_event::<MoveCommandEvent>()
            .add_event::<SnakeReachGoalEvent>()
            .add_event::<SnakeExitedLevelEvent>()
            .add_event::<crate::gameplay::undo::UndoEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>()
                    .label(KEYBOARD_INPUT)
                    .with_system(keyboard_undo_system)
                    .with_system(keyboard_move_command_system)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>()
                    .label(UNDO)
                    .after(KEYBOARD_INPUT)
                    .with_system(undo_event_system)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>()
                    .label(SNAKE_MOVEMENT)
                    .after(UNDO)
                    .with_system(snake_movement_control_system)
                    .into(),
            )
            .add_system(
                grow_snake_on_move_system
                    .run_in_state(GameState::Game)
                    .label(SNAKE_GROW)
                    .after(SNAKE_MOVEMENT),
            )
            .add_system(
                gravity_system
                    .run_in_state(GameState::Game)
                    .label(SNAKE_FALL)
                    .after(SNAKE_GROW),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Game)
                    .run_if_resource_exists::<LevelInstance>()
                    .label(SMOOTH_MOVEMENT)
                    .after(SNAKE_FALL)
                    .with_system(snake_smooth_movement_system)
                    .with_system(snake_push_anim_system)
                    .with_system(snake_exit_level_anim_system)
                    .with_system(respawn_snake_on_fall_system)
                    .into(),
            )
            .add_system(
                component_animator_system::<PartGrowAnim>
                    .run_in_state(GameState::Game)
                    .label(AnimationSystem::AnimationUpdate),
            );
    }
}

fn min_distance_to_ground(level: &LevelInstance, snake: &Snake) -> i32 {
    snake
        .parts()
        .iter()
        .map(|(position, _)| level.get_distance_to_ground(*position, snake.index()))
        .min()
        .unwrap()
}

pub fn keyboard_move_command_system(
    keyboard: Res<Input<KeyCode>>,
    mut move_command_event: EventWriter<MoveCommandEvent>,
) {
    let new_direction = if keyboard.any_just_pressed(MOVE_UP_KEYS) {
        Some(UP)
    } else if keyboard.any_just_pressed(MOVE_LEFT_KEYS) {
        Some(LEFT)
    } else if keyboard.any_just_pressed(MOVE_DOWN_KEYS) {
        Some(DOWN)
    } else if keyboard.any_just_pressed(MOVE_RIGHT_KEYS) {
        Some(RIGHT)
    } else {
        None
    };

    let Some(direction) = new_direction else {
        return;
    };

    move_command_event.send(MoveCommandEvent(direction));
}

type WithMovementControlSystemFilter = (
    With<SelectedSnake>,
    With<Active>,
    Without<MoveCommand>,
    Without<GravityFall>,
);

#[allow(clippy::too_many_arguments)]
pub fn snake_movement_control_system(
    assets: Res<Assets>,
    audio: Res<Audio>,
    mut level_instance: ResMut<LevelInstance>,
    constants: Res<GameConstants>,
    mut snake_history: ResMut<SnakeHistory>,
    mut move_command_event: EventReader<MoveCommandEvent>,
    mut snake_reach_goal_event: EventWriter<SnakeReachGoalEvent>,
    mut commands: Commands,
    mut snake_moved_event: EventWriter<SnakeMovedEvent>,
    mut selected_snake_query: Query<(Entity, &mut Snake), WithMovementControlSystemFilter>,
    mut other_snakes_query: Query<(Entity, &mut Snake), Without<SelectedSnake>>,
    foods_query: Query<&Food>,
    goal_query: Query<&Goal, With<Active>>,
) {
    let Ok((snake_entity, mut snake)) = selected_snake_query.get_single_mut() else {
        return;
    };

    let Some(MoveCommandEvent(direction)) = move_command_event.iter().next() else {
        return;
    };

    let new_position = snake.head_position() + *direction;

    // Check that we have enough parts to go up.
    let is_goal = if let Ok(goal) = goal_query.get_single() {
        goal.0 == new_position
    } else {
        false
    };

    if *direction == IVec2::Y
        && snake.is_standing()
        && !level_instance.is_food(new_position)
        && !is_goal
    {
        commands.entity(snake_entity).insert(GravityFall {
            velocity: constants.jump_velocity,
            relative_y: 0.0,
            grid_distance: 0,
        });
        return;
    }

    // Check for collition with self and walls.
    if snake.occupies_position(new_position) || level_instance.is_wall_or_spike(new_position) {
        return;
    }

    // Find if there is a snake in the way.
    let (other_snake_entity, mut other_snake) = level_instance
        .is_snake(new_position)
        .and_then(|other_snake_id| {
            other_snakes_query
                .iter_mut()
                .find(|(_, snake)| snake.index() == other_snake_id)
        })
        .unzip();

    if let Some(other_snake) = &mut other_snake {
        if !level_instance.can_push_snake(other_snake.as_ref(), *direction) {
            return;
        }
    };

    let other_snake = other_snake.as_mut().map(|some| some.as_mut());

    // Any food?
    let food = foods_query.iter().find(|food| food.0 == new_position);

    // Finaly move the snake forward and commit the state.
    let mut snake_commands = SnakeCommands::new(&mut level_instance, &mut snake_history);

    snake_commands
        .player_move(snake.as_mut(), *direction)
        .pushing_snake(other_snake)
        .eating_food(food)
        .execute();

    if let Ok(goal) = goal_query.get_single() {
        if snake.head_position() == goal.0 {
            snake_reach_goal_event.send(SnakeReachGoalEvent(snake_entity));
        }
    }

    snake_moved_event.send(SnakeMovedEvent);

    // Smooth move animation starts.
    commands.entity(snake_entity).insert(MoveCommand {
        velocity: constants.move_velocity,
        lerp_time: 0.0,
    });

    if let Some(other_snake_entity) = other_snake_entity {
        commands.entity(other_snake_entity).insert(PushedAnim {
            direction: direction.as_vec2(),
            velocity: constants.move_velocity,
            lerp_time: 0.0,
        });
    }

    audio
        .play(assets.move_effect_2.clone())
        .with_playback_rate(1.0 + rand::thread_rng().gen_range(-0.05..0.1))
        .with_volume(2.0);
}

pub fn grow_snake_on_move_system(
    mut snake_moved_event: EventReader<SnakeMovedEvent>,
    mut commands: Commands,
    snake_query: Query<(Entity, &Snake), With<SelectedSnake>>,
    foods_query: Query<(Entity, &Food), With<Food>>,
) {
    if snake_moved_event.iter().next().is_none() {
        return;
    }

    let Ok((snake_entity, snake)) = snake_query.get_single() else {
        return;
    };

    for (food_entity, food) in &foods_query {
        if food.0 != snake.head_position() {
            continue;
        }

        commands.entity(food_entity).despawn();

        let grow_tween = Tween::new(
            EaseFunction::QuadraticInOut,
            std::time::Duration::from_secs_f32(0.2),
            GrowPartLens,
        );

        commands.entity(snake_entity).with_children(|parent| {
            parent
                .spawn(SnakePartBundle::new(snake.index(), snake.len() - 1))
                .insert((Animator::new(grow_tween), PartGrowAnim { grow_factor: 0.0 }));
        });
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn gravity_system(
    time: Res<Time>,
    constants: Res<GameConstants>,
    mut level: ResMut<LevelInstance>,
    mut snake_history: ResMut<SnakeHistory>,
    mut trigger_undo_event: EventWriter<UndoEvent>,
    mut snake_reach_goal_event: EventReader<SnakeReachGoalEvent>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Snake,
            Option<&mut GravityFall>,
            Option<&SelectedSnake>,
        ),
        (With<Active>, Without<LevelExitAnim>),
    >,
) {
    let mut sorted_snakes: Vec<(
        Entity,
        Mut<Snake>,
        Option<Mut<GravityFall>>,
        Option<&SelectedSnake>,
    )> = query.iter_mut().collect();

    sorted_snakes.sort_by_key(|(_, _, _, selected_snake)| selected_snake.is_none());

    for (snake_entity, mut snake, gravity_fall, _) in sorted_snakes.into_iter() {
        if snake_reach_goal_event
            .iter()
            .any(|event| event.0 == snake_entity)
        {
            continue;
        }

        match gravity_fall {
            Some(mut gravity_fall) => {
                gravity_fall.velocity -= constants.gravity * time.delta_seconds();
                gravity_fall.relative_y += gravity_fall.velocity * time.delta_seconds();

                // While relative y is positive, we haven't moved fully into the cell.
                if gravity_fall.relative_y >= 0.0 {
                    continue;
                }

                // Check if we fell on spikes, if, so trigger undo.
                for (position, _) in snake.parts() {
                    if !level.is_spike(*position) {
                        continue;
                    }

                    let mut snake_commands = SnakeCommands::new(&mut level, &mut snake_history);
                    snake_commands.stop_falling_on_spikes(snake.as_ref());

                    commands.entity(snake_entity).remove::<GravityFall>();

                    trigger_undo_event.send(UndoEvent);
                    return;
                }

                // keep falling..
                if min_distance_to_ground(&level, &snake) > 1 {
                    gravity_fall.relative_y = GRID_TO_WORLD_UNIT;
                    gravity_fall.grid_distance += 1;

                    snake.fall_one_unit();
                } else {
                    // ..or stop falling animation.
                    commands.entity(snake_entity).remove::<GravityFall>();

                    // Nothing to do if we fell less than an unit, meaning we stayed at the same place.
                    if gravity_fall.grid_distance == 0 {
                        return;
                    }

                    let mut snake_commands = SnakeCommands::new(&mut level, &mut snake_history);
                    snake_commands.stop_falling(snake.as_ref());
                }
            }
            None => {
                // Check if snake is on the ground and spawn gravity fall if not.
                let min_distance_to_ground = min_distance_to_ground(&level, &snake);
                if min_distance_to_ground > 1 {
                    let mut snake_commands = SnakeCommands::new(&mut level, &mut snake_history);
                    snake_commands.start_falling(snake.as_ref());

                    snake.fall_one_unit();

                    commands.entity(snake_entity).insert(GravityFall {
                        velocity: 0.0,
                        relative_y: GRID_TO_WORLD_UNIT,
                        grid_distance: 1,
                    });
                }
            }
        }
    }
}

fn snake_smooth_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MoveCommand)>,
) {
    for (entity, mut move_command) in query.iter_mut() {
        move_command.lerp_time +=
            move_command.velocity * GRID_TO_WORLD_UNIT_INVERSE * time.delta_seconds();
        if move_command.lerp_time > 1.0 {
            commands.entity(entity).remove::<MoveCommand>();
        }
    }
}

pub fn snake_push_anim_system(
    time: Res<Time>,
    mut commands: Commands,
    mut push_anim_query: Query<(Entity, &mut PushedAnim)>,
) {
    for (entity, mut move_command) in push_anim_query.iter_mut() {
        move_command.lerp_time +=
            move_command.velocity * GRID_TO_WORLD_UNIT_INVERSE * time.delta_seconds();
        if move_command.lerp_time > 1.0 {
            commands.entity(entity).remove::<PushedAnim>();
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn snake_exit_level_anim_system(
    constants: Res<GameConstants>,
    level: Res<LevelTemplate>,
    mut commands: Commands,
    mut event_despawn_snake_parts: EventWriter<DespawnSnakePartEvent>,
    mut event_snake_exited_level: EventWriter<SnakeExitedLevelEvent>,
    mut anim_query: Query<(
        Entity,
        &mut Snake,
        &mut LevelExitAnim,
        Option<&MoveCommand>,
        &Children,
    )>,
    mut snake_part_query: Query<(Entity, &SnakePart, Option<&mut PartClipper>)>,
    eye_query: Query<(Entity, &Parent, &GlobalTransform), With<SnakeEye>>,
) {
    for (entity, mut snake, mut level_exit, move_command, children) in anim_query.iter_mut() {
        for &child in children {
            let Ok((entity, part, modifier)) = snake_part_query.get_mut(child) else {
                continue;
            };

            if modifier.is_some() {
                if (snake.parts()[part.part_index].0 - level.goal_position)
                    .abs()
                    .max_element()
                    > 1
                {
                    event_despawn_snake_parts.send(DespawnSnakePartEvent(part.clone()));
                }

                for (eye_entity, parent, transform) in &eye_query {
                    if parent.get() != entity {
                        continue;
                    }
                    let offset = transform.translation().truncate() - to_world(level.goal_position);
                    let distance = offset.dot(snake.parts()[part.part_index].1.as_vec2());

                    if distance > 0.0 {
                        commands.entity(eye_entity).despawn();
                    }
                }
            } else if snake.parts()[part.part_index].0 == level.goal_position {
                commands.entity(entity).insert(PartClipper {
                    clip_position: level.goal_position,
                });
            }
        }

        if move_command.is_some() {
            continue;
        }

        level_exit.distance_to_move -= 1;

        if level_exit.distance_to_move < 0 {
            commands
                .entity(entity)
                .remove::<LevelExitAnim>()
                .remove::<Active>();

            event_snake_exited_level.send(SnakeExitedLevelEvent);

            snake.set_parts(level_exit.initial_snake_position.clone());
        } else {
            commands.entity(entity).insert(MoveCommand {
                velocity: 2.0 * constants.move_velocity,
                lerp_time: 0.0,
            });
            let direction = snake.head_direction();
            snake.move_forward(direction);
        }
    }
}

use bevy::prelude::*;

use crate::{
    commands::SnakeCommands,
    game_constants_pluggin::*,
    level_instance::LevelInstance,
    level_pluggin::Food,
    snake_pluggin::{
        grow_snake_on_move_system, respawn_snake_on_fall_system, Active, SelectedSnake, Snake,
        SnakePart, SpawnSnakeEvent,
    },
    undo::{keyboard_undo_system, undo_event_system, SnakeHistory, UndoEvent},
};

const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

#[derive(Component, Default)]
pub struct MoveCommand {
    direction: Option<IVec2>,
    velocity: f32,
    anim_offset: f32,
}

#[derive(Component, Copy, Clone)]
pub struct GravityFall {
    velocity: f32,
    relative_y: f32,
    pub grid_distance: i32,
}

pub struct MovementPluggin;

pub struct MoveCommandEvent(pub IVec2);

pub struct SnakeMovedEvent;

impl Plugin for MovementPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSnakeEvent>()
            .add_event::<SnakeMovedEvent>()
            .add_event::<MoveCommandEvent>()
            .add_event::<crate::undo::UndoEvent>()
            .add_system(keyboard_undo_system)
            .add_system(keyboard_move_command_system)
            .add_system(undo_event_system.after(keyboard_undo_system))
            .add_system(
                snake_movement_control_system
                    .after(undo_event_system)
                    .after(keyboard_move_command_system),
            )
            .add_system(grow_snake_on_move_system.after(snake_movement_control_system))
            .add_system(gravity_system.after(grow_snake_on_move_system))
            .add_system(snake_smooth_movement_system.after(gravity_system))
            .add_system(respawn_snake_on_fall_system.after(gravity_system))
            .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system);
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
    mut level_instance: ResMut<LevelInstance>,
    constants: Res<GameConstants>,
    mut snake_history: ResMut<SnakeHistory>,
    mut move_command_event: EventReader<MoveCommandEvent>,
    mut commands: Commands,
    mut snake_moved_event: EventWriter<SnakeMovedEvent>,
    mut selected_snake_query: Query<(Entity, &mut Snake), WithMovementControlSystemFilter>,
    mut other_snakes_query: Query<(Entity, &mut Snake), Without<SelectedSnake>>,
    foods_query: Query<&Food>,
) {
    let Ok((snake_entity, mut snake)) = selected_snake_query.get_single_mut() else {
        return;
    };

    let Some(MoveCommandEvent(direction)) = move_command_event.iter().next() else {
        return;
    };

    let new_position = snake.head_position() + *direction;

    // Check that we have enough parts to go up.
    if *direction == IVec2::Y && snake.is_standing() && !level_instance.is_food(new_position) {
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

    snake_moved_event.send(SnakeMovedEvent);

    // Smooth move animation starts.
    commands.entity(snake_entity).insert(MoveCommand {
        direction: None,
        velocity: constants.move_velocity,
        anim_offset: GRID_TO_WORLD_UNIT,
    });

    if let Some(other_snake_entity) = other_snake_entity {
        commands.entity(other_snake_entity).insert(MoveCommand {
            direction: Some(*direction),
            velocity: constants.move_velocity,
            anim_offset: GRID_TO_WORLD_UNIT,
        });
    }
}

pub fn gravity_system(
    time: Res<Time>,
    constants: Res<GameConstants>,
    mut level: ResMut<LevelInstance>,
    mut snake_history: ResMut<SnakeHistory>,
    mut trigger_undo_event: EventWriter<UndoEvent>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake, Option<&mut GravityFall>), With<Active>>,
) {
    for (snake_entity, mut snake, gravity_fall) in query.iter_mut() {
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
        move_command.anim_offset -= move_command.velocity + time.delta_seconds();
        if move_command.anim_offset < 0.0 {
            commands.entity(entity).remove::<MoveCommand>();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_sprite_positions_system(
    snake_query: Query<(&Snake, Option<&MoveCommand>, Option<&GravityFall>), With<Active>>,
    mut sprite_query: Query<(&mut Transform, &SnakePart)>,
) {
    for (snake, move_command, gravity_fall) in snake_query.iter() {
        for (mut transform, part) in sprite_query.iter_mut() {
            if part.snake_index != snake.index() {
                continue;
            }

            // Can happen when undoing grow, before the despawn have take effect.
            if part.part_index > snake.len() - 1 {
                continue;
            }

            let mut part_position = to_world(snake.parts()[part.part_index].0);

            // Move sprite with move anim.
            if let Some(move_command) = move_command {
                let direction = move_command
                    .direction
                    .unwrap_or(snake.parts()[part.part_index].1);

                part_position -= move_command.anim_offset * direction.as_vec2();

                // Extend sprites at a turn to cover the gaps. Reset normal size otherwize.
                if move_command.direction.is_none()
                    && part.part_index < snake.len() - 1
                    && direction != snake.parts()[part.part_index + 1].1
                {
                    let size_offset =
                        direction.as_vec2() * (GRID_TO_WORLD_UNIT - move_command.anim_offset);
                    transform.scale =
                        (Vec2::ONE + size_offset.abs() * GRID_TO_WORLD_UNIT_INVERSE).extend(1.0);
                    part_position -= size_offset * 0.5;
                } else {
                    transform.scale = Vec3::ONE;
                }
            } else {
                transform.scale = Vec3::ONE;
            }

            // Move sprite with gravity fall anim.
            if let Some(gravity_fall) = gravity_fall {
                part_position += gravity_fall.relative_y * Vec2::Y;
            }

            transform.translation.x = part_position.x;
            transform.translation.y = part_position.y;
        }
    }
}

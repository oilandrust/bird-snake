use core::panic;

use bevy::prelude::*;

use crate::{
    game_constants_pluggin::*,
    level_pluggin::{spawn_food, LevelInstance, Walkable},
    level_template::SnakeElement,
    snake_pluggin::{
        grow_snake_on_move_system, respawn_snake_on_fall_system, DespawnSnakePartEvent,
        SelectedSnake, Snake, SnakePart, SpawnSnakeEvent,
    },
};

const MOVE_UP_KEYS: [KeyCode; 2] = [KeyCode::W, KeyCode::Up];
const MOVE_LEFT_KEYS: [KeyCode; 2] = [KeyCode::A, KeyCode::Left];
const MOVE_DOWN_KEYS: [KeyCode; 2] = [KeyCode::S, KeyCode::Down];
const MOVE_RIGHT_KEYS: [KeyCode; 2] = [KeyCode::D, KeyCode::Right];

#[derive(Component, Default)]
pub struct MoveCommand {
    velocity: f32,
    anim_offset: f32,
}

#[derive(Component, Copy, Clone)]
pub struct GravityFall {
    velocity: f32,
    relative_y: f32,
    pub grid_distance: i32,
}

#[derive(Copy, Clone)]
pub enum MoveHistoryEvent {
    Move((IVec2, IVec2)),
    Fall(i32),
    Eat(IVec2),
}

#[derive(Copy, Clone)]
struct SnakeHistoryEvent {
    event: MoveHistoryEvent,
    snake_index: i32,
}

#[derive(Resource, Default)]
pub struct SnakeHistory {
    move_history: Vec<SnakeHistoryEvent>,
}

pub struct UndoEvent;

impl SnakeHistory {
    pub fn push(&mut self, event: MoveHistoryEvent, snake_index: i32) {
        self.move_history
            .push(SnakeHistoryEvent { event, snake_index });
    }

    pub fn undo_last(
        &mut self,
        snakes: &mut [Mut<Snake>],
        level: &mut LevelInstance,
        commands: &mut Commands,
        despawn_snake_part_event: &mut EventWriter<DespawnSnakePartEvent>,
    ) {
        let top = *self.move_history.last().unwrap();
        let snake: &mut Snake = snakes
            .iter_mut()
            .find(|snake| snake.index == top.snake_index)
            .expect("Missing snake in query")
            .as_mut();

        match top.event {
            MoveHistoryEvent::Move(part) => {
                self.undo_move(level, snake, &part);
                self.move_history.pop();
            }
            MoveHistoryEvent::Fall(fall_distance) => {
                for (position, _) in &snake.parts {
                    level.set_empty(*position);
                }

                snake.move_up(fall_distance);

                for (position, _) in &snake.parts {
                    level.mark_position_walkable(*position, Walkable::Snake(snake.index as i32));
                }

                self.move_history.pop();

                // If a fall history happens, it must be preceded by a move, undo that as well.
                self.expect_and_undo_move(level, snake);
            }
            MoveHistoryEvent::Eat(position) => {
                despawn_snake_part_event.send(DespawnSnakePartEvent(SnakePart {
                    snake_index: snake.index,
                    part_index: snake.parts.len() - 1,
                }));

                level.set_empty(snake.tail_position());

                spawn_food(commands, &position, level);

                self.move_history.pop();

                // If a eat history happens, it must be preceded by a move, undo that as well.
                self.expect_and_undo_move(level, snake);
            }
        }
    }

    fn undo_move(&mut self, level: &mut LevelInstance, snake: &mut Snake, part: &SnakeElement) {
        let old_head = snake.head_position();

        snake.parts.push_back(*part);
        snake.parts.pop_front();

        level.set_empty(old_head);
        level.mark_position_walkable(part.0, Walkable::Snake(snake.index as i32));
    }

    fn expect_and_undo_move(&mut self, level: &mut LevelInstance, snake: &mut Snake) {
        assert!(!self.move_history.is_empty());
        let event = self.move_history.last().unwrap();
        if let MoveHistoryEvent::Move(part) = event.event {
            self.undo_move(level, snake, &part);

            self.move_history.pop();
        } else {
            panic!("Fall history should always happen after a move history.")
        }
    }
}

pub struct MovementPluggin;

pub struct SnakeMovedEvent;

impl Plugin for MovementPluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSnakeEvent>()
            .add_event::<SnakeMovedEvent>()
            .add_event::<UndoEvent>()
            .add_system(snake_movement_undo_system)
            .add_system(snake_movement_control_system.after(snake_movement_undo_system))
            .add_system(grow_snake_on_move_system.after(snake_movement_control_system))
            .add_system(gravity_system.after(grow_snake_on_move_system))
            .add_system(undo_event_system.after(gravity_system))
            .add_system(snake_smooth_movement_system.after(gravity_system))
            .add_system(respawn_snake_on_fall_system.after(gravity_system))
            .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system);
    }
}

fn min_distance_to_ground(level: &LevelInstance, snake: &Snake) -> i32 {
    snake
        .parts
        .iter()
        .map(|(position, _)| level.get_distance_to_ground(*position, snake.index as i32))
        .min()
        .unwrap()
}

type WithMovementControlSystemFilter = (
    With<SelectedSnake>,
    Without<MoveCommand>,
    Without<GravityFall>,
);

pub fn snake_movement_control_system(
    keyboard: Res<Input<KeyCode>>,
    mut level: ResMut<LevelInstance>,
    constants: Res<GameConstants>,
    mut snake_history: ResMut<SnakeHistory>,
    mut commands: Commands,
    mut snake_moved_event: EventWriter<SnakeMovedEvent>,
    mut query: Query<(Entity, &mut Snake), WithMovementControlSystemFilter>,
) {
    let Ok((snake_entity, mut snake)) = query.get_single_mut() else {
        return;
    };

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

    let new_position = snake.parts[0].0 + direction;

    // Check that we have enough parts to go up.
    if direction == IVec2::Y && snake.is_standing() && !level.is_food(new_position) {
        commands.entity(snake_entity).insert(GravityFall {
            velocity: constants.jump_velocity,
            relative_y: 0.0,
            grid_distance: 0,
        });
        return;
    }

    // Check for collition with self.
    if snake.occupies_position(new_position) || !level.is_food_or_empty(new_position) {
        return;
    }

    snake_history.push(
        MoveHistoryEvent::Move(*snake.parts.back().unwrap()),
        snake.index,
    );

    // Finaly move the snake forward.
    level.set_empty(snake.tail_position());
    level.mark_position_walkable(new_position, Walkable::Snake(snake.index as i32));

    snake.parts.push_front((new_position, direction));
    snake.parts.pop_back();

    snake_moved_event.send(SnakeMovedEvent);

    // Smooth move animation starts.
    commands.entity(snake_entity).insert(MoveCommand {
        velocity: constants.move_velocity,
        anim_offset: GRID_TO_WORLD_UNIT,
    });
}

pub fn snake_movement_undo_system(
    keyboard: Res<Input<KeyCode>>,
    mut trigger_undo_event: EventWriter<UndoEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Back) {
        return;
    }

    trigger_undo_event.send(UndoEvent);
}

pub fn undo_event_system(
    mut trigger_undo_event: EventReader<UndoEvent>,
    mut snake_history: ResMut<SnakeHistory>,
    mut level: ResMut<LevelInstance>,
    mut despawn_snake_part_event: EventWriter<DespawnSnakePartEvent>,
    mut commands: Commands,
    mut query: Query<&mut Snake>,
) {
    if trigger_undo_event.iter().next().is_none() {
        return;
    }

    if snake_history.move_history.is_empty() {
        return;
    }

    let mut snakes: Vec<Mut<Snake>> = query.iter_mut().collect();

    snake_history.undo_last(
        &mut snakes,
        &mut level,
        &mut commands,
        &mut despawn_snake_part_event,
    );
}

fn gravity_system(
    time: Res<Time>,
    constants: Res<GameConstants>,
    mut level: ResMut<LevelInstance>,
    mut snake_history: ResMut<SnakeHistory>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Snake, Option<&mut GravityFall>)>,
) {
    for (snake_entity, mut snake, gravity_fall) in query.iter_mut() {
        match gravity_fall {
            Some(mut gravity_fall) => {
                gravity_fall.velocity -= constants.gravity * time.delta_seconds();
                gravity_fall.relative_y += gravity_fall.velocity * time.delta_seconds();

                // When relative y is 0, the sprites are aligned with the actual position.
                if gravity_fall.relative_y < 0.0 {
                    // keep falling..
                    if min_distance_to_ground(&level, &snake) > 1 {
                        gravity_fall.relative_y = GRID_TO_WORLD_UNIT;
                        gravity_fall.grid_distance += 1;

                        for (position, _) in &snake.parts {
                            level.set_empty(*position);
                        }

                        snake.fall_one_unit();
                    } else {
                        // ..or stop falling animation.
                        commands.entity(snake_entity).remove::<GravityFall>();

                        for (position, _) in &snake.parts {
                            level.mark_position_walkable(
                                *position,
                                Walkable::Snake(snake.index as i32),
                            );
                        }

                        snake_history.push(
                            MoveHistoryEvent::Fall(gravity_fall.grid_distance),
                            snake.index,
                        );
                    }
                }
            }
            None => {
                // Check if snake is on the ground and spawn gravity fall if not.
                if min_distance_to_ground(&level, &snake) > 1 {
                    commands.entity(snake_entity).insert(GravityFall {
                        velocity: 0.0,
                        relative_y: GRID_TO_WORLD_UNIT,
                        grid_distance: 1,
                    });

                    for (position, _) in &snake.parts {
                        level.set_empty(*position);
                    }

                    snake.fall_one_unit();
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

pub fn update_sprite_positions_system(
    snake_query: Query<(&Snake, Option<&MoveCommand>, Option<&GravityFall>)>,
    mut sprite_query: Query<(&mut Transform, &SnakePart)>,
) {
    for (snake, move_command, gravity_fall) in snake_query.iter() {
        for (mut transform, part) in sprite_query.iter_mut() {
            if part.snake_index != snake.index {
                continue;
            }

            let mut part_position = to_world(snake.parts[part.part_index].0);

            // Move sprite with move anim.
            if let Some(move_command) = move_command {
                let direction = snake.parts[part.part_index].1;
                part_position -= move_command.anim_offset * direction.as_vec2();

                // Extend sprites at a turn to cover the gaps. Reset normal size otherwize.
                if part.part_index < snake.parts.len() - 1
                    && direction != snake.parts[part.part_index + 1].1
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

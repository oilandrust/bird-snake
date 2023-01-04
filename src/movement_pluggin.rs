use bevy::prelude::*;

use crate::{
    game_constants_pluggin::*,
    level_pluggin::{spawn_food, LevelInstance},
    snake_pluggin::{
        grow_snake_on_move_system, respawn_snake_on_fall_system, DespawnSnakePartEvent, Snake,
        SnakePart, SpawnSnakeEvent,
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

#[derive(Resource, Default)]
pub struct SnakeHistory {
    move_history: Vec<MoveHistoryEvent>,
}

impl SnakeHistory {
    pub fn push(&mut self, event: MoveHistoryEvent) {
        self.move_history.push(event);
    }

    pub fn undo_last(
        &mut self,
        snake: &mut Snake,
        level: &mut LevelInstance,
        commands: &mut Commands,
        despawn_snake_part_event: &mut EventWriter<DespawnSnakePartEvent>,
    ) {
        let top = *self.move_history.last().unwrap();
        match top {
            MoveHistoryEvent::Move(part) => {
                snake.parts.push_back(part);
                snake.parts.pop_front();
                self.move_history.pop();
            }
            MoveHistoryEvent::Fall(fall_distance) => {
                snake.move_up(fall_distance);
                self.move_history.pop();

                // If a fall history happens, it must be preceded by a move, undo that as well.
                self.expect_and_undo_move(snake);
            }
            MoveHistoryEvent::Eat(position) => {
                despawn_snake_part_event.send(DespawnSnakePartEvent(snake.parts.len() - 1));

                spawn_food(commands, &position, level);

                self.move_history.pop();

                // If a eat history happens, it must be preceded by a move, undo that as well.
                self.expect_and_undo_move(snake);
            }
        }
    }

    fn expect_and_undo_move(&mut self, snake: &mut Snake) {
        assert!(!self.move_history.is_empty());
        if let MoveHistoryEvent::Move(part) = self.move_history.last().unwrap() {
            snake.parts.push_back(*part);
            snake.parts.pop_front();

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
            .add_system(snake_movement_undo_system)
            .add_system(snake_movement_control_system.after(snake_movement_undo_system))
            .add_system(grow_snake_on_move_system.after(snake_movement_control_system))
            .add_system(gravity_system.after(grow_snake_on_move_system))
            .add_system(snake_smooth_movement_system.after(gravity_system))
            .add_system(respawn_snake_on_fall_system.after(gravity_system))
            .add_system_to_stage(CoreStage::PostUpdate, update_sprite_positions_system);
    }
}

fn min_distance_to_ground(level: &LevelInstance, snake: &Snake) -> i32 {
    snake
        .parts
        .iter()
        .map(|(position, _)| level.get_distance_to_ground(*position))
        .min()
        .unwrap()
}

type WithoutMoveOrFall = (Without<MoveCommand>, Without<GravityFall>);

pub fn snake_movement_control_system(
    keyboard: Res<Input<KeyCode>>,
    level: Res<LevelInstance>,
    constants: Res<GameConstants>,
    mut snake_history: ResMut<SnakeHistory>,
    mut commands: Commands,
    mut snake_moved_event: EventWriter<SnakeMovedEvent>,
    mut query: Query<(Entity, &mut Snake), WithoutMoveOrFall>,
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

    snake_history.push(MoveHistoryEvent::Move(*snake.parts.back().unwrap()));

    // Finaly move the snake forward.
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
    mut snake_history: ResMut<SnakeHistory>,
    mut level: ResMut<LevelInstance>,
    mut despawn_snake_part_event: EventWriter<DespawnSnakePartEvent>,
    mut commands: Commands,
    mut query: Query<&mut Snake>,
) {
    if !keyboard.just_pressed(KeyCode::Back) {
        return;
    }

    if snake_history.move_history.is_empty() {
        return;
    }

    let Ok(mut snake) = query.get_single_mut() else {
        return;
    };

    snake_history.undo_last(
        &mut snake,
        &mut level,
        &mut commands,
        &mut despawn_snake_part_event,
    );
}

fn gravity_system(
    time: Res<Time>,
    constants: Res<GameConstants>,
    level: Res<LevelInstance>,
    mut snake_history: ResMut<SnakeHistory>,
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
                    gravity_fall.relative_y = GRID_TO_WORLD_UNIT;
                    gravity_fall.grid_distance += 1;

                    snake.fall_one_unit();
                } else {
                    // ..or stop falling animation.
                    commands.entity(snake_entity).remove::<GravityFall>();

                    snake_history.push(MoveHistoryEvent::Fall(gravity_fall.grid_distance));
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

pub fn update_sprite_positions_system(
    snake_query: Query<(&Snake, Option<&MoveCommand>, Option<&GravityFall>)>,
    mut sprite_query: Query<(&mut Transform, &SnakePart)>,
) {
    let Ok((snake, move_command, gravity_fall)) = snake_query.get_single() else {
        return;
    };

    for (mut transform, part) in sprite_query.iter_mut() {
        let mut part_position = to_world(snake.parts[part.0].0);

        // Move sprite with move anim.
        if let Some(move_command) = move_command {
            let direction = snake.parts[part.0].1;
            part_position -= move_command.anim_offset * direction.as_vec2();

            // Extend sprites at a turn to cover the gaps. Reset normal size otherwize.
            if part.0 < snake.parts.len() - 1 && direction != snake.parts[part.0 + 1].1 {
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

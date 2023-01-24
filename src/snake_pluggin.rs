use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillMode, Path, PathBuilder, StrokeMode},
};
use bevy_tweening::Lens;
use std::{collections::VecDeque, mem};

use crate::{
    commands::SnakeCommands,
    game_constants_pluggin::{to_grid, to_world, GRID_TO_WORLD_UNIT, SNAKE_COLORS, SNAKE_EYE_SIZE},
    level_instance::{LevelEntityType, LevelInstance},
    level_pluggin::{Food, LevelEntity},
    level_template::{LevelTemplate, SnakeTemplate},
    movement_pluggin::{GravityFall, MoveCommand, PushedAnim, SnakeMovedEvent},
    undo::{SnakeHistory, UndoEvent},
};

pub struct SnakePluggin;

impl Plugin for SnakePluggin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnSnakePartEvent>()
            .add_event::<DespawnSnakeEvent>()
            .add_event::<DespawnSnakePartsEvent>()
            .add_system_to_stage(CoreStage::PreUpdate, spawn_snake_system)
            .add_system(select_snake_mouse_system)
            .add_system_to_stage(CoreStage::PostUpdate, update_snake_parts_mesh_system)
            .add_system_to_stage(CoreStage::PostUpdate, despawn_snake_system)
            .add_system_to_stage(CoreStage::PostUpdate, despawn_snake_parts_system);
    }
}

#[derive(PartialEq, Eq)]
pub struct DespawnSnakePartEvent(pub SnakePart);

#[derive(PartialEq, Eq)]
pub struct DespawnSnakeEvent(pub i32);

#[derive(PartialEq, Eq)]
pub struct DespawnSnakePartsEvent(pub i32);

#[derive(Component)]
pub struct SelectedSnake;

#[derive(Component)]
pub struct Active;

#[derive(Component, PartialEq, Eq, Reflect)]
pub struct SnakePart {
    pub snake_index: i32,
    pub part_index: usize,
}

#[derive(Component)]
pub struct SnakePartSprite;

#[derive(Bundle)]
struct SnakePartBundle {
    part: SnakePart,
    level_entity: LevelEntity,
    shape: ShapeBundle,
}

impl SnakePartBundle {
    fn new(snake_index: i32, part_index: usize) -> Self {
        SnakePartBundle {
            shape: ShapeBundle {
                mode: DrawMode::Outlined {
                    fill_mode: FillMode::color(SNAKE_COLORS[snake_index as usize]),
                    outline_mode: StrokeMode::new(Color::BLACK, 1.0),
                },
                ..default()
            },
            part: SnakePart {
                snake_index,
                part_index,
            },
            level_entity: LevelEntity,
        }
    }
}

struct GrowPartLens {
    scale_start: Vec2,
    scale_end: Vec2,
    grow_direction: Vec2,
}

impl Lens<Transform> for GrowPartLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.scale_start + (self.scale_end - self.scale_start) * ratio;
        target.scale = value.extend(1.0);

        let mut offset = 0.5 * value * self.grow_direction - 0.5 * self.grow_direction;
        offset *= GRID_TO_WORLD_UNIT;
        let z = target.translation.z;
        target.translation = (offset).extend(z);
    }
}

#[derive(Component, Debug)]
pub struct Snake {
    parts: VecDeque<(IVec2, IVec2)>,
    index: i32,
}

pub struct SpawnSnakeEvent;

impl Snake {
    pub fn parts(&self) -> &VecDeque<(IVec2, IVec2)> {
        &self.parts
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn move_back(&mut self, part: &(IVec2, IVec2)) {
        self.parts.push_back(*part);
        self.parts.pop_front();
    }

    pub fn move_forward(&mut self, direction: IVec2) {
        self.parts
            .push_front((self.head_position() + direction, direction));
        self.parts.pop_back();
    }

    pub fn head_position(&self) -> IVec2 {
        self.parts.front().unwrap().0
    }

    pub fn grow(&mut self) {
        let (tail_position, tail_direction) = self.tail();
        let new_part_position = tail_position - tail_direction;
        self.parts.push_back((new_part_position, tail_direction));
    }

    pub fn shrink(&mut self) {
        self.parts.pop_back();
    }

    pub fn tail(&self) -> (IVec2, IVec2) {
        *self.parts.back().unwrap()
    }

    pub fn tail_position(&self) -> IVec2 {
        self.parts.back().unwrap().0
    }

    pub fn is_standing(&self) -> bool {
        (self.parts.front().unwrap().0.y - self.parts.back().unwrap().0.y)
            == (self.len() - 1) as i32
    }

    pub fn occupies_position(&self, position: IVec2) -> bool {
        self.parts.iter().any(|part| part.0 == position)
    }

    pub fn fall_one_unit(&mut self) {
        for (position, _) in self.parts.iter_mut() {
            *position += IVec2::NEG_Y;
        }
    }

    pub fn translate(&mut self, offset: IVec2) {
        for (position, _) in self.parts.iter_mut() {
            *position += offset;
        }
    }

    pub fn set_parts(&mut self, parts: Vec<(IVec2, IVec2)>) {
        self.parts = parts.into();
    }
}

pub fn spawn_snake(
    commands: &mut Commands,
    level_instance: &mut LevelInstance,
    snake_template: &SnakeTemplate,
    snake_index: i32,
) -> Entity {
    let mut spawn_command = commands.spawn((
        Snake {
            parts: VecDeque::from(snake_template.clone()),
            index: snake_index,
        },
        SpatialBundle { ..default() },
        LevelEntity,
        Active,
    ));

    spawn_command.with_children(|parent| {
        for (index, part) in snake_template.iter().enumerate() {
            let mut part = parent.spawn(SnakePartBundle::new(snake_index, index));

            // Spawn the eye.
            if index == 0 {
                part.with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::BLACK,
                                custom_size: Some(SNAKE_EYE_SIZE),
                                ..default()
                            },
                            transform: Transform::from_xyz(5.0, 5.0, 1.0),
                            ..default()
                        },
                        LevelEntity,
                    ));
                });
            }
        }
    });

    for (position, _) in snake_template {
        level_instance.mark_position_occupied(*position, LevelEntityType::Snake(snake_index));
    }

    spawn_command.id()
}

const FOWARD_LEFT: IVec2 = IVec2::new(1, 1);
const FOWARD_RIGHT: IVec2 = IVec2::new(1, -1);
const BACK_RIGHT: IVec2 = IVec2::new(-1, -1);
const BACK_LEFT: IVec2 = IVec2::new(-1, 1);

const CORNERS: [IVec2; 4] = [FOWARD_LEFT, FOWARD_RIGHT, BACK_RIGHT, BACK_LEFT];

fn corner_position(corner: &IVec2, position: &IVec2, direction: &IVec2, ortho_dir: &IVec2) -> Vec2 {
    to_world(*position)
        + corner.x as f32 * 0.5 * GRID_TO_WORLD_UNIT * direction.as_vec2()
        + corner.y as f32 * 0.5 * GRID_TO_WORLD_UNIT * ortho_dir.as_vec2()
}

pub fn update_snake_parts_mesh_system(
    mut snake_parts_query: Query<(&mut Path, &SnakePart, &Parent)>,
    snake_query: Query<
        (
            &Snake,
            Option<&MoveCommand>,
            Option<&PushedAnim>,
            Option<&GravityFall>,
        ),
        With<Active>,
    >,
) {
    for (mut path, part, parent) in snake_parts_query.iter_mut() {
        let Ok((snake, move_command, pushed_anim, fall)) = snake_query.get(parent.get()) else {
            continue;
        };

        let mut path_builder = PathBuilder::new();

        let next_part = snake.parts.get(part.part_index + 1);
        let prev_part = if part.part_index > 0 {
            snake.parts.get(part.part_index - 1)
        } else {
            None
        };

        let mut part_vertices: Vec<Vec2> = Vec::with_capacity(5);

        let (position, direction) = snake.parts[part.part_index];
        let ortho_dir = IVec2::new(-direction.y, direction.x);

        part_vertices.clear();

        for corner in CORNERS {
            let corner_world_position = corner_position(&corner, &position, &direction, &ortho_dir);

            let mut anim_offset = Vec2::ZERO;
            if let Some(command) = move_command {
                let anim_direction = direction.as_vec2();
                let initial_offset = -GRID_TO_WORLD_UNIT * anim_direction;
                anim_offset = initial_offset.lerp(Vec2::ZERO, command.lerp_time);

                if let Some((_, next_direction)) = next_part {
                    let next_dir_relative =
                        IVec2::new(next_direction.dot(direction), next_direction.dot(ortho_dir));

                    if direction != *next_direction && corner.x < 0 {
                        if corner.dot(next_dir_relative) > 0 {
                            if command.lerp_time < 0.5 {
                                let mut extra_vertex_offset = -GRID_TO_WORLD_UNIT * anim_direction;
                                anim_offset = -GRID_TO_WORLD_UNIT * anim_direction
                                    - GRID_TO_WORLD_UNIT
                                        * (1.0 - 2.0 * command.lerp_time)
                                        * next_direction.as_vec2();

                                if next_dir_relative.y > 0 {
                                    mem::swap(&mut extra_vertex_offset, &mut anim_offset);
                                }

                                part_vertices.push(corner_world_position + extra_vertex_offset);
                            } else {
                                anim_offset = -direction.as_vec2()
                                    * GRID_TO_WORLD_UNIT
                                    * (1.0 - 2.0 * (command.lerp_time - 0.5));
                            }
                        } else {
                            anim_offset = Vec2::ZERO;
                        }
                    }
                }

                if let Some((_, prev_direction)) = prev_part {
                    let prev_dir_relative =
                        IVec2::new(prev_direction.dot(direction), prev_direction.dot(ortho_dir));

                    if direction != *prev_direction && corner.x > 0 {
                        if corner.dot(prev_dir_relative) < 0 {
                            if command.lerp_time < 0.5 {
                                anim_offset = direction.as_vec2()
                                    * GRID_TO_WORLD_UNIT
                                    * (2.0 * command.lerp_time - 1.0);
                            } else {
                                let mut extra_vertex_offset = Vec2::ZERO;
                                anim_offset = GRID_TO_WORLD_UNIT
                                    * (2.0 * (command.lerp_time - 0.5))
                                    * prev_direction.as_vec2();

                                if prev_dir_relative.y > 0 {
                                    mem::swap(&mut extra_vertex_offset, &mut anim_offset);
                                }

                                part_vertices.push(corner_world_position + extra_vertex_offset);
                            }
                        } else {
                            anim_offset = -direction.as_vec2() * GRID_TO_WORLD_UNIT;
                        }
                    }
                }
            }

            part_vertices.push(corner_world_position + anim_offset);
        }

        // Move sprite with gravity fall anim.
        let fall_offset = fall.map_or(Vec2::ZERO, |gravity_fall| gravity_fall.relative_y * Vec2::Y);

        let push_offset = pushed_anim.map_or(Vec2::ZERO, |command| {
            let initial_offset = -GRID_TO_WORLD_UNIT * command.direction;
            initial_offset.lerp(Vec2::ZERO, command.lerp_time)
        });

        let offset = fall_offset + push_offset;

        path_builder.move_to(*part_vertices.first().unwrap() + offset);
        part_vertices.iter().skip(1).for_each(|vertex| {
            path_builder.line_to(*vertex + offset);
        });

        path_builder.close();

        *path = path_builder.build();
    }
}

pub fn set_snake_active(commands: &mut Commands, snake: &Snake, snake_entity: Entity) {
    commands
        .entity(snake_entity)
        .insert(Active)
        .with_children(|parent| {
            for (index, part) in snake.parts().iter().enumerate() {
                let mut part = parent.spawn(SnakePartBundle::new(snake.index(), index));
            }
        });
}

pub fn spawn_snake_system(
    level: Res<LevelTemplate>,
    mut level_instance: ResMut<LevelInstance>,
    mut commands: Commands,
    mut event_spawn_snake: EventReader<SpawnSnakeEvent>,
) {
    if event_spawn_snake.iter().next().is_none() {
        return;
    }

    for (snake_index, snake_template) in level.initial_snakes.iter().enumerate() {
        let entity = spawn_snake(
            &mut commands,
            &mut level_instance,
            snake_template,
            snake_index as i32,
        );

        if snake_index == 0 {
            commands.entity(entity).insert(SelectedSnake);
        }
    }
}

pub fn select_snake_mouse_system(
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform)>,
    selected_snake: Query<Entity, With<SelectedSnake>>,
    unselected_snakes: Query<(Entity, &Snake), Without<SelectedSnake>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.get_primary().unwrap();

    let Some(mouse_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = camera.single();
    let mouse_world_position = {
        let window_size = Vec2::new(window.width(), window.height());
        let ndc = (mouse_position / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        world_pos.xy()
    };

    let mouse_grid_position = to_grid(mouse_world_position);
    let selected_snake_entity = selected_snake.single();

    for (entity, snake) in unselected_snakes.iter() {
        if !snake.occupies_position(mouse_grid_position) {
            continue;
        }

        commands
            .entity(selected_snake_entity)
            .remove::<SelectedSnake>();

        commands.entity(entity).insert(SelectedSnake);
    }
}

pub fn respawn_snake_on_fall_system(
    mut snake_history: ResMut<SnakeHistory>,
    mut level: ResMut<LevelInstance>,
    mut trigger_undo_event: EventWriter<UndoEvent>,
    mut commands: Commands,
    mut snake_query: Query<(Entity, &Snake), With<GravityFall>>,
) {
    for (snake_entity, snake) in snake_query.iter_mut() {
        if snake.head_position().y >= -2 {
            return;
        }

        let mut snake_commands = SnakeCommands::new(&mut level, &mut snake_history);
        snake_commands.stop_falling(snake);

        commands.entity(snake_entity).remove::<GravityFall>();

        trigger_undo_event.send(UndoEvent);
    }
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

    let (snake_entity, snake) = snake_query.single();

    for (food_entity, food) in &foods_query {
        if food.0 != snake.head_position() {
            continue;
        }

        commands.entity(food_entity).despawn();

        //let (tail_direction, new_part_position) = snake.tail();

        // let grow_tween = Tween::new(
        //     EaseFunction::QuadraticInOut,
        //     std::time::Duration::from_secs_f32(0.2),
        //     GrowPartLens {
        //         scale_start: Vec2::ONE - tail_direction.as_vec2().abs(),
        //         scale_end: Vec2::ONE,
        //         grow_direction: -tail_direction.as_vec2(),
        //     },
        // );

        commands.entity(snake_entity).with_children(|parent| {
            parent.spawn(SnakePartBundle::new(snake.index, snake.len() - 1));
        });
    }
}

fn despawn_snake_system(
    mut despawn_snake_event: EventReader<DespawnSnakeEvent>,
    mut level_instance: ResMut<LevelInstance>,
    mut commands: Commands,
    snakes_query: Query<(Entity, &Snake)>,
    parts_query: Query<(Entity, &SnakePart)>,
) {
    for message in despawn_snake_event.iter() {
        // Despawn snake.
        for (entity, snake) in snakes_query.iter() {
            if snake.index != message.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();

            for (position, _) in &snake.parts {
                level_instance.set_empty(*position);
            }
        }

        // Despawn parts
        for (entity, part) in parts_query.iter() {
            if part.snake_index != message.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_snake_parts_system(
    mut despawn_snake_event: EventReader<DespawnSnakePartsEvent>,
    mut commands: Commands,
    parts_query: Query<(Entity, &SnakePart)>,
) {
    for message in despawn_snake_event.iter() {
        // Despawn parts
        for (entity, part) in parts_query.iter() {
            if part.snake_index != message.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}

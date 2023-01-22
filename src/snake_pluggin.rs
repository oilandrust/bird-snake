use bevy::prelude::*;
use bevy_prototype_lyon::{
    prelude::{DrawMode, FillMode, GeometryBuilder, Path, PathBuilder, ShapePath, StrokeMode},
    shapes,
};
use bevy_tweening::{Animator, EaseFunction, Lens, Tween};
use std::collections::VecDeque;

use crate::{
    commands::SnakeCommands,
    game_constants_pluggin::{
        to_grid, to_world, GRID_TO_WORLD_UNIT, GRID_TO_WORLD_UNIT_INVERSE, SNAKE_COLORS,
        SNAKE_EYE_SIZE, SNAKE_SIZE,
    },
    level_instance::{LevelEntityType, LevelInstance},
    level_pluggin::{Food, LevelEntity},
    level_template::{LevelTemplate, SnakeTemplate},
    movement_pluggin::{update_sprite_positions_system, GravityFall, SnakeMovedEvent},
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
            .add_system_to_stage(CoreStage::PostUpdate, update_snake_mesh_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                despawn_snake_system.after(update_sprite_positions_system),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                despawn_snake_parts_system.after(update_sprite_positions_system),
            );
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
    spatial_bundle: SpatialBundle,
    part: SnakePart,
    level_entity: LevelEntity,
}

impl SnakePartBundle {
    fn new(position: IVec2, snake_index: i32, part_index: usize) -> Self {
        SnakePartBundle {
            spatial_bundle: SpatialBundle {
                transform: Transform {
                    translation: to_world(position).extend(0.0),
                    ..default()
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

#[derive(Bundle)]
struct SnakePartSpriteBundle {
    sprite_bundle: SpriteBundle,
    level_entity: LevelEntity,
    part_sprite: SnakePartSprite,
}

impl SnakePartSpriteBundle {
    fn new(scale: Vec2, size: Vec2, color: Color) -> Self {
        SnakePartSpriteBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform {
                    scale: scale.extend(1.0),
                    ..default()
                },

                ..default()
            },
            level_entity: LevelEntity,
            part_sprite: SnakePartSprite,
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

    pub fn head_direction(&self) -> IVec2 {
        self.parts.front().unwrap().1
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

    pub fn tail_direction(&self) -> IVec2 {
        self.parts.back().unwrap().1
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
    // for (index, part) in snake_template.iter().enumerate() {
    //     let mut part = commands.spawn(SnakePartBundle::new(part.0, snake_index, index));

    //     part.with_children(|parent| {
    //         parent.spawn(SnakePartSpriteBundle::new(
    //             Vec2::ONE,
    //             SNAKE_SIZE,
    //             SNAKE_COLORS[snake_index as usize],
    //         ));
    //     });

    //     if index == 0 {
    //         part.with_children(|parent| {
    //             parent.spawn((
    //                 SpriteBundle {
    //                     sprite: Sprite {
    //                         color: Color::BLACK,
    //                         custom_size: Some(SNAKE_EYE_SIZE),
    //                         ..default()
    //                     },
    //                     transform: Transform::from_xyz(5.0, 5.0, 1.0),
    //                     ..default()
    //                 },
    //                 LevelEntity,
    //             ));
    //         });
    //     }
    // }

    let shape = shapes::RegularPolygon {
        sides: 4,
        feature: shapes::RegularPolygonFeature::Radius(SNAKE_SIZE.x),
        ..shapes::RegularPolygon::default()
    };

    let spawn_command = commands.spawn((
        Snake {
            parts: VecDeque::from(snake_template.clone()),
            index: snake_index,
        },
        LevelEntity,
        Active,
        GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: FillMode::color(SNAKE_COLORS[snake_index as usize]),
                outline_mode: StrokeMode::new(Color::BLACK, 1.0),
            },
            Transform::default(),
        ),
    ));

    for (position, _) in snake_template {
        level_instance.mark_position_occupied(*position, LevelEntityType::Snake(snake_index));
    }

    spawn_command.id()
}

const FOWARD_LEFT: IVec2 = IVec2::new(1, 1);
const FOWARD_RIGHT: IVec2 = IVec2::new(1, -1);
const BACK_RIGHT: IVec2 = IVec2::new(-1, -1);
const BACK_LEFT: IVec2 = IVec2::new(-1, 1);

fn next_corner(corner: &IVec2) -> IVec2 {
    match *corner {
        FOWARD_LEFT => FOWARD_RIGHT,
        FOWARD_RIGHT => BACK_RIGHT,
        BACK_RIGHT => BACK_LEFT,
        BACK_LEFT => FOWARD_LEFT,
        _ => panic!("should not happen"),
    }
}

fn corner_position(corner: &IVec2, position: &IVec2, direction: &IVec2, ortho_dir: &IVec2) -> Vec2 {
    to_world(*position)
        + corner.x as f32 * 0.5 * GRID_TO_WORLD_UNIT * direction.as_vec2()
        + corner.y as f32 * 0.5 * GRID_TO_WORLD_UNIT * ortho_dir.as_vec2()
}

pub fn update_snake_mesh_system(mut query: Query<&mut Path>, snake_query: Query<&Snake>) {
    let snake = snake_query.single();

    let mut path = query.single_mut();
    let mut path_builder = PathBuilder::new();

    let mut current_corner = BACK_LEFT;
    let direction = snake.head_direction();
    let ortho_dir = IVec2::new(-direction.y, direction.x);
    let mut corner_world_position = corner_position(
        &current_corner,
        &snake.head_position(),
        &direction,
        &ortho_dir,
    );

    let forward_iter = snake.parts.iter().copied().zip(snake.parts.iter().skip(1));
    let backwards_iter = snake
        .parts
        .iter()
        .copied()
        .rev()
        .map(|(position, direction)| (position, -direction))
        .zip(snake.parts.iter().rev().skip(1));

    for (part, next_part) in forward_iter.chain(backwards_iter) {
        let (position, direction) = part;
        let ortho_dir = IVec2::new(-direction.y, direction.x);

        let corner_grid_pos =
            (corner_world_position - to_world(position)) * GRID_TO_WORLD_UNIT_INVERSE;

        current_corner = IVec2::new(
            (2.0 * corner_grid_pos.dot(direction.as_vec2())) as i32,
            (2.0 * corner_grid_pos.dot(ortho_dir.as_vec2())) as i32,
        );
        assert!(current_corner.abs() == IVec2::new(1, 1));

        let to_next = next_part.0 - position;
        let to_next_relative = IVec2::new(to_next.dot(direction), to_next.dot(ortho_dir));
        let is_next_part = |corner: IVec2| {
            corner.dot(to_next_relative) == 1 && next_corner(&corner).dot(to_next_relative) == 1
        };

        loop {
            path_builder.line_to(corner_world_position);
            if is_next_part(current_corner) {
                break;
            }

            current_corner = next_corner(&current_corner);
            corner_world_position =
                corner_position(&current_corner, &position, &direction, &ortho_dir);
        }
    }

    *path = path_builder.build();
}

pub fn set_snake_active(commands: &mut Commands, snake: &Snake, snake_entity: Entity) {
    for (index, part) in snake.parts().iter().enumerate() {
        commands
            .spawn(SnakePartBundle::new(part.0, snake.index(), index))
            .with_children(|parent| {
                parent.spawn(SnakePartSpriteBundle::new(
                    Vec2::ONE,
                    SNAKE_SIZE,
                    SNAKE_COLORS[snake.index() as usize],
                ));
            });
    }

    commands.entity(snake_entity).insert(Active);
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

        world_pos.truncate()
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
    snake_query: Query<&Snake, With<SelectedSnake>>,
    foods_query: Query<(Entity, &Food), With<Food>>,
) {
    if snake_moved_event.iter().next().is_none() {
        return;
    }

    let snake = snake_query.single();

    for (food_entity, food) in &foods_query {
        if food.0 != snake.head_position() {
            continue;
        }

        commands.entity(food_entity).despawn();
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

use std::collections::VecDeque;

use bevy::{app::AppExit, prelude::*, utils::HashMap};

use crate::{
    commands::SnakeCommands,
    game_constants_pluggin::{
        to_world, BRIGHT_COLOR_PALETTE, GRID_CELL_SIZE, GRID_TO_WORLD_UNIT, WALL_COLOR,
    },
    level_template::{Cell, LevelTemplate},
    levels::LEVELS,
    movement_pluggin::snake_movement_control_system,
    snake_pluggin::{Active, DespawnSnakePartsEvent, SelectedSnake, Snake, SpawnSnakeEvent},
    undo::{SnakeHistory, WalkableUpdateEvent},
};

pub struct StartLevelEvent(pub usize);
pub struct ClearLevelEvent;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Clone, Copy)]
pub struct Food(pub IVec2);

#[derive(Resource)]
pub struct CurrentLevelId(usize);

pub struct LevelPluggin;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Walkable {
    Food,
    Wall,
    Snake(i32),
}

#[derive(Resource)]
pub struct LevelInstance {
    walkable_positions: HashMap<IVec2, Walkable>,
}

impl LevelInstance {
    pub fn new() -> Self {
        LevelInstance {
            walkable_positions: HashMap::new(),
        }
    }

    pub fn walkable_positions(&self) -> &HashMap<IVec2, Walkable> {
        &self.walkable_positions
    }

    pub fn is_empty(&self, position: IVec2) -> bool {
        !self.walkable_positions.contains_key(&position)
    }

    pub fn set_empty(&mut self, position: IVec2) -> Option<Walkable> {
        self.walkable_positions.remove(&position)
    }

    pub fn mark_position_occupied(&mut self, position: IVec2, value: Walkable) {
        self.walkable_positions.insert(position, value);
    }

    pub fn is_food(&self, position: IVec2) -> bool {
        matches!(self.walkable_positions.get(&position), Some(Walkable::Food))
    }

    pub fn is_snake(&self, position: IVec2) -> Option<i32> {
        let walkable = self.walkable_positions.get(&position);
        match walkable {
            Some(Walkable::Snake(index)) => Some(*index),
            _ => None,
        }
    }

    /// Move a snake forward.
    /// Set the old tail location empty and mark the new head as occupied.
    /// Returns a list of updates to the walkable cells that can be undone.
    pub fn move_snake_forward(
        &mut self,
        snake: &Snake,
        direction: IVec2,
    ) -> Vec<WalkableUpdateEvent> {
        let mut updates: Vec<WalkableUpdateEvent> = Vec::with_capacity(2);
        let new_position = snake.head_position() + direction;

        let old_value = self.set_empty(snake.tail_position()).unwrap();
        self.mark_position_occupied(new_position, Walkable::Snake(snake.index()));

        updates.push(WalkableUpdateEvent::ClearPosition(
            snake.tail_position(),
            old_value,
        ));
        updates.push(WalkableUpdateEvent::FillPosition(new_position));

        updates
    }

    /// Move a snake by an offset:
    /// Set the old locations are empty and mark the new locations as occupied.
    /// Returns a list of updates to the walkable cells that can be undone.
    pub fn move_snake(&mut self, snake: &Snake, offset: IVec2) -> Vec<WalkableUpdateEvent> {
        let mut updates: VecDeque<WalkableUpdateEvent> = VecDeque::with_capacity(2 * snake.len());

        for (position, _) in snake.parts() {
            let old_value = self.set_empty(*position).unwrap();
            updates.push_front(WalkableUpdateEvent::ClearPosition(*position, old_value));
        }
        for (position, _) in snake.parts() {
            let new_position = *position + offset;
            self.mark_position_occupied(new_position, Walkable::Snake(snake.index()));
            updates.push_front(WalkableUpdateEvent::FillPosition(new_position));
        }

        updates.into()
    }

    pub fn eat_food(&mut self, position: IVec2) -> Vec<WalkableUpdateEvent> {
        let old_value = self.set_empty(position).unwrap();
        vec![WalkableUpdateEvent::ClearPosition(position, old_value)]
    }

    pub fn grow_snake(&mut self, snake: &Snake) -> Vec<WalkableUpdateEvent> {
        let (tail_position, tail_direction) = snake.tail();
        let new_part_position = tail_position - tail_direction;

        self.mark_position_occupied(new_part_position, Walkable::Snake(snake.index()));
        vec![WalkableUpdateEvent::FillPosition(new_part_position)]
    }

    pub fn clear_snake_positions(&mut self, snake: &Snake) -> Vec<WalkableUpdateEvent> {
        let mut updates: Vec<WalkableUpdateEvent> = Vec::with_capacity(snake.len());
        for (position, _) in snake.parts() {
            let old_value = self.set_empty(*position).unwrap();
            updates.push(WalkableUpdateEvent::ClearPosition(*position, old_value));
        }
        updates
    }

    pub fn mark_snake_positions(&mut self, snake: &Snake) -> Vec<WalkableUpdateEvent> {
        let mut updates: Vec<WalkableUpdateEvent> = Vec::with_capacity(snake.len());
        for (position, _) in snake.parts() {
            self.mark_position_occupied(*position, Walkable::Snake(snake.index()));
            updates.push(WalkableUpdateEvent::FillPosition(*position));
        }
        updates
    }

    pub fn undo_updates(&mut self, updates: &Vec<WalkableUpdateEvent>) {
        for update in updates {
            match update {
                WalkableUpdateEvent::ClearPosition(position, value) => {
                    self.mark_position_occupied(*position, *value);
                }
                WalkableUpdateEvent::FillPosition(position) => {
                    self.set_empty(*position);
                }
            }
        }
    }

    pub fn can_push_snake(&self, snake: &Snake, direction: IVec2) -> bool {
        snake.parts().iter().all(|(position, _)| {
            self.is_empty(*position + direction)
                || self.is_snake_with_index(*position + direction, snake.index())
        })
    }

    pub fn is_snake_with_index(&self, position: IVec2, snake_index: i32) -> bool {
        let walkable = self.walkable_positions.get(&position);
        match walkable {
            Some(Walkable::Snake(index)) => *index == snake_index,
            _ => false,
        }
    }

    pub fn is_wall(&self, position: IVec2) -> bool {
        matches!(self.walkable_positions.get(&position), Some(Walkable::Wall))
    }

    pub fn get_distance_to_ground(&self, position: IVec2, snake_index: i32) -> i32 {
        let mut distance = 1;

        const ARBITRARY_HIGH_DISTANCE: i32 = 50;

        let mut current_position = position + IVec2::NEG_Y;
        while self.is_empty(current_position)
            || self.is_snake_with_index(current_position, snake_index)
        {
            current_position += IVec2::NEG_Y;
            distance += 1;

            // There is no ground below.
            if current_position.y <= 0 {
                return ARBITRARY_HIGH_DISTANCE;
            }
        }

        distance
    }
}

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
    let level = LevelTemplate::parse(LEVELS[next_level_index]).unwrap();

    commands.insert_resource(SnakeHistory::default());

    commands.insert_resource(level);
    commands.insert_resource(LevelInstance::new());
    commands.insert_resource(CurrentLevelId(next_level_index));

    spawn_snake_event.send(SpawnSnakeEvent);
}

fn spawn_level_entities_system(
    mut commands: Commands,
    mut event_start_level: EventReader<StartLevelEvent>,
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

        level_instance.mark_position_occupied(position, Walkable::Wall);
    }

    // Spawn the food sprites.
    for position in &level_template.food_positions {
        spawn_food(&mut commands, position, &mut level_instance);
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
                0.0,
            ),
            ..default()
        })
        .insert(LevelEntity);
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

    level_instance.mark_position_occupied(*position, Walkable::Food);
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
    mut history: ResMut<SnakeHistory>,
    mut level_instance: ResMut<LevelInstance>,
    level: Res<LevelTemplate>,
    level_id: Res<CurrentLevelId>,
    mut event_start_level: EventWriter<StartLevelEvent>,
    mut event_clear_level: EventWriter<ClearLevelEvent>,
    mut event_despawn_snake_parts: EventWriter<DespawnSnakePartsEvent>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    selected_snake_query: Query<(Entity, &Snake), With<SelectedSnake>>,
    other_snakes_query: Query<Entity, (With<Snake>, Without<SelectedSnake>)>,
) {
    let (entity, snake) = selected_snake_query.single();

    if level.goal_position != snake.head_position() {
        return;
    }

    if let Some(next_snake_entity) = other_snakes_query.iter().next() {
        commands
            .entity(entity)
            .remove::<SelectedSnake>()
            .remove::<Active>();

        event_despawn_snake_parts.send(DespawnSnakePartsEvent(snake.index()));

        SnakeCommands::new(level_instance.as_mut(), history.as_mut()).exit_level(snake, entity);

        commands.entity(next_snake_entity).insert(SelectedSnake);
    } else if level_id.0 == LEVELS.len() - 1 {
        exit.send(AppExit);
    } else {
        event_clear_level.send(ClearLevelEvent);
        event_start_level.send(StartLevelEvent(level_id.0 + 1));
    }
}

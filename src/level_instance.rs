use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashMap};

use crate::{snake_pluggin::Snake, undo::LevelEntityUpdateEvent};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LevelEntityType {
    Food,
    Spike,
    Wall,
    Snake(i32),
}

#[derive(Resource)]
pub struct LevelInstance {
    occupied_cells: HashMap<IVec2, LevelEntityType>,
}

impl LevelInstance {
    pub fn new() -> Self {
        LevelInstance {
            occupied_cells: HashMap::new(),
        }
    }

    pub fn occupied_cells(&self) -> &HashMap<IVec2, LevelEntityType> {
        &self.occupied_cells
    }

    pub fn is_empty(&self, position: IVec2) -> bool {
        !self.occupied_cells.contains_key(&position)
    }

    pub fn is_empty_or_spike(&self, position: IVec2) -> bool {
        !self.occupied_cells.contains_key(&position) || self.is_spike(position)
    }

    pub fn set_empty(&mut self, position: IVec2) -> Option<LevelEntityType> {
        self.occupied_cells.remove(&position)
    }

    pub fn mark_position_occupied(&mut self, position: IVec2, value: LevelEntityType) {
        self.occupied_cells.insert(position, value);
    }

    pub fn is_food(&self, position: IVec2) -> bool {
        matches!(
            self.occupied_cells.get(&position),
            Some(LevelEntityType::Food)
        )
    }

    pub fn is_spike(&self, position: IVec2) -> bool {
        matches!(
            self.occupied_cells.get(&position),
            Some(LevelEntityType::Spike)
        )
    }

    pub fn is_snake(&self, position: IVec2) -> Option<i32> {
        let walkable = self.occupied_cells.get(&position);
        match walkable {
            Some(LevelEntityType::Snake(index)) => Some(*index),
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
    ) -> Vec<LevelEntityUpdateEvent> {
        let mut updates: Vec<LevelEntityUpdateEvent> = Vec::with_capacity(2);
        let new_position = snake.head_position() + direction;

        let old_value = self.set_empty(snake.tail_position()).unwrap();
        self.mark_position_occupied(new_position, LevelEntityType::Snake(snake.index()));

        updates.push(LevelEntityUpdateEvent::ClearPosition(
            snake.tail_position(),
            old_value,
        ));
        updates.push(LevelEntityUpdateEvent::FillPosition(new_position));

        updates
    }

    /// Move a snake by an offset:
    /// Set the old locations are empty and mark the new locations as occupied.
    /// Returns a list of updates to the walkable cells that can be undone.
    pub fn move_snake(&mut self, snake: &Snake, offset: IVec2) -> Vec<LevelEntityUpdateEvent> {
        let mut updates: VecDeque<LevelEntityUpdateEvent> =
            VecDeque::with_capacity(2 * snake.len());

        for (position, _) in snake.parts() {
            let old_value = self.set_empty(*position).unwrap();
            updates.push_front(LevelEntityUpdateEvent::ClearPosition(*position, old_value));
        }
        for (position, _) in snake.parts() {
            let new_position = *position + offset;
            self.mark_position_occupied(new_position, LevelEntityType::Snake(snake.index()));
            updates.push_front(LevelEntityUpdateEvent::FillPosition(new_position));
        }

        updates.into()
    }

    pub fn eat_food(&mut self, position: IVec2) -> Vec<LevelEntityUpdateEvent> {
        let old_value = self.set_empty(position).unwrap();
        vec![LevelEntityUpdateEvent::ClearPosition(position, old_value)]
    }

    pub fn grow_snake(&mut self, snake: &Snake) -> Vec<LevelEntityUpdateEvent> {
        let (tail_position, tail_direction) = snake.tail();
        let new_part_position = tail_position - tail_direction;

        self.mark_position_occupied(new_part_position, LevelEntityType::Snake(snake.index()));
        vec![LevelEntityUpdateEvent::FillPosition(new_part_position)]
    }

    pub fn clear_snake_positions(&mut self, snake: &Snake) -> Vec<LevelEntityUpdateEvent> {
        let mut updates: Vec<LevelEntityUpdateEvent> = Vec::with_capacity(snake.len());
        for (position, _) in snake.parts() {
            let old_value = self.set_empty(*position).unwrap();
            updates.push(LevelEntityUpdateEvent::ClearPosition(*position, old_value));
        }
        updates
    }

    pub fn mark_snake_positions(&mut self, snake: &Snake) -> Vec<LevelEntityUpdateEvent> {
        let mut updates: Vec<LevelEntityUpdateEvent> = Vec::with_capacity(snake.len());
        for (position, _) in snake.parts() {
            self.mark_position_occupied(*position, LevelEntityType::Snake(snake.index()));
            updates.push(LevelEntityUpdateEvent::FillPosition(*position));
        }
        updates
    }

    pub fn undo_updates(&mut self, updates: &Vec<LevelEntityUpdateEvent>) {
        for update in updates {
            match update {
                LevelEntityUpdateEvent::ClearPosition(position, value) => {
                    self.mark_position_occupied(*position, *value);
                }
                LevelEntityUpdateEvent::FillPosition(position) => {
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
        let walkable = self.occupied_cells.get(&position);
        match walkable {
            Some(LevelEntityType::Snake(index)) => *index == snake_index,
            _ => false,
        }
    }

    pub fn is_wall_or_spike(&self, position: IVec2) -> bool {
        matches!(
            self.occupied_cells.get(&position),
            Some(LevelEntityType::Wall)
        ) || matches!(
            self.occupied_cells.get(&position),
            Some(LevelEntityType::Spike)
        )
    }

    pub fn get_distance_to_ground(&self, position: IVec2, snake_index: i32) -> i32 {
        let mut distance = 1;

        const ARBITRARY_HIGH_DISTANCE: i32 = 50;

        let mut current_position = position + IVec2::NEG_Y;
        while self.is_empty_or_spike(current_position)
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

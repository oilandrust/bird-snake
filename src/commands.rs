use crate::{
    level_pluggin::{Food, LevelInstance},
    snake_pluggin::Snake,
    undo::{MoveHistoryEvent, SnakeHistory},
};
use bevy::prelude::*;

pub struct SnakeCommands<'a> {
    pub level_instance: &'a mut LevelInstance,
    pub history: &'a mut SnakeHistory,
}

impl<'a> SnakeCommands<'a> {
    pub fn new(level_instance: &'a mut LevelInstance, history: &'a mut SnakeHistory) -> Self {
        SnakeCommands {
            level_instance,
            history,
        }
    }

    pub fn player_move(&mut self, snake: &'a mut Snake, direction: IVec2) -> PlayerMoveCommand {
        PlayerMoveCommand {
            level_instance: self.level_instance,
            history: self.history,
            snake,
            other_snake: None,
            food: None,
            direction,
        }
    }

    pub fn start_falling(&mut self, snake: &'a Snake) {
        let updates = self.level_instance.clear_snake_positions(snake);

        self.history
            .push_with_updates(MoveHistoryEvent::BeginFall, snake.index(), updates);
    }

    pub fn stop_falling(&mut self, snake: &'a Snake, distance_fallen: i32) {
        let updates = self.level_instance.mark_snake_positions(snake);

        self.history.push_with_updates(
            MoveHistoryEvent::EndFall(distance_fallen),
            snake.index(),
            updates,
        );
    }
}

pub struct PlayerMoveCommand<'a> {
    level_instance: &'a mut LevelInstance,
    history: &'a mut SnakeHistory,
    snake: &'a mut Snake,
    other_snake: Option<(Entity, &'a mut Snake)>,
    food: Option<&'a Food>,
    direction: IVec2,
}

impl<'a> PlayerMoveCommand<'a> {
    pub fn pushing_snake(&mut self, other_snake: (Entity, &'a mut Snake)) -> &Self {
        self.other_snake = Some(other_snake);
        self
    }

    pub fn eating_food(&mut self, food: Option<&'a Food>) -> &Self {
        self.food = food;
        self
    }

    pub fn execute(&mut self) {
        // Push the player action marker.
        self.history
            .push(MoveHistoryEvent::PlayerSnakeMove, self.snake.index());

        // Move the other snake.
        if let Some((_, other_snake)) = &mut self.other_snake {
            let walkable_updates = self.level_instance.move_snake(other_snake, self.direction);

            other_snake.translate(self.direction);

            self.history.push_with_updates(
                MoveHistoryEvent::PassiveSnakeMove(self.direction),
                other_snake.index(),
                walkable_updates,
            );
        };

        // Consume food.
        if let Some(food) = &self.food {
            let walkable_updates = self.level_instance.eat_food(food.0);
            self.history.push_with_updates(
                MoveHistoryEvent::Eat(food.0),
                self.snake.index(),
                walkable_updates,
            );
        }

        // Then move the selected snake.
        let old_tail = self.snake.tail();
        let updates = self
            .level_instance
            .move_snake_forward(self.snake, self.direction);

        self.snake.move_forward(self.direction);

        self.history.push_with_updates(
            MoveHistoryEvent::SnakeMoveForward(old_tail),
            self.snake.index(),
            updates,
        );

        // Grow.
        if self.food.is_some() {
            let walkable_updates = self.level_instance.grow_snake(self.snake);
            self.snake.grow();

            self.history.push_with_updates(
                MoveHistoryEvent::Grow,
                self.snake.index(),
                walkable_updates,
            );
        }
    }
}

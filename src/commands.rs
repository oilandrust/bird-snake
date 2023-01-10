use crate::{
    level_pluggin::{Food, LevelInstance},
    movement_pluggin::GravityFall,
    snake_pluggin::Snake,
    undo::{EndFall, MoveHistoryEvent, SnakeHistory},
};
use bevy::prelude::*;

/// Provides commands that implement the undoable game mechanics.
/// Commands manage the state of the game data such as snakes, food, etc..
/// In addition they propagate the changes to the level instance that keep track of which object occupies which position.
/// Finaly, commands make sure that the changes are generate undoable instructions that can be executed by the undo system.
pub struct SnakeCommands<'a> {
    level_instance: &'a mut LevelInstance,
    history: &'a mut SnakeHistory,
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

    pub fn exit_level(&mut self, snake: &'a Snake, entity: Entity, falling: Option<&GravityFall>) {
        let updates = if falling.is_none() {
            self.level_instance.clear_snake_positions(snake)
        } else {
            vec![]
        };

        self.history
            .push_with_updates(MoveHistoryEvent::ExitLevel(entity), snake.index(), updates);
    }

    /// Execute a command when a skake start falling.
    pub fn start_falling(&mut self, snake: &'a Snake) {
        let updates = self.level_instance.clear_snake_positions(snake);

        self.history
            .push_with_updates(MoveHistoryEvent::BeginFall(None), snake.index(), updates);
    }

    pub fn stop_falling(&mut self, snake: &'a Snake, distance_fallen: i32) {
        let updates = self.level_instance.mark_snake_positions(snake);

        // Stop fall can happen a long time after beggin fall, and other actions can be done in between.
        // We find the corresponding beggin fall and add the undo info to it so that both can be undone at the same time.
        let begin_fall = self
            .history
            .move_history
            .iter_mut()
            .rev()
            .find(|event| {
                event.snake_index == snake.index()
                    && matches!(event.event, MoveHistoryEvent::BeginFall(None))
            })
            .unwrap();

        begin_fall.event = MoveHistoryEvent::BeginFall(Some(EndFall {
            distance_fallen,
            walkable_updates: updates,
        }));
    }
}

pub struct PlayerMoveCommand<'a> {
    level_instance: &'a mut LevelInstance,
    history: &'a mut SnakeHistory,
    snake: &'a mut Snake,
    other_snake: Option<&'a mut Snake>,
    food: Option<&'a Food>,
    direction: IVec2,
}

impl<'a> PlayerMoveCommand<'a> {
    pub fn pushing_snake(mut self, other_snake: Option<&'a mut Snake>) -> Self {
        self.other_snake = other_snake;
        self
    }

    pub fn eating_food(mut self, food: Option<&'a Food>) -> Self {
        self.food = food;
        self
    }

    pub fn execute(&mut self) {
        // Push the player action marker.
        self.history
            .push(MoveHistoryEvent::PlayerSnakeMove, self.snake.index());

        // Move the other snake.
        if let Some(other_snake) = &mut self.other_snake {
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

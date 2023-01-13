use std::iter::once;

use anyhow::{bail, Result};
use bevy::{prelude::*, utils::HashSet};
use game_grid::*;
use thiserror::Error;

#[derive(GridCell, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Cell {
    #[cell('#')]
    Wall,

    #[cell(' '|'.')]
    #[default]
    Empty,

    #[cell('o')]
    Food,

    #[cell('X')]
    Goal,

    #[cell('+')]
    Spike,

    #[cell('A'..='Z')]
    SnakeHead(char),

    #[cell('a'..='z')]
    SnakePart(char),
}

pub type SnakeElement = (IVec2, IVec2);
pub type SnakeTemplate = Vec<SnakeElement>;

#[derive(Debug, Clone, Resource)]
pub struct LevelTemplate {
    pub grid: Grid<Cell>,
    pub goal_position: IVec2,
    pub initial_snakes: Vec<SnakeTemplate>,
    pub food_positions: Vec<IVec2>,
    pub spike_positions: Vec<IVec2>,
}

#[derive(Debug, Error)]
enum ParseLevelError {
    #[error("Missing goal cell 'X'.")]
    MissingLevelGoal,

    #[error("Missing snake head start position 'A'..='Z'.")]
    MissingSnakeHead,

    #[error("Snake should be of length at least 2.")]
    InvalidSnake,
}

fn extract_snake_template(grid: &Grid<Cell>, start_head_index: usize) -> Result<SnakeTemplate> {
    let head_cell = grid[start_head_index];
    let start_head_position = grid.position_for_index(start_head_index);
    let Cell::SnakeHead(head_char) = head_cell else {
        panic!("Should not happen.");
    };

    let part_char = head_char
        .to_lowercase()
        .into_iter()
        .next()
        .expect("Snake head should be in the range 'A'..='Z' and have a valid lowercase.");

    // Search for the parts around the head.
    let mut parts = vec![start_head_position];
    {
        let mut visited = HashSet::<IVec2>::new();
        let mut current_position = start_head_position;
        let search_dirs = vec![IVec2::Y, IVec2::NEG_Y, IVec2::X, IVec2::NEG_X];

        while !visited.contains(&current_position)
            && (grid.cell_at(current_position) == Cell::SnakeHead(head_char)
                || grid.cell_at(current_position) == Cell::SnakePart(part_char))
        {
            visited.insert(current_position);
            for search_dir in search_dirs.iter() {
                let new_position = current_position + *search_dir;
                if visited.contains(&new_position) {
                    continue;
                }

                if grid.cell_at(new_position) == Cell::SnakePart(part_char) {
                    parts.push(new_position);
                    current_position = new_position;
                    break;
                }
            }
        }
    }

    if parts.len() < 2 {
        bail!(ParseLevelError::InvalidSnake);
    }

    // Infer parts direction from previous part.
    let directions = parts
        .iter()
        .zip(parts.iter().skip(1))
        .map(|(position, prev_position)| *position - *prev_position)
        .chain(once(parts[parts.len() - 2] - parts[parts.len() - 1]));

    let snake = parts.iter().copied().zip(directions).collect();

    Ok(snake)
}

impl LevelTemplate {
    pub fn parse(level_string: &str) -> Result<LevelTemplate> {
        let mut grid = level_string.parse::<Grid<Cell>>()?.flip_y();

        // Find and extract the snakes.
        let mut start_heads = grid
            .cells()
            .enumerate()
            .filter(|(_, &cell)| matches!(cell, Cell::SnakeHead(_)))
            .peekable();

        if start_heads.peek().is_none() {
            bail!(ParseLevelError::MissingSnakeHead);
        }

        let snakes: Vec<SnakeTemplate> = start_heads
            .map(|(start_head_index, _)| extract_snake_template(&grid, start_head_index))
            .collect::<Result<Vec<SnakeTemplate>>>()?;

        // Set the cells where the snakes are as empty, they are managed as part of the game state.
        for snake in &snakes {
            for part in snake {
                grid.set_cell(part.0, Cell::Empty);
            }
        }

        // Find the goal position.
        let goal_index = grid
            .cells()
            .position(|&cell| cell == Cell::Goal)
            .ok_or(ParseLevelError::MissingLevelGoal)?;

        let goal_position = grid.position_for_index(goal_index);

        grid.set_cell(goal_position, Cell::Empty);

        // Find the food positons.
        let food_positions: Vec<IVec2> = grid
            .iter()
            .filter(|(_, cell)| *cell == Cell::Food)
            .map(|(position, _)| position)
            .collect();

        // And set empty.
        for position in &food_positions {
            grid.set_cell(*position, Cell::Empty);
        }

        // Find the spikes positons.
        let spike_positions: Vec<IVec2> = grid
            .iter()
            .filter(|(_, cell)| *cell == Cell::Spike)
            .map(|(position, _)| position)
            .collect();

        // And set empty.
        for position in &spike_positions {
            grid.set_cell(*position, Cell::Empty);
        }

        Ok(LevelTemplate {
            grid,
            goal_position,
            initial_snakes: snakes,
            food_positions,
            spike_positions,
        })
    }
}

const LEVEL: &str = "
..A.....
#.aBbb..
#.aa..X.
#..#...";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_snake_extraction_complex() {
        let level = LevelTemplate::parse(LEVEL);
        assert!(level.is_ok());
        let level = level.unwrap();
        assert_eq!(level.initial_snakes.len(), 2);
        assert_eq!(level.initial_snakes[0].len(), 3);
        assert_eq!(level.initial_snakes[1].len(), 4);
    }
}

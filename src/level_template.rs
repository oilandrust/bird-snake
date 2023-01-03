use std::iter::once;

use anyhow::{bail, Result};
use bevy::{prelude::*, utils::HashSet};
use game_grid::*;
use thiserror::Error;

const LEVEL_1: &str = "....................
.............######.
.............####...
..............##....
......##.........#..
......#..#.....X.##.
.........##......##.
...aa@.......######.
..#################.
.###################
####################
####################";

const LEVEL_2: &str = "............
................
..X.............
............... 
...a@........... 
.###......o.....
####.......o...
.####.....o.o.
.###......####.
..##....#######
..#############
..#############";

const LEVEL_3: &str = "............
...............
...............
..a@........... 
..###..........
.#####...X....
.oooooo....
.....######.... 
......######...";

const LEVEL_4: &str = "............
......X.....
............
....#o#.....
..............
......@a.....
....####..o...
...#########.
..#####..####..
..####...####..";

pub const LEVELS: [&str; 4] = [LEVEL_4, LEVEL_2, LEVEL_3, LEVEL_4];

#[derive(GridCell, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Cell {
    #[cell('#')]
    Wall,

    #[cell(' '|'.')]
    #[default]
    Empty,

    #[cell('@')]
    SnakeHead,

    #[cell('a')]
    SnakePart,

    #[cell('o')]
    Food,

    #[cell('X')]
    Goal,
}

#[derive(Debug, Clone, Resource)]
pub struct LevelTemplate {
    pub grid: Grid<Cell>,
    pub goal_position: IVec2,
    pub initial_snake: Vec<(IVec2, IVec2)>,
    pub food_positions: Vec<IVec2>,
}

#[derive(Debug, Error)]
enum ParseLevelError {
    #[error("Missing goal cell 'X'.")]
    MissingLevelGoal,

    #[error("Missing snake head start position '@'.")]
    MissingSnakeHead,

    #[error("Snake should be of length at least 2.")]
    InvalidSnake,
}

impl LevelTemplate {
    pub fn parse(level_string: &str) -> Result<LevelTemplate> {
        let mut grid = level_string.parse::<Grid<Cell>>()?.flip_y();

        // Find the player start position.
        let start_head_index = grid
            .cells()
            .position(|&cell| cell == Cell::SnakeHead)
            .ok_or(ParseLevelError::MissingSnakeHead)?;

        let start_head_position = grid.position_for_index(start_head_index);

        // Search for the parts around the head.
        let mut parts = vec![start_head_position];
        {
            let mut visited = HashSet::<IVec2>::new();
            let mut current_position = start_head_position;
            let search_dirs = vec![IVec2::Y, IVec2::NEG_Y, IVec2::X, IVec2::NEG_X];

            while !visited.contains(&current_position)
                && (grid.cell_at(current_position) == Cell::SnakeHead
                    || grid.cell_at(current_position) == Cell::SnakePart)
            {
                visited.insert(current_position);
                for search_dir in search_dirs.iter() {
                    let new_position = current_position + *search_dir;
                    if visited.contains(&new_position) {
                        continue;
                    }

                    if grid.cell_at(new_position) == Cell::SnakePart {
                        parts.push(new_position);
                        current_position = new_position;
                        continue;
                    }
                }
            }
        }

        if parts.len() < 2 {
            bail!(ParseLevelError::InvalidSnake);
        }

        // Find the goal position.
        let goal_index = grid
            .cells()
            .position(|&cell| cell == Cell::Goal)
            .ok_or(ParseLevelError::MissingLevelGoal)?;

        let goal_position = grid.position_for_index(goal_index);

        // Set the cells where the player and goal are as empty, they are managed as part of the game state.
        for part in &parts {
            grid.set_cell(*part, Cell::Empty);
        }
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

        // Infer parts direction from previous part.
        let directions = parts
            .iter()
            .zip(parts.iter().skip(1))
            .map(|(position, prev_position)| *position - *prev_position)
            .chain(once(parts[parts.len() - 2] - parts[parts.len() - 1]));

        Ok(LevelTemplate {
            grid,
            goal_position,
            initial_snake: parts.iter().copied().zip(directions).collect(),
            food_positions,
        })
    }
}

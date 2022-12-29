use anyhow::Result;
use bevy::{prelude::*, utils::HashSet};
use game_grid::*;
use thiserror::Error;

const LEVEL_0: &str = "....................
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

const LEVEL_1: &str = "............
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

const LEVEL_2: &str = "............
...............
...............
..a@........... 
..###..........
.#####...X....
.oooooo....
.....######.... 
......######...";

const EAT_GYM: &str = "....................
.............######.
...X...o..oo.......
..........#.......
.........##.........
...aa@.o.o.o.o.
..#################.
.###################
####################
####################";

pub const LEVELS: [&str; 3] = [LEVEL_0, LEVEL_1, LEVEL_2];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Empty,
    SnakeHead,
    SnakePart,
    Food,
    Goal,
}

impl GridCell for Cell {
    const EMPTY: Self = Cell::Empty;
}

impl From<Cell> for char {
    fn from(cell: Cell) -> char {
        match cell {
            Cell::Wall => '#',
            Cell::Empty => ' ',
            Cell::SnakeHead => '@',
            Cell::SnakePart => 'a',
            Cell::Food => 'o',
            Cell::Goal => 'X',
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid character '{0}'")]
pub struct ParseCellError(char);

impl TryFrom<char> for Cell {
    type Error = ParseCellError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Cell::Wall),
            ' ' => Ok(Cell::Empty),
            '.' => Ok(Cell::Empty),
            '@' => Ok(Cell::SnakeHead),
            'a' => Ok(Cell::SnakePart),
            'o' => Ok(Cell::Food),
            'X' => Ok(Cell::Goal),
            _ => Err(ParseCellError(value)),
        }
    }
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

        // TODO: Infer direction from parts!
        Ok(LevelTemplate {
            grid,
            goal_position,
            initial_snake: parts.iter().map(|part| (*part, IVec2::X)).collect(),
            food_positions,
        })
    }
}

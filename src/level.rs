use bevy::{prelude::*, utils::HashSet};
use game_grid::*;

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
................ 
...a@........... 
.###............
####...........
.####.........
.###......####.
..##....#######
..#############
..#############";

pub const LEVELS: [&str; 2] = [LEVEL_0, LEVEL_1];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Empty,
    SnakeHead,
    SnakePart,
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
            Cell::Goal => 'X',
        }
    }
}

impl TryFrom<char> for Cell {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Cell::Wall),
            ' ' => Ok(Cell::Empty),
            '.' => Ok(Cell::Empty),
            '@' => Ok(Cell::SnakeHead),
            'a' => Ok(Cell::SnakePart),
            'X' => Ok(Cell::Goal),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Level {
    pub grid: Grid<Cell>,
    pub goal_position: IVec2,
    pub initial_snake: Vec<(IVec2, IVec2)>,
}

impl Level {
    pub fn get_distance_to_ground(&self, position: IVec2) -> i32 {
        let mut distance = 0;

        let mut current_position = position;
        while self.grid.cell_at(current_position) != Cell::Wall {
            current_position += IVec2::NEG_Y;
            distance += 1;
        }

        distance
    }

    pub fn parse(level_string: &str) -> Result<Level, String> {
        let mut grid = level_string.parse::<Grid<Cell>>()?.flip_y();

        // Find the player start position.
        let start_head_index = grid
            .cells()
            .position(|&cell| cell == Cell::SnakeHead)
            .ok_or_else(|| "Level is missing a snake head position.".to_string())?;

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

        // Find the player start position.
        let goal_index = grid
            .cells()
            .position(|&cell| cell == Cell::Goal)
            .ok_or_else(|| "Level is missing a goal position.".to_string())?;

        let goal_position = grid.position_for_index(goal_index);

        // Set the cells where the player and loads are as empty, they are managed as part of the game state.
        for part in &parts {
            grid.set_cell(*part, Cell::Empty);
        }
        grid.set_cell(goal_position, Cell::Empty);

        // TODO: Infer direction from parts!
        Ok(Level {
            grid,
            goal_position,
            initial_snake: parts.iter().map(|part| (*part, IVec2::X)).collect(),
        })
    }
}

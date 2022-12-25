use std::ops::Index;

use bevy::{prelude::*, utils::HashSet};

const LEVEL_0: &str = ".................... 
.............######.
.............####...
..............##....
......##.........#..
......#..#.......##.
.........##......##.
...aa@.......######.
..#################.
.###################
####################
####################";

pub const LEVELS: [&str; 1] = [LEVEL_0];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Empty,
    SnakeHead,
    SnakePart,
    Goal,
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
    pub grid: Grid,
    //pub goal_position: IVec2,
    pub initial_snake: Vec<(IVec2, IVec2)>,
}

#[derive(Debug, Clone)]
pub struct Grid {
    grid: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn cell_at(&self, position: IVec2) -> Cell {
        self.grid[position.x as usize + self.width * position.y as usize]
    }

    pub fn set_cell(&mut self, position: IVec2, value: Cell) {
        self.grid[position.x as usize + self.width * position.y as usize] = value;
    }

    pub fn is_empty(&self, position: IVec2) -> bool {
        let cell = self.cell_at(position);
        cell == Cell::Empty
    }

    pub fn print(&self) {
        let mut output_string = String::with_capacity(self.grid.len() + self.height);
        for line in self.grid.chunks(self.width) {
            output_string.extend(line.iter().map(|cell| char::from(*cell)));
            output_string.push('\n');
        }
        print!("{output_string}");
    }

    pub fn position_for_index(&self, index: usize) -> IVec2 {
        IVec2 {
            x: (index % self.width) as i32,
            y: (index / self.width) as i32,
        }
    }

    pub fn iter(&self) -> GridIter {
        GridIter {
            current: 0,
            grid: self,
        }
    }

    pub fn len(&self) -> usize {
        self.grid.len()
    }
}

impl Index<usize> for Grid {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index]
    }
}

pub struct GridIter<'a> {
    current: usize,
    grid: &'a Grid,
}

impl<'a> Iterator for GridIter<'a> {
    type Item = (Cell, IVec2);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.grid.len() {
            return None;
        }

        let result = (
            self.grid[self.current],
            self.grid.position_for_index(self.current),
        );

        self.current += 1;

        Some(result)
    }
}

pub fn parse_level(level_string: &str) -> Result<Level, String> {
    let mut lines: Vec<Vec<Cell>> = level_string
        .split('\n')
        .rev()
        .map(|line| {
            line.chars()
                .filter_map(|char| char.try_into().ok())
                .collect()
        })
        .collect();

    let width = lines
        .iter()
        .max_by_key(|line| line.len())
        .ok_or("Malformated level")?
        .len();

    let height = lines.len();

    for line in &mut lines {
        line.resize(width, Cell::Empty);
    }

    let grid: Vec<Cell> = lines.into_iter().flatten().collect();
    let mut grid = Grid {
        grid,
        width,
        height,
    };

    grid.print();

    // Find the player start position.
    let start_head_index = grid
        .grid
        .iter()
        .position(|&cell| cell == Cell::SnakeHead)
        .ok_or_else(|| "Level is missing a player position.".to_string())?;

    let start_head_position = IVec2 {
        x: (start_head_index % width) as i32,
        y: (start_head_index / width) as i32,
    };

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

    // Set the cells where the player and loads are as empty, they are managed as part of the game state.
    grid.grid[start_head_index] = Cell::Empty;

    // TODO: Infer direction from parts!
    Ok(Level {
        grid,
        initial_snake: parts.iter().map(|part| (*part, IVec2::X)).collect(),
    })
}
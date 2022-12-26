use core::slice::Iter;
use std::marker::PhantomData;
use std::ops::Index;
use std::{fmt::Display, str::FromStr};

pub use derive::GridCell;

/// Trait to implement a type that can be used as a grid cell.
pub trait GridCell: TryFrom<char> + Clone + Copy + PartialEq + Eq {
    /// Provide a value that is considered as an empty cell.
    const EMPTY: Self;
}

/// Trait to implement a type that can be used as a grid position.
pub trait Position {
    /// Construct a position from x and y coordinates.
    fn new(x: i32, y: i32) -> Self;

    /// Access the x coordinate of a position.
    fn x(&self) -> i32;

    /// Access the y coordinate of a position.
    fn y(&self) -> i32;
}

#[cfg(feature = "bevy-ivec2")]
use bevy::prelude::IVec2;

#[cfg(feature = "bevy-ivec2")]
impl Position for IVec2 {
    fn new(x: i32, y: i32) -> Self {
        IVec2::new(x, y)
    }

    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }
}

#[derive(Debug, Clone)]
pub struct Grid<Cell>
where
    Cell: GridCell,
{
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl<Cell> Grid<Cell>
where
    Cell: GridCell,
{
    pub fn cell_at<Point: Position>(&self, position: Point) -> Cell {
        self.cells[self.index_for_position(position)]
    }

    pub fn set_cell<Point: Position>(&mut self, position: Point, value: Cell) {
        let index = self.index_for_position(position);
        self.cells[index] = value;
    }

    pub fn is_empty<Point: Position>(&self, position: Point) -> bool {
        self.cell_at(position) == Cell::EMPTY
    }

    pub fn position_for_index<Point: Position>(&self, index: usize) -> Point {
        Point::new((index % self.width) as i32, (index / self.width) as i32)
    }

    pub fn index_for_position<Point: Position>(&self, position: Point) -> usize {
        position.x() as usize + self.width * position.y() as usize
    }

    pub fn cells(&self) -> Iter<'_, Cell> {
        self.cells.iter()
    }

    pub fn iter<Point: Position>(&self) -> GridIter<Cell, Point> {
        GridIter {
            current: 0,
            grid: self,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn flip_y(mut self) -> Self {
        self.cells = self
            .cells
            .chunks(self.width)
            .rev()
            .flatten()
            .map(|cell| *cell)
            .collect();
        self
    }
}

impl<Cell> Index<usize> for Grid<Cell>
where
    Cell: GridCell,
{
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl<Cell> Display for Grid<Cell>
where
    char: From<Cell>,
    Cell: GridCell,
{
    fn fmt(&self, formater: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_string = String::with_capacity(self.cells.len() + self.height);
        for line in self.cells.chunks(self.width) {
            output_string.extend(line.iter().map(|cell| char::from(*cell)));
            output_string.push('\n');
        }
        write!(formater, "{output_string}")
    }
}

pub struct GridIter<'a, Cell, Point>
where
    Cell: GridCell,
{
    current: usize,
    grid: &'a Grid<Cell>,
    phantom: PhantomData<Point>,
}

impl<'a, Cell, Point> Iterator for GridIter<'a, Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    type Item = (Point, Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.grid.len() {
            return None;
        }

        let result = (
            self.grid.position_for_index(self.current),
            self.grid[self.current],
        );

        self.current += 1;

        Some(result)
    }
}

impl<Cell> FromStr for Grid<Cell>
where
    Cell: GridCell,
{
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut lines: Vec<Vec<Cell>> = string
            .split('\n')
            .map(|line| {
                line.chars()
                    .filter_map(|char| char.try_into().ok())
                    .collect()
            })
            .collect();

        let width = lines
            .iter()
            .max_by_key(|line| line.len())
            .ok_or("Malformated grid, empty line")?
            .len();

        let height = lines.len();

        for line in &mut lines {
            line.resize(width, Cell::EMPTY);
        }

        let grid: Vec<Cell> = lines.into_iter().flatten().collect();
        Ok(Grid {
            cells: grid,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Cell {
        Wall,
        Empty,
    }

    impl GridCell for Cell {
        const EMPTY: Self = Cell::Empty;
    }

    impl From<Cell> for char {
        fn from(cell: Cell) -> char {
            match cell {
                Cell::Wall => '#',
                Cell::Empty => ' ',
            }
        }
    }

    impl TryFrom<char> for Cell {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '#' => Ok(Cell::Wall),
                ' ' => Ok(Cell::Empty),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn test_parse_grid() {
        // Empty string.
        let result = "".parse::<Grid<Cell>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.len() == 0);

        // Wrong character is error.
        let result = "a".parse::<Grid<Cell>>();
        assert!(!result.is_ok());
    }
}

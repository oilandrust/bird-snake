use core::slice::Iter;
use std::ops::Index;
use std::{fmt::Display, str::FromStr};

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
pub struct Grid<Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    cells: Vec<Cell>,
    width: usize,
    height: usize,

    // TODO: Remove that.
    _p: Point,
}

impl<Cell, Point> Grid<Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    pub fn cell_at(&self, position: Point) -> Cell {
        self.cells[self.index_for_position(position)]
    }

    pub fn set_cell(&mut self, position: Point, value: Cell) {
        let index = self.index_for_position(position);
        self.cells[index] = value;
    }

    pub fn is_empty(&self, position: Point) -> bool {
        self.cell_at(position) == Cell::EMPTY
    }

    pub fn position_for_index(&self, index: usize) -> Point {
        Point::new((index % self.width) as i32, (index / self.width) as i32)
    }

    pub fn index_for_position(&self, position: Point) -> usize {
        position.x() as usize + self.width * position.y() as usize
    }

    pub fn cells(&self) -> Iter<'_, Cell> {
        self.cells.iter()
    }

    pub fn iter(&self) -> GridIter<Cell, Point> {
        GridIter {
            current: 0,
            grid: self,
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
}

impl<Cell, Point> Index<usize> for Grid<Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl<Cell, Point> Display for Grid<Cell, Point>
where
    char: From<Cell>,
    Cell: GridCell,
    Point: Position,
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
    Point: Position,
{
    current: usize,
    grid: &'a Grid<Cell, Point>,
}

impl<'a, Cell, Point> Iterator for GridIter<'a, Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    type Item = (Cell, Point);

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

impl<Cell, Point> FromStr for Grid<Cell, Point>
where
    Cell: GridCell,
    Point: Position,
{
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut lines: Vec<Vec<Cell>> = string
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
            _p: Point::new(0, 0),
        })
    }
}

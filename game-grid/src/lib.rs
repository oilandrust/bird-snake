//! # Game Grid
//!
//! `game_grid` provides a simple 2D grid that can be used to prototype games.
//!
//! The main struct is `Grid` that implements a grid able to contain values of a user `Cell` type.
//! The user cell must implement the `GameGrid` trait that provide an `EMPTY` value as well as conversion from and to `char`.
//! `Grid` provides access to the cells with 2D indexing with user types that implement the `GridPosition` trait.
//! On top of that `Grid` provide iterators, parsing form string, printing as well as other utilities.
//!
//! # Examples:
//!
//! ```
//! use game_grid::*;
//! // A custom Cell type.
//! #[derive(Copy, Clone, Debug, PartialEq, Eq)]
//! enum Cell {
//!     Empty,
//!     Wall,
//!     Food,
//! }
//! // Implement the GridCell trait.
//! impl GridCell for Cell {
//!     const EMPTY: Self = Cell::Empty;
//! }
//! // Implement cell char conversion to enable printing a grid.
//! impl From<Cell> for char {
//!     fn from(cell: Cell) -> char {
//!         match cell {
//!             Cell::Wall => '#',
//!             Cell::Empty => ' ',
//!             Cell::Food => 'o',
//!         }
//!     }
//! }
//! // Implement char to cell conversion to enable parsing a grid from a string.
//! impl TryFrom<char> for Cell {
//!     type Error = ();
//!     fn try_from(value: char) -> Result<Self, Self::Error> {
//!         match value {
//!             '#' => Ok(Cell::Wall),
//!             ' ' => Ok(Cell::Empty),
//!             'o' => Ok(Cell::Food),
//!             _ => Err(()),
//!         }
//!     }
//! }
//! // A 2D point struct.
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//! // Implement the GridPosition struct to be able to index into the grid with our points.
//! impl GridPosition for Point {
//!     fn new(x: i32, y: i32) -> Self {
//!         Self { x, y }
//!     }
//!     fn x(&self) -> i32 {
//!         self.x
//!     }
//!     fn y(&self) -> i32 {
//!         self.y
//!     }
//! }
//! fn main() {
//!     let grid: Grid<Cell> = "####\n# o#\n####".parse().unwrap();
//!
//!     let food_position = Point { x: 2, y: 1 };
//!     if grid.cell_at(food_position) == Cell::Food {
//!         println!("Found the food!");
//!     }
//!
//!     let as_string = grid.to_string();
//!
//!     print!("{grid}");
//!     // outputs:
//!     // ####
//!     // # o#
//!     // ####
//! }
//! ```
use core::slice::Iter;
use std::marker::PhantomData;
use std::ops::Index;
use std::slice::IterMut;
use std::{fmt::Display, str::FromStr};

pub use derive::GridCell;

/// Trait to implement a type that can be used as a grid cell.
pub trait GridCell: TryFrom<char> + Clone + Copy + PartialEq + Eq {
    /// Provide a value that is considered as an empty cell.
    const EMPTY: Self;
}

/// Implementation of GridCell for char.
impl GridCell for char {
    const EMPTY: Self = ' ';
}

/// Implementation for Option<T>
impl<T> GridCell for Option<T>
where
    Option<T>: From<char>,
    T: Copy + PartialEq + Eq,
{
    const EMPTY: Self = None;
}

/// Trait to implement a type that can be used as a grid position.
pub trait GridPosition {
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
impl GridPosition for IVec2 {
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

/// A struct maintaining a grid usable for game prototyping.
/// The grid is represented as a linear vector containing cells and Grid provides
/// functions to look up and write to the grid with 2-dimentional vector types implementing the trait
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
    // TODO: test and make sure size is correct!
    pub fn from_slice(width: usize, data: &[Cell]) -> Self {
        Self {
            cells: data.into(),
            width: width,
            height: data.len() / width,
        }
    }

    /// Get the cell value at some position.
    pub fn cell_at<Point: GridPosition>(&self, position: Point) -> Cell {
        self.cells[self.index_for_position(position)]
    }

    /// Set the cell value at some position.
    pub fn set_cell<Point: GridPosition>(&mut self, position: Point, value: Cell) {
        let index = self.index_for_position(position);
        self.cells[index] = value;
    }

    /// Check whether a cell at a position as empty value.
    pub fn is_empty<Point: GridPosition>(&self, position: Point) -> bool {
        self.cell_at(position) == Cell::EMPTY
    }

    /// Get the 2D position for an index in the linear array.
    pub fn position_for_index<Point: GridPosition>(&self, index: usize) -> Point {
        Point::new((index % self.width) as i32, (index / self.width) as i32)
    }

    /// Get the index in the linear array for a 2D position.
    pub fn index_for_position<Point: GridPosition>(&self, position: Point) -> usize {
        position.x() as usize + self.width * position.y() as usize
    }

    /// An iterator visiting the cells in order of memory.
    pub fn cells(&self) -> Iter<'_, Cell> {
        self.cells.iter()
    }

    /// An iterator visiting the cells mutably in order of memory.
    pub fn mut_cells(&mut self) -> IterMut<'_, Cell> {
        self.cells.iter_mut()
    }

    /// An iterator visiting the cell and associated position in the grid.
    pub fn iter<Point: GridPosition>(&self) -> GridIter<Cell, Point> {
        GridIter {
            current: 0,
            grid: self,
            phantom: PhantomData,
        }
    }

    /// Returns the number of cells in the grid.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns the width of the grid.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the grid.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Flips the order of the lines vertically. Useful when the game's y axis is upwards.
    /// # Example:
    /// ```
    /// use game_grid::Grid;
    ///
    /// let string_grid = "aaa
    /// bbb
    /// ccc";
    ///
    /// let grid = string_grid.parse::<Grid<char>>().unwrap().flip_y();
    ///
    /// let string_grid_flipped = "ccc
    /// bbb
    /// aaa";
    ///
    /// assert_eq!(grid.to_string(), string_grid_flipped);
    /// ```
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

impl<Cell, Point: GridPosition> Index<Point> for Grid<Cell>
where
    Cell: GridCell,
{
    type Output = Cell;

    fn index(&self, position: Point) -> &Self::Output {
        &self.cells[self.index_for_position(position)]
    }
}

impl<Cell> Display for Grid<Cell>
where
    char: From<Cell>,
    Cell: GridCell,
{
    fn fmt(&self, formater: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_string = String::with_capacity(self.cells.len() + (self.height - 1));
        for (index, line) in self.cells.chunks(self.width).enumerate() {
            output_string.extend(line.iter().map(|cell| char::from(*cell)));
            if index != self.height - 1 {
                output_string.push('\n');
            }
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
    Point: GridPosition,
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

    // Using an enum.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Cell {
        Wall(i32),
        Empty,
    }

    impl GridCell for Cell {
        const EMPTY: Self = Cell::Empty;
    }

    impl From<Cell> for char {
        fn from(cell: Cell) -> char {
            match cell {
                Cell::Wall(_) => '#',
                Cell::Empty => ' ',
            }
        }
    }

    impl TryFrom<char> for Cell {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '#' => Ok(Cell::Wall(0)),
                ' ' => Ok(Cell::Empty),
                _ => Err(()),
            }
        }
    }

    // A 2D point struct.
    struct Point {
        x: i32,
        y: i32,
    }
    // Implement the GridPosition struct to be able to index into the grid with our points.
    impl GridPosition for Point {
        fn new(x: i32, y: i32) -> Self {
            Self { x, y }
        }
        fn x(&self) -> i32 {
            self.x
        }
        fn y(&self) -> i32 {
            self.y
        }
    }

    #[test]
    fn test_char_grid() {
        // Valid input.
        let result = "abc".parse::<Grid<char>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.to_string(), "abc");
    }

    #[test]
    fn test_struct_grid() {
        // Using a stuct.
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct StructCell {
            c: char,
        }

        impl GridCell for StructCell {
            const EMPTY: Self = StructCell { c: ' ' };
        }

        impl From<StructCell> for char {
            fn from(cell: StructCell) -> char {
                cell.c
            }
        }

        impl TryFrom<char> for StructCell {
            type Error = ();

            fn try_from(value: char) -> Result<Self, Self::Error> {
                Ok(StructCell { c: value })
            }
        }

        // Valid input.
        let result = "abc".parse::<Grid<StructCell>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.to_string(), "abc");
    }

    #[test]
    fn test_option_grid() {
        // TODO!
        // Valid input.
        // let result = "1 0".parse::<Grid<Option<i32>>>();
        // assert!(result.is_ok());
        // let result = result.unwrap();
        // assert_eq!(result.to_string(), "1 0");
        // assert!(resut.is_empty(Point { x: 1, y: 0 }));
    }

    #[test]
    fn test_enum_grid() {
        // Empty string.
        let result = "".parse::<Grid<Cell>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.len() == 0);

        // Valid input.
        let result = "## #".parse::<Grid<Cell>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.to_string(), "## #");

        // Wrong character is error.
        let result = "a".parse::<Grid<Cell>>();
        assert!(!result.is_ok());
    }

    #[test]
    fn test_indicing() {
        let grid: Grid<char> = Grid::from_slice(2, &['a', 'b', 'c', 'd']);
        assert_eq!(grid[0], 'a');
        assert_eq!(grid[Point::new(0, 0)], 'a');
    }
}

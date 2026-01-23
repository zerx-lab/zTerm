//! Terminal cell representation

use std::fmt;

/// Flags for cell styling
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub blink: bool,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Default foreground or background
    Default,
    /// Named color (0-7 for standard, 8-15 for bright)
    Named(u8),
    /// 256-color palette index
    Indexed(u8),
    /// True color RGB
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

/// A single cell in the terminal grid
#[derive(Clone, PartialEq, Eq)]
pub struct Cell {
    /// The character in this cell
    pub c: char,
    /// Foreground color
    pub fg: Color,
    /// Background color
    pub bg: Color,
    /// Cell styling flags
    pub flags: CellFlags,
    /// Width of the character (1 or 2 for wide chars)
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::Default,
            bg: Color::Default,
            flags: CellFlags::default(),
            width: 1,
        }
    }
}

impl Cell {
    /// Create a new cell with the given character
    pub fn new(c: char) -> Self {
        Self {
            c,
            ..Default::default()
        }
    }

    /// Create a new cell with character and colors
    pub fn with_colors(c: char, fg: Color, bg: Color) -> Self {
        Self {
            c,
            fg,
            bg,
            ..Default::default()
        }
    }

    /// Reset the cell to default state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check if this is an empty cell
    pub fn is_empty(&self) -> bool {
        self.c == ' ' && self.fg == Color::Default && self.bg == Color::Default
    }

    /// Check if this is a wide character placeholder
    pub fn is_wide_continuation(&self) -> bool {
        self.width == 0
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cell({:?})", self.c)
    }
}

//! Terminal grid (screen buffer)

use super::Cell;

/// A row in the terminal grid
#[derive(Clone)]
pub struct Row {
    cells: Vec<Cell>,
}

impl Row {
    /// Create a new row with the given width
    pub fn new(cols: usize) -> Self {
        Self {
            cells: vec![Cell::default(); cols],
        }
    }

    /// Get the number of columns
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if the row is empty
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Get a cell at the given column
    pub fn get(&self, col: usize) -> Option<&Cell> {
        self.cells.get(col)
    }

    /// Get a mutable cell at the given column
    pub fn get_mut(&mut self, col: usize) -> Option<&mut Cell> {
        self.cells.get_mut(col)
    }

    /// Resize the row
    pub fn resize(&mut self, cols: usize) {
        self.cells.resize(cols, Cell::default());
    }

    /// Clear the row
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    /// Iterate over cells
    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }
}

impl std::ops::Index<usize> for Row {
    type Output = Cell;

    fn index(&self, col: usize) -> &Self::Output {
        &self.cells[col]
    }
}

impl std::ops::IndexMut<usize> for Row {
    fn index_mut(&mut self, col: usize) -> &mut Self::Output {
        &mut self.cells[col]
    }
}

/// The terminal grid (screen buffer)
pub struct Grid {
    /// Rows in the visible screen
    rows: Vec<Row>,
    /// Number of columns
    cols: usize,
    /// Number of visible rows
    num_rows: usize,
    /// Cursor column position
    cursor_col: usize,
    /// Cursor row position
    cursor_row: usize,
}

impl Grid {
    /// Create a new grid with the given dimensions
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid_rows = (0..rows).map(|_| Row::new(cols)).collect();

        Self {
            rows: grid_rows,
            cols,
            num_rows: rows,
            cursor_col: 0,
            cursor_row: 0,
        }
    }

    /// Get the number of columns
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get the number of rows
    pub fn rows(&self) -> usize {
        self.num_rows
    }

    /// Get the cursor position
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    /// Set the cursor position
    pub fn set_cursor(&mut self, col: usize, row: usize) {
        self.cursor_col = col.min(self.cols.saturating_sub(1));
        self.cursor_row = row.min(self.num_rows.saturating_sub(1));
    }

    /// Get a row at the given index
    pub fn get_row(&self, row: usize) -> Option<&Row> {
        self.rows.get(row)
    }

    /// Get a mutable row at the given index
    pub fn get_row_mut(&mut self, row: usize) -> Option<&mut Row> {
        self.rows.get_mut(row)
    }

    /// Get a cell at the given position
    pub fn get_cell(&self, col: usize, row: usize) -> Option<&Cell> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    /// Get a mutable cell at the given position
    pub fn get_cell_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        self.rows.get_mut(row).and_then(|r| r.get_mut(col))
    }

    /// Write a character at the cursor position
    pub fn write_char(&mut self, c: char) {
        if let Some(cell) = self.get_cell_mut(self.cursor_col, self.cursor_row) {
            cell.c = c;
        }
        self.cursor_col += 1;
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.num_rows {
                self.scroll_up(1);
                self.cursor_row = self.num_rows - 1;
            }
        }
    }

    /// Scroll the grid up by the given number of lines
    pub fn scroll_up(&mut self, lines: usize) {
        if lines >= self.num_rows {
            self.clear();
            return;
        }

        // Remove top lines and add new empty lines at bottom
        self.rows.drain(0..lines);
        for _ in 0..lines {
            self.rows.push(Row::new(self.cols));
        }
    }

    /// Scroll the grid down by the given number of lines
    pub fn scroll_down(&mut self, lines: usize) {
        if lines >= self.num_rows {
            self.clear();
            return;
        }

        // Remove bottom lines and add new empty lines at top
        self.rows.truncate(self.num_rows - lines);
        for _ in 0..lines {
            self.rows.insert(0, Row::new(self.cols));
        }
    }

    /// Clear the entire grid
    pub fn clear(&mut self) {
        for row in &mut self.rows {
            row.clear();
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    /// Resize the grid
    pub fn resize(&mut self, cols: usize, rows: usize) {
        // Resize existing rows
        for row in &mut self.rows {
            row.resize(cols);
        }

        // Add or remove rows
        if rows > self.num_rows {
            for _ in self.num_rows..rows {
                self.rows.push(Row::new(cols));
            }
        } else if rows < self.num_rows {
            self.rows.truncate(rows);
        }

        self.cols = cols;
        self.num_rows = rows;

        // Clamp cursor position
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
    }

    /// Iterate over all rows
    pub fn iter_rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// Convert visible content to a string (for debugging)
    pub fn to_string_content(&self) -> String {
        let mut result = String::new();
        for row in &self.rows {
            for cell in row.iter() {
                result.push(cell.c);
            }
            result.push('\n');
        }
        result
    }
}

impl std::ops::Index<usize> for Grid {
    type Output = Row;

    fn index(&self, row: usize) -> &Self::Output {
        &self.rows[row]
    }
}

impl std::ops::IndexMut<usize> for Grid {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        &mut self.rows[row]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(80, 24);
        assert_eq!(grid.cols(), 80);
        assert_eq!(grid.rows(), 24);
    }

    #[test]
    fn test_grid_write() {
        let mut grid = Grid::new(10, 5);
        grid.write_char('H');
        grid.write_char('i');
        assert_eq!(grid.get_cell(0, 0).unwrap().c, 'H');
        assert_eq!(grid.get_cell(1, 0).unwrap().c, 'i');
    }

    #[test]
    fn test_grid_resize() {
        let mut grid = Grid::new(80, 24);
        grid.resize(100, 30);
        assert_eq!(grid.cols(), 100);
        assert_eq!(grid.rows(), 30);
    }

    #[test]
    fn test_grid_cursor_initial_position() {
        let grid = Grid::new(80, 24);
        assert_eq!(grid.cursor(), (0, 0));
    }

    #[test]
    fn test_grid_set_cursor() {
        let mut grid = Grid::new(80, 24);
        grid.set_cursor(10, 5);
        assert_eq!(grid.cursor(), (10, 5));
    }

    #[test]
    fn test_grid_set_cursor_clamps_to_bounds() {
        let mut grid = Grid::new(10, 5);
        grid.set_cursor(100, 100);
        assert_eq!(grid.cursor(), (9, 4)); // clamped to max valid position
    }

    #[test]
    fn test_grid_write_wraps_line() {
        let mut grid = Grid::new(3, 2);
        grid.write_char('a');
        grid.write_char('b');
        grid.write_char('c');
        // After writing 3 chars in a 3-column grid, cursor wraps to next line
        assert_eq!(grid.cursor(), (0, 1));
    }

    #[test]
    fn test_grid_write_scrolls_at_bottom() {
        let mut grid = Grid::new(2, 2);
        grid.write_char('a');
        grid.write_char('b');
        grid.write_char('c');
        grid.write_char('d');
        // After writing 4 chars in 2x2 grid:
        // 1. 'a' at (0,0), 'b' at (1,0) -> wraps to row 1
        // 2. 'c' at (0,1), 'd' at (1,1) -> wraps and triggers scroll
        // After scroll: row 0 now has 'c', 'd', row 1 is empty
        assert_eq!(grid.get_cell(0, 0).unwrap().c, 'c');
        assert_eq!(grid.get_cell(1, 0).unwrap().c, 'd');
    }

    #[test]
    fn test_grid_scroll_up() {
        let mut grid = Grid::new(3, 3);
        grid.write_char('A');
        grid.set_cursor(0, 1);
        grid.write_char('B');
        grid.set_cursor(0, 2);
        grid.write_char('C');

        grid.scroll_up(1);
        // Row 0 should now have 'B', row 1 should have 'C', row 2 is empty
        assert_eq!(grid.get_cell(0, 0).unwrap().c, 'B');
        assert_eq!(grid.get_cell(0, 1).unwrap().c, 'C');
    }

    #[test]
    fn test_grid_scroll_up_all() {
        let mut grid = Grid::new(3, 3);
        grid.write_char('X');
        grid.scroll_up(10); // scroll more than rows
        // Grid should be cleared
        assert_eq!(grid.cursor(), (0, 0));
    }

    #[test]
    fn test_grid_scroll_down() {
        let mut grid = Grid::new(3, 3);
        grid.set_cursor(0, 0);
        grid.write_char('A');
        grid.set_cursor(0, 1);
        grid.write_char('B');

        grid.scroll_down(1);
        // Row 0 is now empty, 'A' moved to row 1
        assert_eq!(grid.get_cell(0, 0).unwrap().c, ' ');
        assert_eq!(grid.get_cell(0, 1).unwrap().c, 'A');
    }

    #[test]
    fn test_grid_clear() {
        let mut grid = Grid::new(3, 3);
        grid.write_char('X');
        grid.write_char('Y');
        grid.clear();
        assert_eq!(grid.cursor(), (0, 0));
        assert_eq!(grid.get_cell(0, 0).unwrap().c, ' ');
    }

    #[test]
    fn test_grid_get_row() {
        let grid = Grid::new(10, 5);
        assert!(grid.get_row(0).is_some());
        assert!(grid.get_row(4).is_some());
        assert!(grid.get_row(5).is_none());
    }

    #[test]
    fn test_grid_index_operators() {
        let mut grid = Grid::new(10, 5);
        grid[0][0].c = 'Z';
        assert_eq!(grid[0][0].c, 'Z');
    }

    #[test]
    fn test_grid_resize_smaller() {
        let mut grid = Grid::new(80, 24);
        grid.set_cursor(50, 20);
        grid.resize(40, 10);
        assert_eq!(grid.cols(), 40);
        assert_eq!(grid.rows(), 10);
        // Cursor should be clamped
        assert_eq!(grid.cursor(), (39, 9));
    }

    #[test]
    fn test_grid_to_string_content() {
        let mut grid = Grid::new(3, 2);
        grid.write_char('H');
        grid.write_char('i');
        let content = grid.to_string_content();
        assert!(content.starts_with("Hi "));
    }

    // Row tests
    #[test]
    fn test_row_creation() {
        let row = Row::new(10);
        assert_eq!(row.len(), 10);
        assert!(!row.is_empty());
    }

    #[test]
    fn test_row_empty() {
        let row = Row::new(0);
        assert!(row.is_empty());
        assert_eq!(row.len(), 0);
    }

    #[test]
    fn test_row_get() {
        let row = Row::new(5);
        assert!(row.get(0).is_some());
        assert!(row.get(4).is_some());
        assert!(row.get(5).is_none());
    }

    #[test]
    fn test_row_get_mut() {
        let mut row = Row::new(5);
        if let Some(cell) = row.get_mut(0) {
            cell.c = 'A';
        }
        assert_eq!(row.get(0).unwrap().c, 'A');
    }

    #[test]
    fn test_row_resize() {
        let mut row = Row::new(5);
        row.resize(10);
        assert_eq!(row.len(), 10);
        row.resize(3);
        assert_eq!(row.len(), 3);
    }

    #[test]
    fn test_row_clear() {
        let mut row = Row::new(3);
        row[0].c = 'X';
        row[1].c = 'Y';
        row.clear();
        assert_eq!(row[0].c, ' ');
        assert_eq!(row[1].c, ' ');
    }

    #[test]
    fn test_row_iter() {
        let row = Row::new(3);
        let count = row.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_row_index_operators() {
        let mut row = Row::new(5);
        row[2].c = 'M';
        assert_eq!(row[2].c, 'M');
    }
}

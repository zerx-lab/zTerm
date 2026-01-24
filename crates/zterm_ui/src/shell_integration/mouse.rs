//! Mouse interaction handling for shell integration
//!
//! This module provides utilities for handling mouse interactions with
//! command zones, including hover detection and click handling.

use gpui::Point;

/// Configuration for mouse interaction areas
#[derive(Debug, Clone)]
pub struct MouseConfig {
    /// Width of the gutter area (for command status marks)
    pub gutter_width: f32,
    /// Height of each line in pixels
    pub line_height: f32,
    /// Width of each character in pixels (for monospace fonts)
    pub char_width: f32,
    /// Vertical scroll offset
    pub scroll_offset: f32,
    /// First visible line in scrollback
    pub first_visible_line: usize,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            gutter_width: 20.0,
            line_height: 20.0,
            char_width: 8.0,
            scroll_offset: 0.0,
            first_visible_line: 0,
        }
    }
}

/// Handles mouse position to terminal line/column conversion
#[derive(Debug, Clone)]
pub struct MouseHandler {
    config: MouseConfig,
}

impl MouseHandler {
    /// Create a new mouse handler with the given configuration
    pub fn new(config: MouseConfig) -> Self {
        Self { config }
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: MouseConfig) {
        self.config = config;
    }

    /// Convert screen coordinates to terminal line number
    ///
    /// Returns None if the position is outside the terminal area (e.g., in gutter)
    pub fn screen_to_line(&self, pos: Point<f32>) -> Option<usize> {
        if pos.y < 0.0 {
            return None;
        }

        let adjusted_y = pos.y + self.config.scroll_offset;
        let line_offset = (adjusted_y / self.config.line_height) as usize;

        Some(self.config.first_visible_line.saturating_add(line_offset))
    }

    /// Convert screen coordinates to terminal column number
    ///
    /// Returns None if the position is in the gutter area
    pub fn screen_to_column(&self, pos: Point<f32>) -> Option<usize> {
        if pos.x < self.config.gutter_width {
            return None; // In gutter
        }

        let x_in_terminal = pos.x - self.config.gutter_width;
        if x_in_terminal < 0.0 {
            return None;
        }

        Some((x_in_terminal / self.config.char_width) as usize)
    }

    /// Convert screen coordinates to (line, column) tuple
    pub fn screen_to_cell(&self, pos: Point<f32>) -> Option<(usize, usize)> {
        let line = self.screen_to_line(pos)?;
        let col = self.screen_to_column(pos)?;
        Some((line, col))
    }

    /// Check if the position is in the gutter area
    pub fn is_in_gutter(&self, pos: Point<f32>) -> bool {
        pos.x >= 0.0 && pos.x < self.config.gutter_width && pos.y >= 0.0
    }

    /// Get the line number for a gutter click
    pub fn gutter_line(&self, pos: Point<f32>) -> Option<usize> {
        if !self.is_in_gutter(pos) {
            return None;
        }
        self.screen_to_line(pos)
    }

    /// Convert a terminal line number to screen Y coordinate
    pub fn line_to_screen_y(&self, line: usize) -> f32 {
        let relative_line = line.saturating_sub(self.config.first_visible_line);
        (relative_line as f32) * self.config.line_height - self.config.scroll_offset
    }

    /// Get the screen Y range for a line
    pub fn line_screen_bounds(&self, line: usize) -> (f32, f32) {
        let y = self.line_to_screen_y(line);
        (y, y + self.config.line_height)
    }
}

/// Represents the current hover state
#[derive(Debug, Clone, PartialEq)]
pub enum HoverState {
    /// Not hovering over anything significant
    None,
    /// Hovering over the gutter area at a specific line
    Gutter { line: usize },
    /// Hovering over a command zone
    Zone {
        line: usize,
        column: usize,
        zone_start: usize,
    },
    /// Hovering over a command output area
    Output {
        line: usize,
        column: usize,
        zone_start: usize,
    },
}

impl Default for HoverState {
    fn default() -> Self {
        Self::None
    }
}

impl HoverState {
    /// Create a new hover state from mouse position
    pub fn from_position(
        handler: &MouseHandler,
        pos: Point<f32>,
        zone_at_line: impl Fn(usize) -> Option<(usize, bool)>, // Returns (zone_start_line, is_output)
    ) -> Self {
        // Check gutter first
        if handler.is_in_gutter(pos) {
            if let Some(line) = handler.screen_to_line(pos) {
                return HoverState::Gutter { line };
            }
            return HoverState::None;
        }

        // Check terminal area
        if let Some((line, column)) = handler.screen_to_cell(pos) {
            if let Some((zone_start, is_output)) = zone_at_line(line) {
                if is_output {
                    return HoverState::Output {
                        line,
                        column,
                        zone_start,
                    };
                } else {
                    return HoverState::Zone {
                        line,
                        column,
                        zone_start,
                    };
                }
            }
        }

        HoverState::None
    }

    /// Check if we're hovering over a specific zone
    pub fn is_hovering_zone(&self, zone_start: usize) -> bool {
        match self {
            HoverState::Zone { zone_start: s, .. } | HoverState::Output { zone_start: s, .. } => {
                *s == zone_start
            }
            _ => false,
        }
    }

    /// Check if we're hovering over gutter
    pub fn is_in_gutter(&self) -> bool {
        matches!(self, HoverState::Gutter { .. })
    }

    /// Get the current line if any
    pub fn line(&self) -> Option<usize> {
        match self {
            HoverState::None => None,
            HoverState::Gutter { line } => Some(*line),
            HoverState::Zone { line, .. } => Some(*line),
            HoverState::Output { line, .. } => Some(*line),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_handler() -> MouseHandler {
        MouseHandler::new(MouseConfig::default())
    }

    // ===== MouseConfig Tests =====

    #[test]
    fn test_mouse_config_default() {
        let config = MouseConfig::default();
        assert_eq!(config.gutter_width, 20.0);
        assert_eq!(config.line_height, 20.0);
        assert_eq!(config.char_width, 8.0);
    }

    // ===== MouseHandler Tests =====

    #[test]
    fn test_screen_to_line_basic() {
        let handler = default_handler();

        assert_eq!(handler.screen_to_line(Point::new(50.0, 0.0)), Some(0));
        assert_eq!(handler.screen_to_line(Point::new(50.0, 19.0)), Some(0));
        assert_eq!(handler.screen_to_line(Point::new(50.0, 20.0)), Some(1));
        assert_eq!(handler.screen_to_line(Point::new(50.0, 40.0)), Some(2));
    }

    #[test]
    fn test_screen_to_line_with_scroll() {
        let config = MouseConfig {
            scroll_offset: 10.0,
            first_visible_line: 5,
            ..Default::default()
        };
        let handler = MouseHandler::new(config);

        // Y=10 with scroll_offset=10 -> adjusted_y=20 -> line_offset=1
        // first_visible_line=5, so result = 5 + 1 = 6
        assert_eq!(handler.screen_to_line(Point::new(50.0, 10.0)), Some(6));
    }

    #[test]
    fn test_screen_to_line_negative() {
        let handler = default_handler();
        assert_eq!(handler.screen_to_line(Point::new(50.0, -10.0)), None);
    }

    #[test]
    fn test_screen_to_column_basic() {
        let handler = default_handler();

        // Gutter is 20px, char_width is 8px
        assert_eq!(handler.screen_to_column(Point::new(10.0, 0.0)), None); // In gutter
        assert_eq!(handler.screen_to_column(Point::new(20.0, 0.0)), Some(0)); // First char
        assert_eq!(handler.screen_to_column(Point::new(28.0, 0.0)), Some(1)); // Second char
    }

    #[test]
    fn test_screen_to_column_in_gutter() {
        let handler = default_handler();
        assert!(handler.screen_to_column(Point::new(15.0, 0.0)).is_none());
    }

    #[test]
    fn test_screen_to_cell() {
        let handler = default_handler();

        assert_eq!(handler.screen_to_cell(Point::new(28.0, 25.0)), Some((1, 1)));
    }

    #[test]
    fn test_is_in_gutter() {
        let handler = default_handler();

        assert!(handler.is_in_gutter(Point::new(10.0, 10.0)));
        assert!(!handler.is_in_gutter(Point::new(25.0, 10.0)));
        assert!(!handler.is_in_gutter(Point::new(-5.0, 10.0)));
    }

    #[test]
    fn test_gutter_line() {
        let handler = default_handler();

        assert_eq!(handler.gutter_line(Point::new(10.0, 25.0)), Some(1));
        assert_eq!(handler.gutter_line(Point::new(25.0, 25.0)), None);
    }

    #[test]
    fn test_line_to_screen_y() {
        let config = MouseConfig {
            first_visible_line: 10,
            line_height: 20.0,
            ..Default::default()
        };
        let handler = MouseHandler::new(config);

        assert_eq!(handler.line_to_screen_y(10), 0.0);
        assert_eq!(handler.line_to_screen_y(11), 20.0);
        assert_eq!(handler.line_to_screen_y(12), 40.0);
    }

    #[test]
    fn test_line_screen_bounds() {
        let handler = default_handler();

        let (top, bottom) = handler.line_screen_bounds(1);
        assert_eq!(top, 20.0);
        assert_eq!(bottom, 40.0);
    }

    // ===== HoverState Tests =====

    #[test]
    fn test_hover_state_default() {
        let state = HoverState::default();
        assert_eq!(state, HoverState::None);
    }

    #[test]
    fn test_hover_state_from_position_gutter() {
        let handler = default_handler();

        let state = HoverState::from_position(&handler, Point::new(10.0, 25.0), |_| None);

        assert!(matches!(state, HoverState::Gutter { line: 1 }));
    }

    #[test]
    fn test_hover_state_from_position_zone() {
        let handler = default_handler();

        let state = HoverState::from_position(&handler, Point::new(30.0, 25.0), |line| {
            if line == 1 { Some((0, false)) } else { None }
        });

        assert!(matches!(
            state,
            HoverState::Zone {
                line: 1,
                column: 1,
                zone_start: 0
            }
        ));
    }

    #[test]
    fn test_hover_state_from_position_output() {
        let handler = default_handler();

        let state = HoverState::from_position(&handler, Point::new(30.0, 45.0), |line| {
            if line == 2 { Some((0, true)) } else { None }
        });

        assert!(matches!(
            state,
            HoverState::Output {
                line: 2,
                zone_start: 0,
                ..
            }
        ));
    }

    #[test]
    fn test_hover_state_from_position_none() {
        let handler = default_handler();

        let state = HoverState::from_position(&handler, Point::new(30.0, 25.0), |_| None);

        assert_eq!(state, HoverState::None);
    }

    #[test]
    fn test_hover_state_is_hovering_zone() {
        let state = HoverState::Zone {
            line: 5,
            column: 10,
            zone_start: 3,
        };

        assert!(state.is_hovering_zone(3));
        assert!(!state.is_hovering_zone(0));
    }

    #[test]
    fn test_hover_state_is_in_gutter() {
        assert!(HoverState::Gutter { line: 5 }.is_in_gutter());
        assert!(!HoverState::None.is_in_gutter());
    }

    #[test]
    fn test_hover_state_line() {
        assert_eq!(HoverState::None.line(), None);
        assert_eq!(HoverState::Gutter { line: 5 }.line(), Some(5));
        assert_eq!(
            HoverState::Zone {
                line: 3,
                column: 0,
                zone_start: 0
            }
            .line(),
            Some(3)
        );
    }
}

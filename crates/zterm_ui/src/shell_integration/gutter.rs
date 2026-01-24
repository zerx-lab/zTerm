//! Gutter rendering for shell integration
//!
//! This module provides utilities for rendering command status marks
//! in the terminal gutter (left margin).

use gpui::Rgba;

/// Visual state for a gutter mark
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterVisual {
    /// No mark (empty gutter)
    None,
    /// Command prompt mark (chevron)
    Prompt,
    /// Running command (spinner/loading)
    Running,
    /// Successful command (checkmark)
    Success,
    /// Failed command (X mark)
    Failure,
    /// Continuation line (vertical line)
    Continuation,
}

impl Default for GutterVisual {
    fn default() -> Self {
        Self::None
    }
}

/// Configuration for gutter rendering
#[derive(Debug, Clone)]
pub struct GutterConfig {
    /// Width of the gutter area in pixels
    pub width: f32,
    /// Padding on the left side
    pub padding_left: f32,
    /// Padding on the right side
    pub padding_right: f32,
    /// Size of the gutter marks
    pub mark_size: f32,
    /// Color for prompt marks
    pub prompt_color: Rgba,
    /// Color for running marks
    pub running_color: Rgba,
    /// Color for success marks
    pub success_color: Rgba,
    /// Color for failure marks
    pub failure_color: Rgba,
    /// Color for continuation lines
    pub continuation_color: Rgba,
    /// Whether to show marks
    pub show_marks: bool,
}

impl Default for GutterConfig {
    fn default() -> Self {
        Self {
            width: 20.0,
            padding_left: 4.0,
            padding_right: 4.0,
            mark_size: 12.0,
            prompt_color: Rgba {
                r: 0.4,
                g: 0.6,
                b: 1.0,
                a: 1.0,
            },
            running_color: Rgba {
                r: 1.0,
                g: 0.8,
                b: 0.0,
                a: 1.0,
            },
            success_color: Rgba {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            },
            failure_color: Rgba {
                r: 1.0,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
            continuation_color: Rgba {
                r: 0.4,
                g: 0.4,
                b: 0.4,
                a: 0.6,
            },
            show_marks: true,
        }
    }
}

impl GutterConfig {
    /// Get the color for a visual state
    pub fn color_for_visual(&self, visual: GutterVisual) -> Option<Rgba> {
        match visual {
            GutterVisual::None => None,
            GutterVisual::Prompt => Some(self.prompt_color),
            GutterVisual::Running => Some(self.running_color),
            GutterVisual::Success => Some(self.success_color),
            GutterVisual::Failure => Some(self.failure_color),
            GutterVisual::Continuation => Some(self.continuation_color),
        }
    }

    /// Get the usable width for marks (excluding padding)
    pub fn usable_width(&self) -> f32 {
        (self.width - self.padding_left - self.padding_right).max(0.0)
    }

    /// Get the center X position for marks
    pub fn mark_center_x(&self) -> f32 {
        self.padding_left + self.usable_width() / 2.0
    }
}

/// Information about a gutter mark to be rendered
#[derive(Debug, Clone)]
pub struct GutterMark {
    /// The visual type of the mark
    pub visual: GutterVisual,
    /// Y position (top of the line)
    pub y: f32,
    /// Height of the line
    pub height: f32,
    /// Whether this mark is currently hovered
    pub hovered: bool,
}

impl GutterMark {
    /// Create a new gutter mark
    pub fn new(visual: GutterVisual, y: f32, height: f32) -> Self {
        Self {
            visual,
            y,
            height,
            hovered: false,
        }
    }

    /// Set the hover state
    pub fn with_hover(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Get the center Y position
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
}

/// Compute the visual state from command state
pub fn command_state_to_visual(
    is_prompt_line: bool,
    is_running: bool,
    exit_code: Option<i32>,
) -> GutterVisual {
    if is_prompt_line {
        if is_running {
            GutterVisual::Running
        } else if let Some(code) = exit_code {
            if code == 0 {
                GutterVisual::Success
            } else {
                GutterVisual::Failure
            }
        } else {
            GutterVisual::Prompt
        }
    } else {
        // Not the prompt line - could be continuation
        GutterVisual::Continuation
    }
}

/// Describes the gutter icon to render
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GutterIcon {
    /// Chevron pointing right (>)
    Chevron,
    /// Checkmark
    Checkmark,
    /// X mark
    Cross,
    /// Loading spinner (angle in radians)
    Spinner { angle: f32 },
    /// Vertical line for continuation
    VerticalLine,
}

impl GutterIcon {
    /// Get the icon for a visual state
    pub fn for_visual(visual: GutterVisual, spinner_angle: f32) -> Option<Self> {
        match visual {
            GutterVisual::None => None,
            GutterVisual::Prompt => Some(GutterIcon::Chevron),
            GutterVisual::Running => Some(GutterIcon::Spinner {
                angle: spinner_angle,
            }),
            GutterVisual::Success => Some(GutterIcon::Checkmark),
            GutterVisual::Failure => Some(GutterIcon::Cross),
            GutterVisual::Continuation => Some(GutterIcon::VerticalLine),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== GutterVisual Tests =====

    #[test]
    fn test_gutter_visual_default() {
        assert_eq!(GutterVisual::default(), GutterVisual::None);
    }

    #[test]
    fn test_gutter_visual_variants() {
        let visuals = [
            GutterVisual::None,
            GutterVisual::Prompt,
            GutterVisual::Running,
            GutterVisual::Success,
            GutterVisual::Failure,
            GutterVisual::Continuation,
        ];

        // Just verify they're distinct
        for (i, a) in visuals.iter().enumerate() {
            for (j, b) in visuals.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    // ===== GutterConfig Tests =====

    #[test]
    fn test_gutter_config_default() {
        let config = GutterConfig::default();
        assert_eq!(config.width, 20.0);
        assert!(config.show_marks);
    }

    #[test]
    fn test_gutter_config_color_for_visual() {
        let config = GutterConfig::default();

        assert!(config.color_for_visual(GutterVisual::None).is_none());
        assert!(config.color_for_visual(GutterVisual::Prompt).is_some());
        assert!(config.color_for_visual(GutterVisual::Success).is_some());
        assert!(config.color_for_visual(GutterVisual::Failure).is_some());
    }

    #[test]
    fn test_gutter_config_usable_width() {
        let config = GutterConfig {
            width: 20.0,
            padding_left: 4.0,
            padding_right: 4.0,
            ..Default::default()
        };

        assert_eq!(config.usable_width(), 12.0);
    }

    #[test]
    fn test_gutter_config_usable_width_clamped() {
        let config = GutterConfig {
            width: 5.0,
            padding_left: 4.0,
            padding_right: 4.0,
            ..Default::default()
        };

        assert_eq!(config.usable_width(), 0.0);
    }

    #[test]
    fn test_gutter_config_mark_center_x() {
        let config = GutterConfig {
            width: 20.0,
            padding_left: 4.0,
            padding_right: 4.0,
            ..Default::default()
        };

        assert_eq!(config.mark_center_x(), 10.0); // 4 + 12/2
    }

    // ===== GutterMark Tests =====

    #[test]
    fn test_gutter_mark_new() {
        let mark = GutterMark::new(GutterVisual::Prompt, 100.0, 20.0);

        assert_eq!(mark.visual, GutterVisual::Prompt);
        assert_eq!(mark.y, 100.0);
        assert_eq!(mark.height, 20.0);
        assert!(!mark.hovered);
    }

    #[test]
    fn test_gutter_mark_with_hover() {
        let mark = GutterMark::new(GutterVisual::Success, 0.0, 20.0).with_hover(true);

        assert!(mark.hovered);
    }

    #[test]
    fn test_gutter_mark_center_y() {
        let mark = GutterMark::new(GutterVisual::Prompt, 100.0, 20.0);
        assert_eq!(mark.center_y(), 110.0);
    }

    // ===== command_state_to_visual Tests =====

    #[test]
    fn test_state_to_visual_prompt() {
        let visual = command_state_to_visual(true, false, None);
        assert_eq!(visual, GutterVisual::Prompt);
    }

    #[test]
    fn test_state_to_visual_running() {
        let visual = command_state_to_visual(true, true, None);
        assert_eq!(visual, GutterVisual::Running);
    }

    #[test]
    fn test_state_to_visual_success() {
        let visual = command_state_to_visual(true, false, Some(0));
        assert_eq!(visual, GutterVisual::Success);
    }

    #[test]
    fn test_state_to_visual_failure() {
        let visual = command_state_to_visual(true, false, Some(1));
        assert_eq!(visual, GutterVisual::Failure);
    }

    #[test]
    fn test_state_to_visual_continuation() {
        let visual = command_state_to_visual(false, false, None);
        assert_eq!(visual, GutterVisual::Continuation);
    }

    // ===== GutterIcon Tests =====

    #[test]
    fn test_gutter_icon_for_visual_none() {
        assert!(GutterIcon::for_visual(GutterVisual::None, 0.0).is_none());
    }

    #[test]
    fn test_gutter_icon_for_visual_prompt() {
        assert_eq!(
            GutterIcon::for_visual(GutterVisual::Prompt, 0.0),
            Some(GutterIcon::Chevron)
        );
    }

    #[test]
    fn test_gutter_icon_for_visual_running() {
        let icon = GutterIcon::for_visual(GutterVisual::Running, 1.5);
        assert!(matches!(icon, Some(GutterIcon::Spinner { angle: 1.5 })));
    }

    #[test]
    fn test_gutter_icon_for_visual_success() {
        assert_eq!(
            GutterIcon::for_visual(GutterVisual::Success, 0.0),
            Some(GutterIcon::Checkmark)
        );
    }

    #[test]
    fn test_gutter_icon_for_visual_failure() {
        assert_eq!(
            GutterIcon::for_visual(GutterVisual::Failure, 0.0),
            Some(GutterIcon::Cross)
        );
    }

    #[test]
    fn test_gutter_icon_for_visual_continuation() {
        assert_eq!(
            GutterIcon::for_visual(GutterVisual::Continuation, 0.0),
            Some(GutterIcon::VerticalLine)
        );
    }
}

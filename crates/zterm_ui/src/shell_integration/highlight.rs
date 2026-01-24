//! Zone highlighting for shell integration
//!
//! This module provides utilities for rendering visual highlights
//! around command zones.

use gpui::Rgba;

/// Type of highlight to apply to a zone
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightType {
    /// Subtle hover highlight
    Hover,
    /// Selected zone highlight
    Selected,
    /// Active/current zone highlight
    Active,
    /// Error highlight (for failed commands)
    Error,
}

/// Configuration for zone highlighting
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Background color for hover state
    pub hover_background: Rgba,
    /// Background color for selected state
    pub selected_background: Rgba,
    /// Background color for active state
    pub active_background: Rgba,
    /// Background color for error state
    pub error_background: Rgba,
    /// Border color for hover state
    pub hover_border: Rgba,
    /// Border color for selected state
    pub selected_border: Rgba,
    /// Border width
    pub border_width: f32,
    /// Corner radius for highlight rectangles
    pub corner_radius: f32,
    /// Whether to show highlights
    pub enabled: bool,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            hover_background: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.05,
            },
            selected_background: Rgba {
                r: 0.4,
                g: 0.6,
                b: 1.0,
                a: 0.15,
            },
            active_background: Rgba {
                r: 0.4,
                g: 0.8,
                b: 0.4,
                a: 0.1,
            },
            error_background: Rgba {
                r: 1.0,
                g: 0.3,
                b: 0.3,
                a: 0.1,
            },
            hover_border: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.2,
            },
            selected_border: Rgba {
                r: 0.4,
                g: 0.6,
                b: 1.0,
                a: 0.5,
            },
            border_width: 1.0,
            corner_radius: 4.0,
            enabled: true,
        }
    }
}

impl HighlightConfig {
    /// Get the background color for a highlight type
    pub fn background_for_type(&self, highlight_type: HighlightType) -> Rgba {
        match highlight_type {
            HighlightType::Hover => self.hover_background,
            HighlightType::Selected => self.selected_background,
            HighlightType::Active => self.active_background,
            HighlightType::Error => self.error_background,
        }
    }

    /// Get the border color for a highlight type
    pub fn border_for_type(&self, highlight_type: HighlightType) -> Option<Rgba> {
        match highlight_type {
            HighlightType::Hover => Some(self.hover_border),
            HighlightType::Selected => Some(self.selected_border),
            _ => None,
        }
    }
}

/// A region to be highlighted
#[derive(Debug, Clone)]
pub struct HighlightRegion {
    /// Start line (0-indexed)
    pub start_line: usize,
    /// End line (exclusive)
    pub end_line: usize,
    /// Type of highlight
    pub highlight_type: HighlightType,
    /// Optional custom opacity override
    pub opacity_override: Option<f32>,
}

impl HighlightRegion {
    /// Create a new highlight region
    pub fn new(start_line: usize, end_line: usize, highlight_type: HighlightType) -> Self {
        Self {
            start_line,
            end_line,
            highlight_type,
            opacity_override: None,
        }
    }

    /// Set a custom opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity_override = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Get the number of lines in this region
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line)
    }

    /// Check if this region contains a specific line
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line < self.end_line
    }

    /// Check if this region overlaps with another
    pub fn overlaps(&self, other: &HighlightRegion) -> bool {
        self.start_line < other.end_line && other.start_line < self.end_line
    }

    /// Merge with another region (if they overlap or are adjacent)
    pub fn merge(&self, other: &HighlightRegion) -> Option<HighlightRegion> {
        if self.highlight_type != other.highlight_type {
            return None;
        }

        // Check if adjacent or overlapping
        if self.end_line >= other.start_line && other.end_line >= self.start_line {
            Some(HighlightRegion {
                start_line: self.start_line.min(other.start_line),
                end_line: self.end_line.max(other.end_line),
                highlight_type: self.highlight_type,
                opacity_override: self.opacity_override.or(other.opacity_override),
            })
        } else {
            None
        }
    }
}

/// Computed highlight rectangle for rendering
#[derive(Debug, Clone)]
pub struct HighlightRect {
    /// X position (left edge)
    pub x: f32,
    /// Y position (top edge)
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Background color
    pub background: Rgba,
    /// Border color (if any)
    pub border: Option<Rgba>,
    /// Border width
    pub border_width: f32,
    /// Corner radius
    pub corner_radius: f32,
}

impl HighlightRect {
    /// Compute a highlight rectangle from a region
    pub fn from_region(
        region: &HighlightRegion,
        config: &HighlightConfig,
        line_height: f32,
        first_visible_line: usize,
        x: f32,
        width: f32,
        scroll_offset: f32,
    ) -> Option<Self> {
        if !config.enabled {
            return None;
        }

        // Calculate visible portion
        let visible_start = region.start_line.max(first_visible_line);
        if visible_start >= region.end_line {
            return None; // Completely above viewport
        }

        let relative_start = visible_start.saturating_sub(first_visible_line);
        let relative_end = region.end_line.saturating_sub(first_visible_line);

        let y = (relative_start as f32) * line_height - scroll_offset;
        let height = ((relative_end - relative_start) as f32) * line_height;

        let mut background = config.background_for_type(region.highlight_type);
        if let Some(opacity) = region.opacity_override {
            background.a *= opacity;
        }

        Some(HighlightRect {
            x,
            y,
            width,
            height,
            background,
            border: config.border_for_type(region.highlight_type),
            border_width: config.border_width,
            corner_radius: config.corner_radius,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== HighlightConfig Tests =====

    #[test]
    fn test_highlight_config_default() {
        let config = HighlightConfig::default();
        assert!(config.enabled);
        assert!(config.border_width > 0.0);
        assert!(config.corner_radius >= 0.0);
    }

    #[test]
    fn test_background_for_type() {
        let config = HighlightConfig::default();

        let hover = config.background_for_type(HighlightType::Hover);
        let selected = config.background_for_type(HighlightType::Selected);

        // They should be different
        assert_ne!(hover.a, selected.a);
    }

    #[test]
    fn test_border_for_type() {
        let config = HighlightConfig::default();

        assert!(config.border_for_type(HighlightType::Hover).is_some());
        assert!(config.border_for_type(HighlightType::Selected).is_some());
        assert!(config.border_for_type(HighlightType::Active).is_none());
        assert!(config.border_for_type(HighlightType::Error).is_none());
    }

    // ===== HighlightRegion Tests =====

    #[test]
    fn test_highlight_region_new() {
        let region = HighlightRegion::new(5, 10, HighlightType::Hover);

        assert_eq!(region.start_line, 5);
        assert_eq!(region.end_line, 10);
        assert_eq!(region.highlight_type, HighlightType::Hover);
        assert!(region.opacity_override.is_none());
    }

    #[test]
    fn test_highlight_region_with_opacity() {
        let region = HighlightRegion::new(0, 5, HighlightType::Selected).with_opacity(0.5);

        assert_eq!(region.opacity_override, Some(0.5));
    }

    #[test]
    fn test_highlight_region_opacity_clamped() {
        let region1 = HighlightRegion::new(0, 5, HighlightType::Hover).with_opacity(2.0);
        let region2 = HighlightRegion::new(0, 5, HighlightType::Hover).with_opacity(-0.5);

        assert_eq!(region1.opacity_override, Some(1.0));
        assert_eq!(region2.opacity_override, Some(0.0));
    }

    #[test]
    fn test_highlight_region_line_count() {
        let region = HighlightRegion::new(5, 15, HighlightType::Active);
        assert_eq!(region.line_count(), 10);
    }

    #[test]
    fn test_highlight_region_contains_line() {
        let region = HighlightRegion::new(5, 10, HighlightType::Hover);

        assert!(!region.contains_line(4));
        assert!(region.contains_line(5));
        assert!(region.contains_line(9));
        assert!(!region.contains_line(10)); // Exclusive end
    }

    #[test]
    fn test_highlight_region_overlaps() {
        let region1 = HighlightRegion::new(5, 15, HighlightType::Hover);
        let region2 = HighlightRegion::new(10, 20, HighlightType::Hover);
        let region3 = HighlightRegion::new(20, 25, HighlightType::Hover);

        assert!(region1.overlaps(&region2));
        assert!(region2.overlaps(&region1));
        assert!(!region1.overlaps(&region3));
    }

    #[test]
    fn test_highlight_region_merge_overlapping() {
        let region1 = HighlightRegion::new(5, 15, HighlightType::Hover);
        let region2 = HighlightRegion::new(10, 25, HighlightType::Hover);

        let merged = region1.merge(&region2);
        assert!(merged.is_some());

        let merged = merged.unwrap();
        assert_eq!(merged.start_line, 5);
        assert_eq!(merged.end_line, 25);
    }

    #[test]
    fn test_highlight_region_merge_adjacent() {
        let region1 = HighlightRegion::new(5, 10, HighlightType::Selected);
        let region2 = HighlightRegion::new(10, 15, HighlightType::Selected);

        let merged = region1.merge(&region2);
        assert!(merged.is_some());

        let merged = merged.unwrap();
        assert_eq!(merged.start_line, 5);
        assert_eq!(merged.end_line, 15);
    }

    #[test]
    fn test_highlight_region_merge_different_types() {
        let region1 = HighlightRegion::new(5, 15, HighlightType::Hover);
        let region2 = HighlightRegion::new(10, 20, HighlightType::Selected);

        assert!(region1.merge(&region2).is_none());
    }

    #[test]
    fn test_highlight_region_merge_non_overlapping() {
        let region1 = HighlightRegion::new(5, 10, HighlightType::Hover);
        let region2 = HighlightRegion::new(15, 20, HighlightType::Hover);

        assert!(region1.merge(&region2).is_none());
    }

    // ===== HighlightRect Tests =====

    #[test]
    fn test_highlight_rect_from_region_basic() {
        let config = HighlightConfig::default();
        let region = HighlightRegion::new(5, 10, HighlightType::Hover);

        let rect = HighlightRect::from_region(
            &region,
            &config,
            20.0, // line_height
            0,    // first_visible_line
            0.0,  // x
            800.0, // width
            0.0,  // scroll_offset
        );

        assert!(rect.is_some());
        let rect = rect.unwrap();
        assert_eq!(rect.y, 100.0); // 5 * 20
        assert_eq!(rect.height, 100.0); // 5 lines * 20
    }

    #[test]
    fn test_highlight_rect_disabled() {
        let mut config = HighlightConfig::default();
        config.enabled = false;
        let region = HighlightRegion::new(5, 10, HighlightType::Hover);

        let rect =
            HighlightRect::from_region(&region, &config, 20.0, 0, 0.0, 800.0, 0.0);

        assert!(rect.is_none());
    }

    #[test]
    fn test_highlight_rect_above_viewport() {
        let config = HighlightConfig::default();
        let region = HighlightRegion::new(5, 10, HighlightType::Hover);

        let rect = HighlightRect::from_region(
            &region,
            &config,
            20.0,
            20, // first_visible_line is after region
            0.0,
            800.0,
            0.0,
        );

        assert!(rect.is_none());
    }

    #[test]
    fn test_highlight_rect_with_scroll_offset() {
        let config = HighlightConfig::default();
        let region = HighlightRegion::new(0, 5, HighlightType::Selected);

        let rect = HighlightRect::from_region(
            &region,
            &config,
            20.0,
            0,
            0.0,
            800.0,
            10.0, // scroll_offset
        );

        assert!(rect.is_some());
        let rect = rect.unwrap();
        assert_eq!(rect.y, -10.0); // 0 - 10 scroll offset
    }
}

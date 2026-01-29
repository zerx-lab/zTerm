//! Terminal Element - Low-level rendering using GPUI Element trait
//!
//! Inspired by Zed's terminal_element.rs implementation.
//! Uses batched text rendering, viewport culling, and Interactivity for event handling.

use gpui::{
    fill, point, px, quad, relative, size, App, Bounds, ContentMask, Corners, Edges, Element,
    ElementId, FocusHandle, GlobalElementId, Hitbox, HitboxBehavior, Hsla, InspectorElementId,
    InteractiveElement, Interactivity, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Rgba, ScrollWheelEvent, SharedString,
    StatefulInteractiveElement, Style, TextAlign, TextRun, UnderlineStyle, Window,
};
use std::panic::Location;
use std::sync::Arc;
use zterm_terminal::{grid::Cell, Terminal};

use crate::elements::terminal_scrollbar::{TerminalScrollHandle, ThumbState, SCROLLBAR_WIDTH};
use crate::terminal_colors::color_to_rgba;

/// Terminal bounds information for rendering
#[derive(Debug, Clone, Copy)]
pub struct TerminalBounds {
    /// Width of a single cell in pixels
    pub cell_width: Pixels,
    /// Height of a single line in pixels
    pub line_height: Pixels,
    /// Total bounds of the terminal area
    pub bounds: Bounds<Pixels>,
}

impl TerminalBounds {
    pub fn new(line_height: Pixels, cell_width: Pixels, bounds: Bounds<Pixels>) -> Self {
        Self {
            cell_width,
            line_height,
            bounds,
        }
    }

    /// Number of visible lines
    pub fn num_lines(&self) -> usize {
        (self.bounds.size.height / self.line_height).floor() as usize
    }

    /// Number of visible columns
    pub fn num_columns(&self) -> usize {
        (self.bounds.size.width / self.cell_width).floor() as usize
    }
}

/// A batched text run combining multiple adjacent cells with the same style
#[derive(Debug, Clone)]
pub struct BatchedTextRun {
    /// Starting position (line, column)
    pub start_line: i32,
    pub start_col: i32,
    /// Combined text content
    pub text: String,
    /// Number of cells in this run
    pub cell_count: usize,
    /// Total width in cell units (for CJK characters)
    pub total_width: usize,
    /// Text style
    pub style: TextRunStyle,
    /// Font size
    pub font_size: Pixels,
}

/// Style information for a text run
#[derive(Debug, Clone, PartialEq)]
pub struct TextRunStyle {
    pub foreground: Rgba,
    pub background: Rgba,
    pub is_bold: bool,
    pub is_italic: bool,
    pub has_underline: bool,
    pub has_strikethrough: bool,
    pub is_invisible: bool,
    pub is_dim: bool,
}

impl TextRunStyle {
    pub fn from_cell(cell: &Cell, cx: &App) -> Self {
        let attrs = cell.attrs();

        // Handle reverse attribute (INVERSE) - swap fg/bg colors
        // Reference: Zed's terminal_element.rs line 355-359
        let reverse = attrs.reverse();
        let mut foreground = if reverse {
            color_to_rgba(&attrs.background, false, cx)
        } else {
            color_to_rgba(&attrs.foreground, true, cx)
        };
        let background = if reverse {
            color_to_rgba(&attrs.foreground, true, cx)
        } else {
            color_to_rgba(&attrs.background, false, cx)
        };

        // Handle DIM attribute - reduce alpha by 0.7
        // Reference: Zed's terminal_element.rs line 568-571
        if attrs.is_dim() {
            foreground.a *= 0.7;
        }

        Self {
            foreground,
            background,
            is_bold: attrs.is_bold(),
            is_italic: attrs.italic(),
            has_underline: attrs.has_underline(),
            has_strikethrough: attrs.strikethrough(),
            is_invisible: attrs.invisible(),
            is_dim: attrs.is_dim(),
        }
    }

    /// Convert to GPUI TextRun
    pub fn to_text_run(&self, font_family: SharedString) -> TextRun {
        TextRun {
            len: 0, // Will be set when creating shaped text
            font: gpui::Font {
                family: font_family,
                features: gpui::FontFeatures::default(),
                fallbacks: None,
                weight: if self.is_bold {
                    gpui::FontWeight::BOLD
                } else {
                    gpui::FontWeight::NORMAL
                },
                style: if self.is_italic {
                    gpui::FontStyle::Italic
                } else {
                    gpui::FontStyle::Normal
                },
            },
            color: Hsla::from(self.foreground),
            background_color: if self.background.a > 0.01 {
                Some(Hsla::from(self.background))
            } else {
                None
            },
            underline: if self.has_underline {
                Some(UnderlineStyle {
                    thickness: px(1.0),
                    color: Some(Hsla::from(self.foreground)),
                    wavy: false,
                })
            } else {
                None
            },
            strikethrough: if self.has_strikethrough {
                Some(gpui::StrikethroughStyle {
                    thickness: px(1.0),
                    color: Some(Hsla::from(self.foreground)),
                })
            } else {
                None
            },
        }
    }
}

impl BatchedTextRun {
    pub fn new(cell: &Cell, line: i32, col: usize, style: TextRunStyle, font_size: Pixels) -> Self {
        Self {
            start_line: line,
            start_col: col as i32,
            text: cell.text().to_string(),
            cell_count: 1,
            total_width: cell.width() as usize,
            style,
            font_size,
        }
    }

    /// Check if another cell can be appended to this run
    pub fn can_append(&self, other_style: &TextRunStyle, line: i32, col: usize) -> bool {
        self.style == *other_style
            && self.start_line == line
            && (self.start_col + self.total_width as i32) == col as i32
    }

    /// Append a cell to this run
    pub fn append(&mut self, cell: &Cell) {
        self.text.push_str(cell.text());
        self.cell_count += 1;
        self.total_width += cell.width() as usize;
    }

    /// Paint this text run
    pub fn paint(
        &self,
        origin: Point<Pixels>,
        dimensions: &TerminalBounds,
        font_family: SharedString,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.text.is_empty() || self.style.is_invisible {
            return;
        }

        let pos = Point::new(
            origin.x + dimensions.cell_width * self.start_col as usize,
            origin.y + dimensions.line_height * self.start_line as usize,
        );

        // Create text run
        let mut text_run = self.style.to_text_run(font_family);
        text_run.len = self.text.len();

        // Shape and paint text
        let shaped = window.text_system().shape_line(
            SharedString::from(self.text.clone()),
            self.font_size,
            &[text_run],
            None,
        );
        let _ = shaped.paint(
            pos,
            dimensions.line_height,
            TextAlign::Left,
            None,
            window,
            cx,
        );
    }
}

/// A background rectangle for cells with non-default background
#[derive(Debug, Clone)]
pub struct LayoutRect {
    pub line: i32,
    pub start_col: i32,
    pub end_col: i32,
    pub color: Rgba,
}

impl LayoutRect {
    pub fn new(line: i32, start_col: i32, end_col: i32, color: Rgba) -> Self {
        Self {
            line,
            start_col,
            end_col,
            color,
        }
    }

    /// Paint this background rectangle
    pub fn paint(&self, origin: Point<Pixels>, dimensions: &TerminalBounds, window: &mut Window) {
        let x = origin.x + dimensions.cell_width * self.start_col as usize;
        let y = origin.y + dimensions.line_height * self.line as usize;
        let width = dimensions.cell_width * (self.end_col - self.start_col + 1) as usize;

        window.paint_quad(fill(
            Bounds::new(point(x, y), size(width, dimensions.line_height)),
            Hsla::from(self.color),
        ));
    }
}

/// Cursor layout information
#[derive(Debug, Clone)]
pub struct CursorLayout {
    pub position: Point<Pixels>,
    pub width: Pixels,
    pub height: Pixels,
    pub color: Rgba,
    pub shape: zterm_terminal::grid::CursorShape,
}

impl CursorLayout {
    pub fn paint(&self, origin: Point<Pixels>, window: &mut Window) {
        use zterm_terminal::grid::CursorShape;

        let pos = origin + self.position;
        let bounds = Bounds::new(pos, size(self.width, self.height));

        match self.shape {
            CursorShape::Block => {
                window.paint_quad(fill(bounds, Hsla::from(self.color)));
            }
            CursorShape::Underline => {
                let underline_bounds = Bounds::new(
                    point(pos.x, pos.y + self.height - px(2.0)),
                    size(self.width, px(2.0)),
                );
                window.paint_quad(fill(underline_bounds, Hsla::from(self.color)));
            }
            CursorShape::Bar => {
                let bar_bounds = Bounds::new(pos, size(px(2.0), self.height));
                window.paint_quad(fill(bar_bounds, Hsla::from(self.color)));
            }
        }
    }
}

/// Layout state produced by prepaint, consumed by paint
pub struct LayoutState {
    pub hitbox: Hitbox,
    pub batched_text_runs: Vec<BatchedTextRun>,
    pub background_rects: Vec<LayoutRect>,
    pub cursor: Option<CursorLayout>,
    pub dimensions: TerminalBounds,
    pub background_color: Rgba,
    pub display_offset: usize,
    pub total_lines: usize,
    pub viewport_lines: usize,
    /// Scrollbar track bounds (right edge of terminal)
    pub scrollbar_track_bounds: Option<Bounds<Pixels>>,
    /// Whether content is scrollable
    pub is_scrollable: bool,
}

/// Scroll multiplier for mouse wheel events
const SCROLL_MULTIPLIER: f32 = 3.0;

/// Terminal Element - implements GPUI Element trait for high-performance rendering
///
/// Uses Interactivity for mouse and scroll event handling, following Zed's pattern.
pub struct TerminalElement {
    terminal: Arc<Terminal>,
    focus_handle: FocusHandle,
    font_family: SharedString,
    font_size: Pixels,
    line_height_multiplier: f32,
    theme_colors: axon_ui::theme::ThemeColors,
    /// Interactivity for mouse/scroll events (following Zed's pattern)
    interactivity: Interactivity,
    /// Scroll handle for scrollbar interaction
    scroll_handle: Option<TerminalScrollHandle>,
}

/// Implement InteractiveElement trait to enable event handling
impl InteractiveElement for TerminalElement {
    fn interactivity(&mut self) -> &mut Interactivity {
        &mut self.interactivity
    }
}

/// Implement StatefulInteractiveElement for stateful interactions
impl StatefulInteractiveElement for TerminalElement {}

impl TerminalElement {
    pub fn new(
        terminal: Arc<Terminal>,
        focus_handle: FocusHandle,
        theme_colors: axon_ui::theme::ThemeColors,
    ) -> Self {
        // Create scroll handle
        let scroll_handle = TerminalScrollHandle::new(terminal.clone());

        Self {
            terminal,
            focus_handle: focus_handle.clone(),
            font_family: SharedString::from("Consolas"),
            font_size: px(13.0),
            line_height_multiplier: 1.4,
            theme_colors,
            interactivity: Interactivity::default(),
            scroll_handle: Some(scroll_handle),
        }
        .track_focus(&focus_handle)
    }

    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = family.into();
        self
    }

    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    pub fn line_height_multiplier(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier;
        self
    }

    /// Set the scroll handle (for external control)
    pub fn scroll_handle(mut self, handle: TerminalScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    /// Register mouse event listeners (called in paint phase)
    fn register_mouse_listeners(&mut self, hitbox: &Hitbox, line_height: Pixels) {
        let terminal = self.terminal.clone();
        let max_offset = terminal.max_display_offset();

        tracing::debug!(
            "register_mouse_listeners: hitbox={:?}, max_scroll_offset={}",
            hitbox.bounds,
            max_offset
        );

        // Register scroll wheel handler
        self.interactivity.on_scroll_wheel({
            let terminal = terminal.clone();
            move |event: &ScrollWheelEvent, window, _cx| {
                tracing::info!("on_scroll_wheel triggered: delta={:?}", event.delta);
                let scrolled = Self::handle_scroll_wheel(&terminal, event, line_height);
                if scrolled {
                    // Trigger window refresh to redraw with new display_offset
                    window.refresh();
                }
            }
        });

        // TODO: Add mouse click/drag handlers for text selection
        let _ = hitbox; // Will be used for selection bounds
    }

    /// Handle scroll wheel events
    /// Returns true if scrolling occurred
    fn handle_scroll_wheel(
        terminal: &Terminal,
        event: &ScrollWheelEvent,
        line_height: Pixels,
    ) -> bool {
        // Calculate scroll lines from pixel delta
        // Note: Negative delta.y means scrolling down (mouse wheel away from user)
        // which should show history (scroll_up in terminal terms)
        let delta_y = match event.delta {
            gpui::ScrollDelta::Lines(lines) => lines.y,
            gpui::ScrollDelta::Pixels(pixels) => pixels.y / line_height,
        };

        // Apply scroll multiplier and calculate absolute scroll lines
        let scroll_lines = (delta_y.abs() * SCROLL_MULTIPLIER).round() as usize;
        if scroll_lines == 0 {
            return false;
        }

        let old_offset = terminal.display_offset();

        // Standard scroll behavior (like web browsers):
        // Positive delta = mouse wheel up = view history (scroll content up)
        // Negative delta = mouse wheel down = view current (scroll content down)
        if delta_y > 0.0 {
            // Mouse wheel scrolled up -> view history
            terminal.scroll_up_by(scroll_lines);
        } else {
            // Mouse wheel scrolled down -> view current content
            terminal.scroll_down_by(scroll_lines);
        }

        let new_offset = terminal.display_offset();
        let scrolled = old_offset != new_offset;

        tracing::info!(
            "scroll_wheel: delta_y={:?}, scroll_lines={}, offset: {} -> {}, scrolled={}",
            delta_y,
            scroll_lines,
            old_offset,
            new_offset,
            scrolled
        );

        scrolled
    }

    /// Layout all cells into batched text runs and background rects
    fn layout_grid(
        cells: &[Vec<Cell>],
        start_line_offset: i32,
        font_size: Pixels,
        cx: &App,
    ) -> (Vec<LayoutRect>, Vec<BatchedTextRun>) {
        let mut batched_runs: Vec<BatchedTextRun> = Vec::with_capacity(cells.len() * 10);
        let mut background_rects: Vec<LayoutRect> = Vec::with_capacity(cells.len() * 5);

        for (line_idx, line_cells) in cells.iter().enumerate() {
            let line = start_line_offset + line_idx as i32;
            let mut current_batch: Option<BatchedTextRun> = None;
            let mut current_bg_rect: Option<LayoutRect> = None;

            for (col, cell) in line_cells.iter().enumerate() {
                let style = TextRunStyle::from_cell(cell, cx);

                // Handle background color
                if style.background.a > 0.01 {
                    if let Some(ref mut rect) = current_bg_rect {
                        if rect.color == style.background && rect.end_col + 1 == col as i32 {
                            rect.end_col = col as i32;
                        } else {
                            background_rects.push(current_bg_rect.take().unwrap());
                            current_bg_rect = Some(LayoutRect::new(
                                line,
                                col as i32,
                                col as i32,
                                style.background,
                            ));
                        }
                    } else {
                        current_bg_rect = Some(LayoutRect::new(
                            line,
                            col as i32,
                            col as i32,
                            style.background,
                        ));
                    }
                } else if let Some(rect) = current_bg_rect.take() {
                    background_rects.push(rect);
                }

                // Get cell text - handle empty and placeholder cells
                let text = cell.text();

                // Skip only truly empty cells and wide char placeholders (second cell of CJK chars)
                if text.is_empty() || text == "\0" {
                    if let Some(batch) = current_batch.take() {
                        batched_runs.push(batch);
                    }
                    continue;
                }

                // Try to append to current batch
                if let Some(ref mut batch) = current_batch {
                    if batch.can_append(&style, line, col) {
                        batch.append(cell);
                        continue;
                    } else {
                        batched_runs.push(current_batch.take().unwrap());
                    }
                }

                // Start new batch
                current_batch = Some(BatchedTextRun::new(cell, line, col, style, font_size));
            }

            // Flush remaining batch and rect for this line
            if let Some(batch) = current_batch.take() {
                batched_runs.push(batch);
            }
            if let Some(rect) = current_bg_rect.take() {
                background_rects.push(rect);
            }
        }

        (background_rects, batched_runs)
    }
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = LayoutState;

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::Name("terminal-element".into()))
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Request to fill available space - let parent container decide size
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();
        style.flex_grow = 1.0;

        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Calculate dimensions
        let line_height = self.font_size * self.line_height_multiplier;

        // Measure cell width using font metrics
        let cell_width = {
            let font = gpui::Font {
                family: self.font_family.clone(),
                features: gpui::FontFeatures::default(),
                fallbacks: None,
                weight: gpui::FontWeight::NORMAL,
                style: gpui::FontStyle::Normal,
            };
            let font_id = cx.text_system().resolve_font(&font);
            cx.text_system()
                .advance(font_id, self.font_size, 'M')
                .map(|advance| advance.width)
                .unwrap_or(px(8.4))
        };

        // Calculate terminal size based on available bounds
        let available_height = bounds.size.height;
        let available_width = bounds.size.width;

        let new_rows = (available_height / line_height).floor().max(1.0) as u16;
        let new_cols = (available_width / cell_width).floor().max(1.0) as u16;

        // Resize terminal if needed
        let current_size = self.terminal.size();
        if current_size.rows != new_rows || current_size.cols != new_cols {
            tracing::info!(
                "Resizing terminal: {}x{} -> {}x{}",
                current_size.cols,
                current_size.rows,
                new_cols,
                new_rows
            );
            if let Err(e) = self
                .terminal
                .resize(zterm_terminal::TerminalSize::new(new_rows, new_cols))
            {
                tracing::error!("Failed to resize terminal: {}", e);
            }
        }

        let dimensions = TerminalBounds::new(line_height, cell_width, bounds);

        // Get viewport cells (considering display_offset for scrolling)
        let (cells, total_lines, display_offset) = self.terminal.get_viewport_cells();
        let cursor_info = self.terminal.cursor_info();
        let viewport_rows = new_rows as usize;

        tracing::debug!(
            "TerminalElement::prepaint: cells.len()={}, total_lines={}, viewport={}, display_offset={}",
            cells.len(),
            total_lines,
            viewport_rows,
            display_offset
        );

        // DEBUG: 打印每行的内容预览
        for (i, row) in cells.iter().enumerate() {
            let preview: String = row.iter().take(20).map(|c| c.text()).collect();
            let non_empty_count = row.iter().filter(|c| !c.text().trim().is_empty()).count();
            tracing::debug!(
                "  cells[{}]: len={}, non_empty={}, preview='{}'",
                i,
                row.len(),
                non_empty_count,
                preview
            );
        }

        // === 可见区域裁剪优化 ===
        // 计算终端 bounds 与当前内容遮罩（可见视口）的交集
        // 这允许我们只处理实际可见的行，提升滚动容器中的性能
        let visible_bounds = window.content_mask().bounds;
        let intersection = visible_bounds.intersect(&bounds);

        // 如果终端完全不可见（在视口外），跳过所有单元格处理
        let (background_rects, batched_text_runs) = if intersection.size.height <= px(0.)
            || intersection.size.width <= px(0.)
        {
            tracing::trace!("TerminalElement: fully clipped, skipping layout");
            (Vec::new(), Vec::new())
        } else if intersection == bounds {
            // 快速路径：终端完全可见，无需裁剪
            Self::layout_grid(&cells, 0, self.font_size, cx)
        } else {
            // 计算哪些屏幕行可见
            let rows_above_viewport =
                ((intersection.top() - bounds.top()).max(px(0.)) / line_height).floor() as usize;
            let visible_row_count = (intersection.size.height / line_height).ceil() as usize + 1;

            tracing::trace!(
                "TerminalElement: partial visibility, rows {}..{} of {}",
                rows_above_viewport,
                rows_above_viewport + visible_row_count,
                cells.len()
            );

            // 只处理可见行
            let visible_cells: Vec<Vec<Cell>> = cells
                .into_iter()
                .skip(rows_above_viewport)
                .take(visible_row_count)
                .collect();

            Self::layout_grid(
                &visible_cells,
                rows_above_viewport as i32,
                self.font_size,
                cx,
            )
        };

        // Layout cursor (only visible when display_offset is 0, showing current content)
        let cursor = if display_offset == 0 && cursor_info.visible && cursor_info.y < viewport_rows
        {
            Some(CursorLayout {
                position: point(cell_width * cursor_info.x, line_height * cursor_info.y),
                width: cell_width,
                height: line_height,
                color: self.theme_colors.tab_active_indicator.to_rgb(),
                shape: cursor_info.shape,
            })
        } else {
            None
        };

        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        // Calculate scrollbar track bounds
        let is_scrollable = total_lines > viewport_rows;
        let scrollbar_track_bounds = if is_scrollable {
            Some(Bounds {
                origin: Point::new(
                    bounds.origin.x + bounds.size.width - SCROLLBAR_WIDTH,
                    bounds.origin.y,
                ),
                size: gpui::Size {
                    width: SCROLLBAR_WIDTH,
                    height: bounds.size.height,
                },
            })
        } else {
            None
        };

        // Update scroll handle state
        if let Some(ref scroll_handle) = self.scroll_handle {
            scroll_handle.update(line_height);
            // Apply any pending scroll from scrollbar drag
            scroll_handle.apply_pending_scroll();
        }

        LayoutState {
            hitbox,
            batched_text_runs,
            background_rects,
            cursor,
            dimensions,
            background_color: self.theme_colors.background.to_rgb(),
            display_offset,
            total_lines,
            viewport_lines: viewport_rows,
            scrollbar_track_bounds,
            is_scrollable,
        }
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let layout = prepaint;
        let line_height = layout.dimensions.line_height;

        tracing::debug!(
            "TerminalElement::paint: bounds={:?}, hitbox_bounds={:?}, scrollback={}",
            bounds,
            layout.hitbox.bounds,
            layout.total_lines.saturating_sub(layout.viewport_lines)
        );

        // Register mouse listeners for scroll and selection
        self.register_mouse_listeners(&layout.hitbox, line_height);

        // Paint using Interactivity (handles event dispatch)
        self.interactivity.paint(
            global_id,
            inspector_id,
            bounds,
            Some(&layout.hitbox),
            window,
            cx,
            |_, window, _cx| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    // 1. Paint background
                    window.paint_quad(fill(bounds, Hsla::from(layout.background_color)));

                    let origin = bounds.origin;

                    // 2. Paint background rectangles
                    for rect in &layout.background_rects {
                        rect.paint(origin, &layout.dimensions, window);
                    }
                });
            },
        );

        // Paint text and cursor outside of interactivity.paint to avoid borrowing issues
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            let origin = bounds.origin;

            // 3. Paint batched text runs
            for batch in &layout.batched_text_runs {
                batch.paint(
                    origin,
                    &layout.dimensions,
                    self.font_family.clone(),
                    window,
                    cx,
                );
            }

            // 4. Paint cursor
            if let Some(ref cursor) = layout.cursor {
                cursor.paint(origin, window);
            }
        });

        // 5. Paint scrollbar (if scrollable)
        if let (Some(track_bounds), Some(scroll_handle)) =
            (layout.scrollbar_track_bounds, &self.scroll_handle)
        {
            Self::paint_scrollbar(&self.theme_colors, track_bounds, scroll_handle, window);
            Self::register_scrollbar_listeners(track_bounds, scroll_handle.clone(), window);
        }
    }
}

// Scrollbar painting and event handling methods
impl TerminalElement {
    /// Paint the scrollbar
    fn paint_scrollbar(
        theme_colors: &axon_ui::theme::ThemeColors,
        track_bounds: Bounds<Pixels>,
        scroll_handle: &TerminalScrollHandle,
        window: &mut Window,
    ) {
        // Use dedicated scrollbar colors from theme (reference: Zed theme system)
        let track_color = theme_colors.scrollbar_track_background;
        let thumb_base_color = theme_colors.scrollbar_thumb_background;
        let thumb_hover_color = theme_colors.scrollbar_thumb_hover_background;
        let thumb_active_color = theme_colors.scrollbar_thumb_active_background;

        // Paint track background
        window.paint_quad(fill(track_bounds, track_color));

        // Calculate and paint thumb
        let thumb_bounds = scroll_handle.thumb_bounds(track_bounds);
        tracing::debug!(
            "paint_scrollbar: thumb_bounds={:?}",
            thumb_bounds
        );
        let thumb_color =
            scroll_handle.thumb_color(thumb_base_color, thumb_hover_color, thumb_active_color);

        window.paint_quad(quad(
            thumb_bounds,
            Corners::all(px(4.0)), // Rounded corners
            thumb_color,
            Edges::default(),
            Hsla::transparent_black(),
            gpui::BorderStyle::default(),
        ));
    }

    /// Register scrollbar mouse event listeners
    fn register_scrollbar_listeners(
        track_bounds: Bounds<Pixels>,
        scroll_handle: TerminalScrollHandle,
        window: &mut Window,
    ) {
        tracing::info!(
            "register_scrollbar_listeners called: track_bounds={:?}",
            track_bounds
        );
        // Mouse down - start drag or jump to position
        let handle_down = scroll_handle.clone();
        let track_bounds_down = track_bounds;
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            tracing::info!("MouseDownEvent: pos={:?}, phase={:?}, button={:?}", event.position, phase, event.button);
            if phase != gpui::DispatchPhase::Bubble {
                tracing::trace!("  -> Wrong phase, skipping");
                return;
            }
            if event.button != MouseButton::Left {
                tracing::debug!("  -> Not left button");
                return;
            }
            if !track_bounds_down.contains(&event.position) {
                tracing::info!("  -> Outside scrollbar track");
                tracing::debug!("    track_bounds={:?}, event.pos={:?}", track_bounds_down, event.position);
                return;
            }            let is_in_thumb = handle_down.is_point_in_thumb(event.position, track_bounds_down);
            tracing::info!("  -> Checking thumb hit: is_in_thumb={}", is_in_thumb);
            if is_in_thumb {
                tracing::info!("  -> Click on thumb, starting drag");
                // Click on thumb - start dragging
                handle_down.start_drag(event.position.y, track_bounds_down);
            } else {
                tracing::info!("  -> Click on track, jumping to position");
                // Click on track - jump to position
                handle_down.jump_to_position(event.position.y, track_bounds_down);
            }
            window.refresh();
            cx.stop_propagation();
        });
            
            // Stop propagation after handling scrollbar event
            // This prevents the terminal content area from receiving the event

        // Mouse move - update drag or hover state
        let handle_move = scroll_handle.clone();
        let track_bounds_move = track_bounds;        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if event.dragging() {
                tracing::info!("MouseMoveEvent while dragging: pos={:?}, phase={:?}", event.position, phase);
            }
            if phase != gpui::DispatchPhase::Bubble {
                tracing::trace!("  -> Wrong phase, skipping");
                return;
            }

            let thumb_state = handle_move.thumb_state.get();

            match thumb_state {
                ThumbState::Dragging { .. } => {
                    // Continue dragging
                    handle_move.update_drag(event.position.y, track_bounds_move);
                    window.refresh();
                    cx.stop_propagation();
                }
                _ => {
                    // Update hover state
                    let in_thumb = handle_move.is_point_in_thumb(event.position, track_bounds_move);
                    let was_hovered = matches!(thumb_state, ThumbState::Hovered);
                    if in_thumb != was_hovered {
                        handle_move.set_hovered(in_thumb);
                        window.refresh();
                        if in_thumb {
                            cx.stop_propagation();
                        }
                    }
                }
            }
        });

        // Mouse up - end drag
        let handle_up = scroll_handle.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
            if phase != gpui::DispatchPhase::Bubble {
                tracing::trace!("  -> Wrong phase, skipping");
                return;
            }
            if event.button != MouseButton::Left {
                tracing::debug!("  -> Not left button");
                return;
            }

            if matches!(handle_up.thumb_state.get(), ThumbState::Dragging { .. }) {
                handle_up.end_drag();
                window.refresh();
                cx.stop_propagation();
            }
        });
    }
}

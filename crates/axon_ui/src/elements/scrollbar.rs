//! Terminal scrollbar element
//!
//! Implements scrollbar using Element trait with window-level mouse event
//! handling, following Zed's implementation pattern for reliable drag behavior.

use gpui::*;

/// Scrollbar width in pixels
const SCROLLBAR_WIDTH: Pixels = px(8.0);

/// Minimum thumb height ratio
const MIN_THUMB_RATIO: f32 = 0.1;

/// Scrollbar thumb state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThumbState {
    #[default]
    Inactive,
    Hovered,
    Dragging {
        /// Offset from thumb top where drag started
        offset: i32,
    },
}

impl ThumbState {
    pub fn is_dragging(&self) -> bool {
        matches!(self, ThumbState::Dragging { .. })
    }
}

/// Scrollbar state stored in Entity for persistence across frames
pub struct ScrollbarState {
    /// Current thumb state
    pub thumb_state: ThumbState,
    /// Thumb bounds from last paint (for hit testing)
    pub thumb_bounds: Option<Bounds<Pixels>>,
    /// Track bounds from last paint
    pub track_bounds: Option<Bounds<Pixels>>,
}

impl ScrollbarState {
    pub fn new() -> Self {
        Self {
            thumb_state: ThumbState::Inactive,
            thumb_bounds: None,
            track_bounds: None,
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.thumb_state.is_dragging()
    }

    pub fn is_active(&self) -> bool {
        self.thumb_state != ThumbState::Inactive
    }

    pub fn start_drag(&mut self, offset: i32) {
        self.thumb_state = ThumbState::Dragging { offset };
    }

    pub fn end_drag(&mut self) {
        self.thumb_state = ThumbState::Inactive;
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        if !self.is_dragging() {
            self.thumb_state = if hovered {
                ThumbState::Hovered
            } else {
                ThumbState::Inactive
            };
        }
    }
}

/// Scrollbar element that renders and handles mouse events
pub struct ScrollbarElement {
    /// Entity holding scrollbar state
    state: Entity<ScrollbarState>,
    /// Total content lines (history + visible)
    total_lines: usize,
    /// Visible lines count
    visible_lines: usize,
    /// Current scroll offset (0 = bottom)
    scroll_offset: usize,
    /// Maximum scroll offset (history size)
    max_scroll: usize,
    /// Callback when scroll position changes
    on_scroll: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl ScrollbarElement {
    pub fn new(
        state: Entity<ScrollbarState>,
        total_lines: usize,
        visible_lines: usize,
        scroll_offset: usize,
        max_scroll: usize,
    ) -> Self {
        Self {
            state,
            total_lines,
            visible_lines,
            scroll_offset,
            max_scroll,
            on_scroll: None,
        }
    }

    pub fn on_scroll(mut self, callback: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_scroll = Some(Box::new(callback));
        self
    }

    /// Calculate thumb dimensions
    fn thumb_dimensions(&self, track_height: f32) -> (f32, f32) {
        // Thumb height as ratio of track
        let thumb_height_ratio = if self.total_lines > 0 {
            (self.visible_lines as f32 / self.total_lines as f32)
                .max(MIN_THUMB_RATIO)
                .min(1.0)
        } else {
            1.0
        };

        // Thumb top position
        let thumb_top_ratio = if self.max_scroll > 0 {
            let position_ratio = 1.0 - (self.scroll_offset as f32 / self.max_scroll as f32);
            position_ratio * (1.0 - thumb_height_ratio)
        } else {
            0.0
        };

        let thumb_height = track_height * thumb_height_ratio;
        let thumb_top = track_height * thumb_top_ratio;

        (thumb_top, thumb_height)
    }
}

/// Prepaint state for scrollbar
pub struct ScrollbarPrepaintState {
    thumb_bounds: Bounds<Pixels>,
    track_bounds: Bounds<Pixels>,
}

impl Element for ScrollbarElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<ScrollbarPrepaintState>;

    fn id(&self) -> Option<ElementId> {
        Some("terminal-scrollbar".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            size: Size {
                width: SCROLLBAR_WIDTH.into(),
                height: Length::Auto,
            },
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        };

        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // No scrollbar needed if no scrollback
        if self.max_scroll == 0 {
            return None;
        }

        let track_height: f32 = bounds.size.height.into();
        let (thumb_top, thumb_height) = self.thumb_dimensions(track_height);

        let track_bounds = bounds;
        let thumb_bounds = Bounds::new(
            Point::new(bounds.origin.x, bounds.origin.y + px(thumb_top)),
            Size {
                width: bounds.size.width,
                height: px(thumb_height),
            },
        );

        // Store bounds in state for hit testing in event handlers
        self.state.update(cx, |state, _| {
            state.thumb_bounds = Some(thumb_bounds);
            state.track_bounds = Some(track_bounds);
        });

        Some(ScrollbarPrepaintState {
            thumb_bounds,
            track_bounds,
        })
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(prepaint) = prepaint.take() else {
            return;
        };

        let is_dragging = self.state.read(cx).is_dragging();
        let is_active = self.state.read(cx).is_active();

        // Paint track (subtle background)
        let track_color = if is_active {
            rgba(0x2a2a2a40)
        } else {
            rgba(0x00000000)
        };
        window.paint_quad(quad(
            prepaint.track_bounds,
            Corners::default(),
            track_color,
            Edges::default(),
            Hsla::transparent_black(),
            BorderStyle::default(),
        ));

        // Paint thumb
        let thumb_color = if is_dragging {
            rgba(0xaaaaaa99) // Bright when dragging
        } else if is_active {
            rgba(0x88888880) // Visible on hover
        } else {
            rgba(0x55555530) // Very faint by default
        };

        let scrollbar_width_f32: f32 = SCROLLBAR_WIDTH.into();
        let corner_radius: f32 = scrollbar_width_f32 / 2.0;
        window.paint_quad(quad(
            prepaint.thumb_bounds,
            Corners::all(px(corner_radius)),
            thumb_color,
            Edges::default(),
            Hsla::transparent_black(),
            BorderStyle::default(),
        ));

        // Determine dispatch phase: Capture when dragging, Bubble otherwise
        let phase = if is_dragging {
            DispatchPhase::Capture
        } else {
            DispatchPhase::Bubble
        };

        // Register mouse down event
        let state = self.state.clone();
        let max_scroll = self.max_scroll;
        let total_lines = self.total_lines;
        let visible_lines = self.visible_lines;
        let on_scroll = self.on_scroll.take();

        // Wrap on_scroll in Rc for sharing across closures
        let on_scroll = on_scroll.map(std::rc::Rc::new);

        // Mouse down handler
        {
            let state = state.clone();
            let on_scroll = on_scroll.clone();

            window.on_mouse_event(move |event: &MouseDownEvent, event_phase, window, cx| {
                if event_phase != phase || event.button != MouseButton::Left {
                    return;
                }

                let thumb_bounds = state.read(cx).thumb_bounds;
                let track_bounds = state.read(cx).track_bounds;

                let (Some(thumb), Some(track)) = (thumb_bounds, track_bounds) else {
                    return;
                };

                if thumb.contains(&event.position) {
                    // Start dragging from thumb
                    let offset_px: f32 = (event.position.y - thumb.origin.y).into();
                    let offset: i32 = offset_px as i32;
                    state.update(cx, |state, _| {
                        state.start_drag(offset);
                    });
                    cx.stop_propagation();
                } else if track.contains(&event.position) {
                    // Click on track - jump to position
                    let track_top: f32 = track.origin.y.into();
                    let track_height: f32 = track.size.height.into();
                    let click_y: f32 = event.position.y.into();

                    let thumb_height_ratio = if total_lines > 0 {
                        (visible_lines as f32 / total_lines as f32)
                            .max(MIN_THUMB_RATIO)
                            .min(1.0)
                    } else {
                        1.0
                    };
                    let thumb_height = track_height * thumb_height_ratio;
                    let available_space = track_height - thumb_height;

                    if available_space > 0.0 && max_scroll > 0 {
                        // Center thumb at click position
                        let target_thumb_center = click_y - track_top;
                        let target_thumb_top =
                            (target_thumb_center - thumb_height / 2.0).clamp(0.0, available_space);
                        let position_ratio = target_thumb_top / available_space;
                        let new_offset = ((1.0 - position_ratio) * max_scroll as f32) as usize;

                        if let Some(ref callback) = on_scroll {
                            callback(new_offset.min(max_scroll), window, cx);
                        }
                    }
                    cx.stop_propagation();
                }
            });
        }

        // Mouse move handler
        {
            let state = state.clone();
            let on_scroll = on_scroll.clone();

            window.on_mouse_event(move |event: &MouseMoveEvent, event_phase, window, cx| {
                if event_phase != phase {
                    return;
                }

                let thumb_state = state.read(cx).thumb_state;
                let track_bounds = state.read(cx).track_bounds;

                match thumb_state {
                    ThumbState::Dragging { offset } if event.dragging() => {
                        // Handle drag
                        if let Some(track) = track_bounds {
                            let track_top: f32 = track.origin.y.into();
                            let track_height: f32 = track.size.height.into();
                            let mouse_y: f32 = event.position.y.into();

                            // Calculate new thumb top
                            let new_thumb_top = mouse_y - track_top - offset as f32;

                            // Calculate thumb dimensions for offset calculation
                            let thumb_height_ratio = if total_lines > 0 {
                                (visible_lines as f32 / total_lines as f32)
                                    .max(MIN_THUMB_RATIO)
                                    .min(1.0)
                            } else {
                                1.0
                            };
                            let thumb_height = track_height * thumb_height_ratio;
                            let available_space = track_height - thumb_height;

                            if available_space > 0.0 && max_scroll > 0 {
                                let clamped_top = new_thumb_top.clamp(0.0, available_space);
                                let position_ratio = clamped_top / available_space;
                                let new_offset =
                                    ((1.0 - position_ratio) * max_scroll as f32) as usize;

                                if let Some(ref callback) = on_scroll {
                                    callback(new_offset.min(max_scroll), window, cx);
                                }
                            }
                        }
                        // Stop propagation during drag to prevent text selection
                        cx.stop_propagation();
                    }
                    _ => {
                        // Update hover state
                        let thumb_bounds = state.read(cx).thumb_bounds;
                        let track_bounds = state.read(cx).track_bounds;

                        let is_over_scrollbar = thumb_bounds
                            .map(|b| b.contains(&event.position))
                            .unwrap_or(false)
                            || track_bounds
                                .map(|b| b.contains(&event.position))
                                .unwrap_or(false);

                        state.update(cx, |state, _| {
                            state.set_hovered(is_over_scrollbar);
                        });
                    }
                }
            });
        }

        // Mouse up handler
        {
            let state = state.clone();

            window.on_mouse_event(move |_event: &MouseUpEvent, event_phase, _window, cx| {
                if event_phase != phase {
                    return;
                }

                if state.read(cx).is_dragging() {
                    state.update(cx, |state, _| {
                        state.end_drag();
                    });
                    cx.stop_propagation();
                }
            });
        }
    }
}

impl IntoElement for ScrollbarElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

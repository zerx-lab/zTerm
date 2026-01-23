//! Terminal view component

use crate::elements::TerminalElement;
use crate::theme::TerminalTheme;
use axon_terminal::{Terminal, TerminalEvent};
use gpui::*;
use tracing::debug;

/// The main terminal view component
pub struct TerminalView {
    /// The terminal entity
    terminal: Entity<Terminal>,

    /// Focus handle for keyboard input
    focus_handle: FocusHandle,

    /// Current scroll offset (in pixels)
    scroll_offset: f32,

    /// Terminal theme
    theme: TerminalTheme,
}

impl TerminalView {
    /// Create a new terminal view
    pub fn new(terminal: Entity<Terminal>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Subscribe to terminal events
        cx.subscribe(&terminal, Self::on_terminal_event).detach();

        Self {
            terminal,
            focus_handle,
            scroll_offset: 0.0,
            theme: TerminalTheme::default(),
        }
    }

    /// Handle terminal events
    fn on_terminal_event(
        &mut self,
        _terminal: Entity<Terminal>,
        event: &TerminalEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            TerminalEvent::Output(_) => {
                cx.notify();
            }
            TerminalEvent::TitleChanged(title) => {
                debug!("Terminal title changed: {}", title);
                cx.notify();
            }
            TerminalEvent::Resized { cols, rows } => {
                debug!("Terminal resized: {}x{}", cols, rows);
                cx.notify();
            }
            TerminalEvent::ProcessExited { exit_code } => {
                debug!("Terminal process exited with code: {:?}", exit_code);
                cx.notify();
            }
            _ => {}
        }
    }

    /// Handle key down events
    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        debug!("Key down: {:?}", keystroke.key);

        // Convert keystroke to terminal input
        if let Some(input) = self.keystroke_to_input(keystroke) {
            debug!("Sending input to terminal: {:?}", input);
            self.terminal.update(cx, |terminal, _| {
                terminal.write(input.as_bytes());
            });
        }
    }

    /// Convert a keystroke to terminal input bytes
    fn keystroke_to_input(&self, keystroke: &Keystroke) -> Option<String> {
        // Handle special keys
        if keystroke.modifiers.control {
            if let Some(c) = keystroke.key.chars().next() {
                // Ctrl+A through Ctrl+Z
                if c.is_ascii_lowercase() {
                    let ctrl_char = (c as u8 - b'a' + 1) as char;
                    return Some(ctrl_char.to_string());
                }
                if c.is_ascii_uppercase() {
                    let ctrl_char = (c as u8 - b'A' + 1) as char;
                    return Some(ctrl_char.to_string());
                }
            }
        }

        // Handle other keys
        match keystroke.key.as_str() {
            "enter" => Some("\r".to_string()),
            "tab" => Some("\t".to_string()),
            "backspace" => Some("\x7f".to_string()),
            "escape" => Some("\x1b".to_string()),
            "up" => Some("\x1b[A".to_string()),
            "down" => Some("\x1b[B".to_string()),
            "right" => Some("\x1b[C".to_string()),
            "left" => Some("\x1b[D".to_string()),
            "home" => Some("\x1b[H".to_string()),
            "end" => Some("\x1b[F".to_string()),
            "pageup" => Some("\x1b[5~".to_string()),
            "pagedown" => Some("\x1b[6~".to_string()),
            "delete" => Some("\x1b[3~".to_string()),
            "insert" => Some("\x1b[2~".to_string()),
            "f1" => Some("\x1bOP".to_string()),
            "f2" => Some("\x1bOQ".to_string()),
            "f3" => Some("\x1bOR".to_string()),
            "f4" => Some("\x1bOS".to_string()),
            "f5" => Some("\x1b[15~".to_string()),
            "f6" => Some("\x1b[17~".to_string()),
            "f7" => Some("\x1b[18~".to_string()),
            "f8" => Some("\x1b[19~".to_string()),
            "f9" => Some("\x1b[20~".to_string()),
            "f10" => Some("\x1b[21~".to_string()),
            "f11" => Some("\x1b[23~".to_string()),
            "f12" => Some("\x1b[24~".to_string()),
            "space" => Some(" ".to_string()),
            key if key.len() == 1 => Some(key.to_string()),
            _ => None,
        }
    }

    /// Handle scroll wheel events
    fn on_scroll(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let delta = match event.delta {
            ScrollDelta::Lines(lines) => lines.y * 20.0,
            ScrollDelta::Pixels(pixels) => pixels.y.into(),
        };
        self.scroll_offset = (self.scroll_offset - delta).max(0.0);
        cx.notify();
    }

    /// Handle mouse down for focus
    fn on_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    /// Get the terminal entity
    pub fn terminal(&self) -> &Entity<Terminal> {
        &self.terminal
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let terminal = self.terminal.read(cx);
        let grid = terminal.grid();
        let theme = self.theme.clone();

        div()
            .id("terminal-view")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme.font_family.clone())
            .text_size(px(theme.font_size))
            .track_focus(&self.focus_handle)
            .key_context("Terminal")
            .on_key_down(cx.listener(Self::on_key_down))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p_2()
                    .child(Component::new(TerminalElement::new(grid, theme)))
            )
    }
}

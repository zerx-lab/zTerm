//! Terminal state and entity management using alacritty_terminal
//!
//! This module follows Zed's terminal architecture:
//! - Uses alacritty's EventLoop for PTY I/O (runs in background thread)
//! - Batches events with 4ms timer to reduce UI updates
//! - Syncs content only on Wakeup events

use crate::TerminalEvent;
use alacritty_terminal::event::{Event as AlacTermEvent, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point as AlacPoint};
use alacritty_terminal::selection::SelectionRange;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::term::{Config, RenderableCursor, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::Term;
use gpui::{AsyncApp, Bounds, Context, EventEmitter, Pixels, Size, Task, px};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Event batching interval in milliseconds (following Zed's approach)
const EVENT_BATCH_INTERVAL_MS: u64 = 4;

/// Terminal size in characters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

/// Terminal bounds with cell dimensions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalBounds {
    pub cell_width: Pixels,
    pub line_height: Pixels,
    pub bounds: Bounds<Pixels>,
}

impl Default for TerminalBounds {
    fn default() -> Self {
        Self {
            cell_width: px(8.0),
            line_height: px(16.0),
            bounds: Bounds {
                origin: gpui::Point::default(),
                size: Size {
                    width: px(640.0),
                    height: px(480.0),
                },
            },
        }
    }
}

impl TerminalBounds {
    pub fn new(line_height: Pixels, cell_width: Pixels, bounds: Bounds<Pixels>) -> Self {
        Self {
            cell_width,
            line_height,
            bounds,
        }
    }

    pub fn num_lines(&self) -> usize {
        (self.bounds.size.height / self.line_height).floor() as usize
    }

    pub fn num_columns(&self) -> usize {
        (self.bounds.size.width / self.cell_width).floor() as usize
    }
}

impl Dimensions for TerminalBounds {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        self.num_lines()
    }

    fn columns(&self) -> usize {
        self.num_columns()
    }
}

impl From<TerminalBounds> for WindowSize {
    fn from(val: TerminalBounds) -> Self {
        WindowSize {
            num_lines: val.num_lines() as u16,
            num_cols: val.num_columns() as u16,
            cell_width: f32::from(val.cell_width) as u16,
            cell_height: f32::from(val.line_height) as u16,
        }
    }
}

/// Event listener for alacritty terminal (sends events to our channel)
#[derive(Clone)]
pub struct TerminalEventListener(pub futures::channel::mpsc::UnboundedSender<AlacTermEvent>);

impl EventListener for TerminalEventListener {
    fn send_event(&self, event: AlacTermEvent) {
        let _ = self.0.unbounded_send(event);
    }
}

/// An indexed cell from the terminal grid
#[derive(Debug, Clone)]
pub struct IndexedCell {
    pub point: AlacPoint,
    pub cell: Cell,
}

/// Terminal content for rendering
#[derive(Clone)]
pub struct TerminalContent {
    pub cells: Vec<IndexedCell>,
    pub mode: TermMode,
    pub display_offset: usize,
    pub selection: Option<SelectionRange>,
    pub cursor: RenderableCursor,
    pub cursor_char: char,
    pub terminal_bounds: TerminalBounds,
    pub total_lines: usize,
    pub screen_lines: usize,
    pub history_size: usize,
}

impl Default for TerminalContent {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            mode: TermMode::empty(),
            display_offset: 0,
            selection: None,
            cursor: RenderableCursor {
                shape: alacritty_terminal::vte::ansi::CursorShape::Block,
                point: AlacPoint::new(Line(0), Column(0)),
            },
            cursor_char: ' ',
            terminal_bounds: TerminalBounds::default(),
            total_lines: 24,
            screen_lines: 24,
            history_size: 0,
        }
    }
}

/// The main Terminal entity using alacritty_terminal with EventLoop
pub struct Terminal {
    /// Alacritty terminal emulator (shared with EventLoop)
    term: Arc<FairMutex<Term<TerminalEventListener>>>,

    /// PTY notifier for sending data to PTY
    pty_tx: Option<Notifier>,

    /// Current size
    size: TerminalSize,

    /// Working directory
    working_directory: PathBuf,

    /// Terminal title
    title: String,

    /// Shell program being used
    shell: String,

    /// Whether the process has exited
    exited: bool,

    /// Last rendered content (cached for performance)
    last_content: TerminalContent,

    /// Event loop task (processes alacritty events)
    _event_loop_task: Task<anyhow::Result<()>>,
}

impl EventEmitter<TerminalEvent> for Terminal {}

impl Terminal {
    /// Create a new terminal with the given configuration
    pub fn new(
        shell: Option<String>,
        working_directory: Option<PathBuf>,
        size: TerminalSize,
        cx: &mut Context<Self>,
    ) -> Self {
        let working_directory = working_directory
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        // Create event channel for alacritty events
        let (events_tx, events_rx) = futures::channel::mpsc::unbounded();
        let listener = TerminalEventListener(events_tx.clone());

        // Create terminal config
        let config = Config {
            scrolling_history: 10000,
            ..Config::default()
        };

        // Create terminal bounds
        let bounds = TerminalBounds::default();

        // Create alacritty terminal
        let term = Term::new(config, &bounds, listener.clone());
        let term = Arc::new(FairMutex::new(term));

        // Setup PTY options - use our detect_shell function for proper defaults
        let shell_program = shell.clone().unwrap_or_else(crate::platform::detect_shell);

        let alac_shell = tty::Shell::new(shell_program.clone(), vec![]);
        let mut env: HashMap<String, String> = std::env::vars().collect();
        env.insert("TERM".to_string(), "xterm-256color".to_string());
        env.insert("COLORTERM".to_string(), "truecolor".to_string());

        let pty_options = tty::Options {
            shell: Some(alac_shell),
            working_directory: Some(working_directory.clone()),
            drain_on_exit: true,
            env: env.into_iter().collect(),
            #[cfg(windows)]
            escape_args: false,
        };

        // Create PTY using alacritty's tty module
        let pty = match tty::new(&pty_options, bounds.into(), 0) {
            Ok(pty) => pty,
            Err(e) => {
                error!("Failed to create PTY: {}", e);
                cx.emit(TerminalEvent::Error(format!("Failed to create PTY: {}", e)));
                // Return a dummy terminal that will show error
                return Self::create_error_terminal(working_directory, size, cx);
            }
        };

        // Create EventLoop (this handles PTY I/O in background thread)
        let event_loop = match EventLoop::new(
            term.clone(),
            listener,
            pty,
            pty_options.drain_on_exit,
            false,
        ) {
            Ok(el) => el,
            Err(e) => {
                error!("Failed to create event loop: {}", e);
                cx.emit(TerminalEvent::Error(format!("Failed to create event loop: {}", e)));
                return Self::create_error_terminal(working_directory, size, cx);
            }
        };

        let pty_tx = Notifier(event_loop.channel());
        let _io_thread = event_loop.spawn(); // Spawns background I/O thread

        info!("Terminal created with alacritty EventLoop");

        // Create event processing task (batches events like Zed does)
        let event_loop_task = Self::spawn_event_loop(events_rx, cx);

        Self {
            term,
            pty_tx: Some(pty_tx),
            size,
            working_directory,
            title: "Axon Terminal".to_string(),
            shell: shell_program,
            exited: false,
            last_content: TerminalContent::default(),
            _event_loop_task: event_loop_task,
        }
    }

    /// Create an error terminal (when PTY creation fails)
    fn create_error_terminal(
        working_directory: PathBuf,
        size: TerminalSize,
        _cx: &mut Context<Self>,
    ) -> Self {
        let (events_tx, _) = futures::channel::mpsc::unbounded();
        let listener = TerminalEventListener(events_tx);
        let config = Config::default();
        let bounds = TerminalBounds::default();
        let term = Term::new(config, &bounds, listener);
        let term = Arc::new(FairMutex::new(term));

        Self {
            term,
            pty_tx: None, // No PTY for error terminal
            size,
            working_directory,
            title: "Error".to_string(),
            shell: "unknown".to_string(),
            exited: true,
            last_content: TerminalContent::default(),
            _event_loop_task: Task::ready(Ok(())),
        }
    }

    /// Spawn event loop task that batches alacritty events (following Zed's pattern)
    /// 
    /// Optimization strategy:
    /// - Process first event immediately for lowest latency
    /// - Batch subsequent events within 4ms window to reduce UI updates
    /// - Pre-allocate event buffer to avoid repeated allocations
    /// - Coalesce multiple Wakeup events into one
    fn spawn_event_loop(
        mut events_rx: futures::channel::mpsc::UnboundedReceiver<AlacTermEvent>,
        cx: &mut Context<Self>,
    ) -> Task<anyhow::Result<()>> {
        use futures::StreamExt;

        cx.spawn(async move |terminal, cx: &mut AsyncApp| {
            // Pre-allocate buffer for batch events (reused across iterations)
            let mut batch_buffer: Vec<AlacTermEvent> = Vec::with_capacity(64);
            
            while let Some(event) = events_rx.next().await {
                // Process first event immediately for lower latency
                terminal.update(cx, |terminal, cx| {
                    terminal.process_event(event, cx);
                })?;

                // Batch subsequent events with 4ms timer (like Zed)
                'batch: loop {
                    batch_buffer.clear();
                    let mut wakeup = false;

                    let timer = futures::FutureExt::fuse(
                        smol::Timer::after(Duration::from_millis(EVENT_BATCH_INTERVAL_MS))
                    );
                    futures::pin_mut!(timer);

                    loop {
                        futures::select_biased! {
                            // Check for events first (biased) before timer
                            event = events_rx.next() => {
                                if let Some(event) = event {
                                    if matches!(event, AlacTermEvent::Wakeup) {
                                        // Coalesce multiple wakeups
                                        wakeup = true;
                                    } else {
                                        batch_buffer.push(event);
                                    }
                                    // Limit batch size to prevent UI starvation
                                    if batch_buffer.len() >= 100 {
                                        break;
                                    }
                                } else {
                                    // Channel closed
                                    break;
                                }
                            }
                            _ = timer => break,
                        }
                    }

                    // No events and no wakeup - exit batch loop immediately
                    if batch_buffer.is_empty() && !wakeup {
                        break 'batch;
                    }

                    // Process batched events
                    terminal.update(cx, |terminal, cx| {
                        // Process wakeup first (triggers content sync)
                        if wakeup {
                            terminal.process_event(AlacTermEvent::Wakeup, cx);
                        }
                        // Then process other events
                        for event in batch_buffer.drain(..) {
                            terminal.process_event(event, cx);
                        }
                    })?;

                    // Yield to allow UI to update
                    smol::future::yield_now().await;
                }
            }

            anyhow::Ok(())
        })
    }

    /// Process alacritty terminal event
    fn process_event(&mut self, event: AlacTermEvent, cx: &mut Context<Self>) {
        match event {
            AlacTermEvent::Wakeup => {
                // Sync content and notify UI (main update path)
                self.sync_content();
                cx.notify();
            }
            AlacTermEvent::Title(title) => {
                self.title = title.clone();
                cx.emit(TerminalEvent::TitleChanged(title));
            }
            AlacTermEvent::Bell => {
                // TODO: Handle bell
            }
            AlacTermEvent::Exit => {
                self.exited = true;
                cx.emit(TerminalEvent::ProcessExited { exit_code: None });
            }
            AlacTermEvent::ChildExit(code) => {
                self.exited = true;
                cx.emit(TerminalEvent::ProcessExited { exit_code: Some(code) });
            }
            AlacTermEvent::PtyWrite(data) => {
                self.write_to_pty(data.into_bytes());
            }
            AlacTermEvent::ClipboardStore(_, _) | AlacTermEvent::ClipboardLoad(_, _) => {
                // TODO: Handle clipboard
            }
            _ => {}
        }
    }

    /// Sync content from alacritty term for rendering
    /// Optimized to minimize lock hold time and memory allocations
    fn sync_content(&mut self) {
        // Take ownership of existing cells Vec to reuse its capacity
        let mut cells = std::mem::take(&mut self.last_content.cells);
        cells.clear();
        
        // Preserve terminal_bounds as it's set externally
        let terminal_bounds = self.last_content.terminal_bounds;

        // Minimize lock scope - extract all needed data in one pass
        let (mode, display_offset, selection, cursor, cursor_char, total_lines, screen_lines, history_size) = {
            let term = self.term.lock();
            let content = term.renderable_content();

            // Pre-allocate if needed (only grow, never shrink)
            let (capacity_hint, _) = content.display_iter.size_hint();
            if cells.capacity() < capacity_hint {
                cells.reserve(capacity_hint - cells.capacity());
            }

            // Collect cells while holding lock
            for ic in content.display_iter {
                cells.push(IndexedCell {
                    point: ic.point,
                    cell: ic.cell.clone(),
                });
            }

            let cursor_char = term.grid()[content.cursor.point].c;
            
            (
                content.mode,
                content.display_offset,
                content.selection,
                content.cursor,
                cursor_char,
                term.grid().total_lines(),
                term.grid().screen_lines(),
                term.history_size(),
            )
            // Lock released here
        };

        self.last_content = TerminalContent {
            cells,
            mode,
            display_offset,
            selection,
            cursor,
            cursor_char,
            terminal_bounds,
            total_lines,
            screen_lines,
            history_size,
        };
    }

    /// Write data to PTY
    /// Uses Cow::Owned since Msg::Input requires 'static lifetime
    #[inline(always)]
    fn write_to_pty(&self, data: Vec<u8>) {
        if let Some(ref pty_tx) = self.pty_tx {
            let _ = pty_tx.0.send(Msg::Input(std::borrow::Cow::Owned(data)));
        }
    }

    /// Write data to the terminal (keyboard input)
    #[inline(always)]
    pub fn write(&mut self, data: &[u8]) {
        // For typical keyboard input (1-10 bytes), Vec allocation is minimal
        self.write_to_pty(data.to_vec());
    }

    /// Write a string to the terminal
    #[inline(always)]
    pub fn write_str(&mut self, s: &str) {
        self.write_to_pty(s.as_bytes().to_vec());
    }

    /// Write owned bytes directly (avoids copy when caller already has Vec)
    #[inline(always)]
    pub fn write_owned(&mut self, data: Vec<u8>) {
        self.write_to_pty(data);
    }

    /// Resize the terminal
    pub fn resize(&mut self, size: TerminalSize, cx: &mut Context<Self>) {
        if self.size == size {
            return;
        }

        self.size = size;

        // Create new bounds
        let bounds = TerminalBounds {
            cell_width: self.last_content.terminal_bounds.cell_width,
            line_height: self.last_content.terminal_bounds.line_height,
            bounds: Bounds {
                origin: self.last_content.terminal_bounds.bounds.origin,
                size: Size {
                    width: self.last_content.terminal_bounds.cell_width * size.cols as f32,
                    height: self.last_content.terminal_bounds.line_height * size.rows as f32,
                },
            },
        };

        // Resize alacritty term
        {
            let mut term = self.term.lock();
            term.resize(bounds);
        }

        // Notify PTY of resize
        if let Some(ref pty_tx) = self.pty_tx {
            let _ = pty_tx.0.send(Msg::Resize(bounds.into()));
        }

        cx.emit(TerminalEvent::Resized {
            cols: size.cols as usize,
            rows: size.rows as usize,
        });
        cx.notify();
    }

    /// Set terminal bounds (for rendering calculations)
    pub fn set_bounds(&mut self, bounds: TerminalBounds) {
        self.last_content.terminal_bounds = bounds;

        let cols = bounds.num_columns() as u16;
        let rows = bounds.num_lines() as u16;

        if cols > 0 && rows > 0 && (cols != self.size.cols || rows != self.size.rows) {
            self.size = TerminalSize { cols, rows };

            {
                let mut term = self.term.lock();
                term.resize(bounds);
            }

            if let Some(ref pty_tx) = self.pty_tx {
                let _ = pty_tx.0.send(Msg::Resize(bounds.into()));
            }

            // Force immediate content sync after resize to prevent rendering artifacts
            // This ensures new size is reflected in content immediately, preventing element ghosting
            self.sync_content();
        }
    }

    /// Get the last rendered content
    pub fn content(&self) -> &TerminalContent {
        &self.last_content
    }

    /// Get the alacritty term (for advanced operations)
    pub fn term(&self) -> &Arc<FairMutex<Term<TerminalEventListener>>> {
        &self.term
    }

    /// Get the current size
    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// Get the terminal title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the terminal title
    pub fn set_title(&mut self, title: String, cx: &mut Context<Self>) {
        self.title = title.clone();
        cx.emit(TerminalEvent::TitleChanged(title));
        cx.notify();
    }

    /// Get the shell program being used
    pub fn shell(&self) -> &str {
        &self.shell
    }

    /// Get the shell name (without path)
    pub fn shell_name(&self) -> String {
        std::path::Path::new(&self.shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.shell)
            .to_string()
    }

    /// Check if the process has exited
    pub fn has_exited(&self) -> bool {
        self.exited
    }

    /// Get the working directory
    pub fn working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    /// Scroll the terminal by delta lines
    pub fn scroll(&mut self, delta: i32) {
        let mut term = self.term.lock();
        term.scroll_display(alacritty_terminal::grid::Scroll::Delta(delta));
        drop(term);
        self.sync_content();
    }

    /// Set absolute scroll offset (0 = bottom/newest, history_size = top/oldest)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        let mut term = self.term.lock();
        // Get current display offset
        let current_offset = term.grid().display_offset();
        // Calculate delta needed
        let delta = offset as i32 - current_offset as i32;
        if delta != 0 {
            term.scroll_display(alacritty_terminal::grid::Scroll::Delta(delta));
        }
        drop(term);
        self.sync_content();
    }

    /// Get selection text
    pub fn selection_text(&self) -> Option<String> {
        let term = self.term.lock();
        term.selection_to_string()
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        let mut term = self.term.lock();
        term.selection = None;
        drop(term);
        self.sync_content();
    }

    /// Check if terminal mode includes mouse reporting
    pub fn mouse_mode(&self) -> bool {
        self.last_content.mode.intersects(TermMode::MOUSE_MODE)
    }

    /// Check if in alternate screen mode
    pub fn alternate_screen(&self) -> bool {
        self.last_content.mode.contains(TermMode::ALT_SCREEN)
    }

    /// Check if cursor should be visible (DECTCEM mode)
    pub fn cursor_visible(&self) -> bool {
        self.last_content.mode.contains(TermMode::SHOW_CURSOR)
    }
}

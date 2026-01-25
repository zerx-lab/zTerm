//! Terminal state and entity management using alacritty_terminal
//!
//! This module follows Zed's terminal architecture with shell integration:
//! - Custom PTY I/O loop with OSC sequence interception
//! - Batches events with 4ms timer to reduce UI updates
//! - Syncs content only on Wakeup events
//! - Captures OSC 133/633 for shell integration

use super::pty_loop::{Msg as PtyMsg, Notifier as PtyNotifier, OscEvent, PtyEventLoop};
use crate::shell_integration::{OscSequence, ShellIntegrationHandler, ZoneManager};
use crate::TerminalEvent;
use alacritty_terminal::event::{Event as AlacTermEvent, EventListener, WindowSize};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Direction as AlacDirection, Line, Point as AlacPoint};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::term::{Config, RenderableCursor, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::Term;
use gpui::{AsyncApp, Bounds, Context, EventEmitter, Pixels, Size, Task, px};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
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
/// Note: scrollback_lines is configured separately in Terminal, not in bounds
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
    /// Returns the total number of lines in the terminal grid.
    ///
    /// IMPORTANT: Following Zed's approach, we return only screen_lines() here,
    /// NOT screen_lines + scrollback. This prevents alacritty from clearing
    /// history when the window is resized larger.
    ///
    /// Alacritty uses a separate internal scrollback buffer (max_scroll_limit)
    /// that is configured via Config, not through this Dimensions trait.
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

/// Zone info for rendering shell integration visuals
#[derive(Debug, Clone)]
pub struct ZoneInfo {
    /// Start line of the zone (in scrollback coordinates)
    pub start_line: usize,
    /// End line of the zone (exclusive, None if active)
    pub end_line: Option<usize>,
    /// Whether this is the prompt line
    pub is_prompt_line: bool,
    /// Whether the command is still running
    pub is_running: bool,
    /// Exit code if finished
    pub exit_code: Option<i32>,
    /// Command text if captured
    pub command: Option<String>,
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
    /// Zone information for visible lines (for shell integration rendering)
    pub zones: Vec<ZoneInfo>,
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
            zones: Vec::new(),
        }
    }
}

/// The main Terminal entity using alacritty_terminal with EventLoop
pub struct Terminal {
    /// Alacritty terminal emulator (shared with EventLoop)
    term: Arc<FairMutex<Term<TerminalEventListener>>>,

    /// PTY notifier for sending data to PTY
    pty_tx: Option<PtyNotifier>,

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

    /// Shell integration handler
    shell_handler: ShellIntegrationHandler,

    /// Current cursor line (for shell integration)
    current_line: usize,
}

impl EventEmitter<TerminalEvent> for Terminal {}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Send shutdown signal to PTY event loop thread
        if let Some(pty_tx) = &self.pty_tx {
            let _ = pty_tx.0.send(PtyMsg::Shutdown);
            info!(
                "Terminal dropped, shutdown signal sent to PTY thread for shell: {}",
                self.shell
            );
        }
    }
}

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

        // Get scrollback_lines from global config
        let scrollback_lines = {
            let config = zterm_common::Config::global();
            config.terminal.scrollback_lines
        };

        // Create terminal config with scrollback from settings
        let config = Config {
            scrolling_history: scrollback_lines,
            ..Config::default()
        };

        // Create terminal bounds
        let bounds = TerminalBounds::default();

        // Create alacritty terminal with scrollback configured via Config
        let term = Term::new(config, &bounds, listener.clone());
        let term = Arc::new(FairMutex::new(term));

        // Setup PTY options - use our detect_shell function for proper defaults
        let shell_program = shell.clone().unwrap_or_else(crate::platform::detect_shell);

        let alac_shell = tty::Shell::new(shell_program.clone(), vec![]);
        let mut env: HashMap<String, String> = std::env::vars().collect();
        env.insert("TERM".to_string(), "xterm-256color".to_string());
        env.insert("COLORTERM".to_string(), "truecolor".to_string());
        // Enable shell integration detection
        env.insert("ZTERM_SHELL_INTEGRATION".to_string(), "1".to_string());

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

        // Create OSC event channel for shell integration
        let (osc_tx, osc_rx) = mpsc::channel::<OscEvent>();

        // Create custom PTY EventLoop with OSC scanning
        let event_loop = match PtyEventLoop::new(
            term.clone(),
            listener,
            pty,
            pty_options.drain_on_exit,
            osc_tx,
        ) {
            Ok(el) => el,
            Err(e) => {
                error!("Failed to create event loop: {}", e);
                cx.emit(TerminalEvent::Error(format!("Failed to create event loop: {}", e)));
                return Self::create_error_terminal(working_directory, size, cx);
            }
        };

        let pty_tx = PtyNotifier(event_loop.channel());
        let _io_thread = event_loop.spawn(); // Spawns background I/O thread with OSC scanning

        info!("Terminal created with shell integration enabled (PTY scanning active)");

        // Create event processing task (batches events like Zed does)
        let event_loop_task = Self::spawn_event_loop(events_rx, osc_rx, cx);

        Self {
            term,
            pty_tx: Some(pty_tx),
            size,
            working_directory,
            title: "zTerm".to_string(),
            shell: shell_program,
            exited: false,
            last_content: TerminalContent::default(),
            _event_loop_task: event_loop_task,
            shell_handler: ShellIntegrationHandler::new(),
            current_line: 0,
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
            shell_handler: ShellIntegrationHandler::new(),
            current_line: 0,
        }
    }

    /// Spawn event loop task that batches alacritty events (following Zed's pattern)
    fn spawn_event_loop(
        mut events_rx: futures::channel::mpsc::UnboundedReceiver<AlacTermEvent>,
        osc_rx: mpsc::Receiver<OscEvent>,
        cx: &mut Context<Self>,
    ) -> Task<anyhow::Result<()>> {
        use futures::StreamExt;

        cx.spawn(async move |terminal, cx: &mut AsyncApp| {
            // Pre-allocate buffer for batch events (reused across iterations)
            let mut batch_buffer: Vec<AlacTermEvent> = Vec::with_capacity(64);

            while let Some(event) = events_rx.next().await {
                // Drain OSC events first (non-blocking)
                while let Ok(osc_event) = osc_rx.try_recv() {
                    terminal.update(cx, |terminal, _cx| {
                        terminal.handle_osc_sequence(osc_event.sequence, osc_event.line);
                    })?;
                }

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
                        // Drain OSC events (non-blocking)
                        while let Ok(osc_event) = osc_rx.try_recv() {
                            terminal.update(cx, |terminal, _cx| {
                                terminal.handle_osc_sequence(osc_event.sequence, osc_event.line);
                            })?;
                        }

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
    fn sync_content(&mut self) {
        // Take ownership of existing cells Vec to reuse its capacity
        let mut cells = std::mem::take(&mut self.last_content.cells);
        cells.clear();

        // Prevent unbounded memory growth: if capacity is excessive, shrink it
        // A typical terminal might have ~10,000 cells (100 cols x 100 rows)
        // We allow up to 100,000 cells before shrinking to avoid memory leaks
        const MAX_CELLS_CAPACITY: usize = 100_000;
        if cells.capacity() > MAX_CELLS_CAPACITY {
            cells.shrink_to(MAX_CELLS_CAPACITY / 2);
        }

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
            let history_size = term.history_size();

            // Update current line for shell integration
            self.current_line = content.cursor.point.line.0 as usize + history_size;
            self.shell_handler.set_current_line(self.current_line);

            // Debug: check cell line range
            let mut min_line = i32::MAX;
            let mut max_line = i32::MIN;
            for cell in &cells {
                min_line = min_line.min(cell.point.line.0);
                max_line = max_line.max(cell.point.line.0);
            }
            if !cells.is_empty() {
                eprintln!(
                    "[sync_content] Got {} cells, line range: {} to {}, display_offset: {}, history_size: {}",
                    cells.len(), min_line, max_line, content.display_offset, history_size
                );
            }

            (
                content.mode,
                content.display_offset,
                content.selection,
                content.cursor,
                cursor_char,
                term.grid().total_lines(),
                term.grid().screen_lines(),
                history_size,
            )
            // Lock released here
        };

        // Build zone info for visible lines
        let zones = self.build_zone_info(display_offset, screen_lines, history_size);

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
            zones,
        };
    }

    /// Build zone information for visible lines
    fn build_zone_info(
        &self,
        display_offset: usize,
        screen_lines: usize,
        history_size: usize,
    ) -> Vec<ZoneInfo> {
        let mut zones = Vec::new();
        let zone_manager = self.shell_handler.zone_manager();

        // Calculate visible line range in scrollback coordinates
        let first_visible = history_size.saturating_sub(display_offset);
        let last_visible = first_visible + screen_lines;

        // Find zones that intersect visible range
        for zone in zone_manager.zones() {
            let zone_end = zone.end_line.unwrap_or(usize::MAX);

            // Check if zone intersects visible range
            if zone.start_line < last_visible && zone_end > first_visible {
                zones.push(ZoneInfo {
                    start_line: zone.start_line,
                    end_line: zone.end_line,
                    is_prompt_line: true, // First line of zone is prompt
                    is_running: zone.state.is_running(),
                    exit_code: zone.state.exit_code(),
                    command: zone.command.clone(),
                });
            }
        }

        zones
    }

    /// Handle an OSC sequence (internal, without context)
    fn handle_osc_sequence(&mut self, seq: OscSequence, line: usize) {
        // Update current line for shell integration
        self.current_line = line;
        self.shell_handler.set_current_line(line);

        match seq {
            OscSequence::PromptStart => {
                self.shell_handler.handle_osc(b"133;A");
            }
            OscSequence::CommandStart => {
                self.shell_handler.handle_osc(b"133;B");
            }
            OscSequence::CommandExecuting => {
                self.shell_handler.handle_osc(b"133;C");
            }
            OscSequence::CommandFinished { exit_code } => {
                let osc = format!("133;D;{}", exit_code);
                self.shell_handler.handle_osc(osc.as_bytes());
            }
            OscSequence::CommandText { command } => {
                let encoded: String = command
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                            c.to_string()
                        } else {
                            format!("%{:02X}", c as u8)
                        }
                    })
                    .collect();
                let osc = format!("633;E;{}", encoded);
                self.shell_handler.handle_osc(osc.as_bytes());
            }
            OscSequence::WorkingDirectory { path } | OscSequence::Osc7WorkingDirectory { path } => {
                self.working_directory = PathBuf::from(&path);
                self.shell_handler.zone_manager_mut().set_working_directory(path);
            }
        }
    }

    /// Handle an OSC sequence with context (for external calls)
    pub fn handle_osc_sequence_with_cx(&mut self, seq: OscSequence, line: usize, cx: &mut Context<Self>) {
        self.handle_osc_sequence(seq, line);

        // Emit shell events
        for event in self.shell_handler.take_events() {
            cx.emit(TerminalEvent::ShellIntegration(event));
        }

        cx.notify();
    }

    /// Get the zone manager for external access
    pub fn zone_manager(&self) -> &ZoneManager {
        self.shell_handler.zone_manager()
    }

    /// Get the zone at a specific line
    pub fn zone_at_line(&self, line: usize) -> Option<ZoneInfo> {
        self.shell_handler.zone_manager().zone_at_line(line).map(|zone| {
            ZoneInfo {
                start_line: zone.start_line,
                end_line: zone.end_line,
                is_prompt_line: true,
                is_running: zone.state.is_running(),
                exit_code: zone.state.exit_code(),
                command: zone.command.clone(),
            }
        })
    }

    /// Navigate to previous prompt
    pub fn previous_prompt(&self) -> Option<usize> {
        self.shell_handler
            .zone_manager()
            .previous_prompt(self.current_line)
            .map(|z| z.start_line)
    }

    /// Navigate to next prompt
    pub fn next_prompt(&self) -> Option<usize> {
        self.shell_handler
            .zone_manager()
            .next_prompt(self.current_line)
            .map(|z| z.start_line)
    }

    /// Write data to PTY
    #[inline(always)]
    fn write_to_pty(&self, data: Vec<u8>) {
        if let Some(ref pty_tx) = self.pty_tx {
            let _ = pty_tx.0.send(PtyMsg::Input(std::borrow::Cow::Owned(data)));
        }
    }

    /// Write data to the terminal (keyboard input)
    #[inline(always)]
    pub fn write(&mut self, data: &[u8]) {
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
            eprintln!(
                "[resize] Before resize - total_lines: {}, screen_lines: {}, history_size: {}",
                term.grid().total_lines(),
                term.grid().screen_lines(),
                term.history_size()
            );
            term.resize(bounds);
            eprintln!(
                "[resize] After resize - total_lines: {}, screen_lines: {}, history_size: {}",
                term.grid().total_lines(),
                term.grid().screen_lines(),
                term.history_size()
            );
        }

        // Notify PTY of resize
        if let Some(ref pty_tx) = self.pty_tx {
            let _ = pty_tx.0.send(PtyMsg::Resize(bounds.into()));
        }

        cx.emit(TerminalEvent::Resized {
            cols: size.cols as usize,
            rows: size.rows as usize,
        });
        cx.notify();
    }

    /// Set terminal bounds (for rendering calculations)
    ///
    /// Following Zed's approach: only resize when bounds actually change.
    /// The PartialEq check on TerminalBounds prevents unnecessary resizes.
    pub fn set_bounds(&mut self, bounds: TerminalBounds) {
        // Check if bounds actually changed (防抖 - debounce)
        if self.last_content.terminal_bounds == bounds {
            return;
        }

        self.last_content.terminal_bounds = bounds;

        let new_cols = bounds.num_columns() as u16;
        let new_rows = bounds.num_lines() as u16;

        if new_cols == 0 || new_rows == 0 {
            return;
        }

        // Only resize if the logical size actually changed
        if new_cols == self.size.cols && new_rows == self.size.rows {
            return;
        }

        eprintln!(
            "[set_bounds] Resizing: {}x{} -> {}x{}",
            self.size.rows, self.size.cols, new_rows, new_cols
        );

        self.size = TerminalSize { cols: new_cols, rows: new_rows };

        // Resize terminal
        {
            let mut term = self.term.lock();
            term.resize(bounds);
        }

        // Notify PTY
        if let Some(ref pty_tx) = self.pty_tx {
            let _ = pty_tx.0.send(PtyMsg::Resize(bounds.into()));
        }

        // Sync content
        self.sync_content();
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
        let current_offset = term.grid().display_offset();
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

    /// Start a new selection at the given grid position
    ///
    /// # Arguments
    /// * `col` - Column position (0-based)
    /// * `row` - Row position relative to display (0-based, positive = visible screen)
    /// * `side` - Which side of the cell (Left or Right)
    /// * `selection_type` - Type of selection (Simple, Semantic, Lines)
    pub fn start_selection(
        &mut self,
        col: usize,
        row: i32,
        side: AlacDirection,
        selection_type: SelectionType,
    ) {
        let mut term = self.term.lock();
        let display_offset = term.grid().display_offset();
        // Convert row to alacritty's Line coordinate (accounting for display offset)
        let point = AlacPoint::new(Line(row - display_offset as i32), Column(col));
        let selection = Selection::new(selection_type, point, side);
        term.selection = Some(selection);
        drop(term);
        self.sync_content();
    }

    /// Update the current selection to extend to the given grid position
    ///
    /// # Arguments
    /// * `col` - Column position (0-based)
    /// * `row` - Row position relative to display (0-based, positive = visible screen)
    /// * `side` - Which side of the cell (Left or Right)
    pub fn update_selection(&mut self, col: usize, row: i32, side: AlacDirection) {
        let mut term = self.term.lock();
        if let Some(mut selection) = term.selection.take() {
            let display_offset = term.grid().display_offset();
            let point = AlacPoint::new(Line(row - display_offset as i32), Column(col));
            selection.update(point, side);
            term.selection = Some(selection);
        }
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

    /// Clear the terminal screen
    ///
    /// This clears the visible screen and moves cursor to top-left.
    /// Unlike sending escape sequences to PTY, this directly manipulates
    /// the terminal emulator state.
    pub fn clear_screen(&mut self) {
        use alacritty_terminal::vte::ansi::{ClearMode, Handler};

        {
            let mut term = self.term.lock();
            // Clear the entire screen
            term.clear_screen(ClearMode::All);
            // Move cursor to home position (line 0, column 0)
            term.goto(0, 0);
        }

        // Sync content to reflect changes
        self.sync_content();
    }
}

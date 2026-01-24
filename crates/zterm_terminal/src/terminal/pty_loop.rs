//! Custom PTY event loop with OSC sequence scanning
//!
//! This module provides a custom event loop that intercepts PTY output
//! to scan for OSC 133/633 shell integration sequences before passing
//! the data to the VTE parser.

use crate::shell_integration::{OscScanner, OscSequence};
use alacritty_terminal::event::{Event as AlacTermEvent, EventListener, OnResize, WindowSize};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use alacritty_terminal::tty::{self, ChildEvent, EventedPty};
use polling::{Event as PollingEvent, Events, PollMode};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::{self, ErrorKind, Read, Write};
use std::num::NonZeroUsize;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;
use tracing::error;
use vte::ansi;

/// Max bytes to read from the PTY before forced terminal synchronization.
const READ_BUFFER_SIZE: usize = 0x10_0000;

/// Max bytes to read from the PTY while the terminal is locked.
const MAX_LOCKED_READ: usize = u16::MAX as usize;

/// Messages that may be sent to the event loop.
#[derive(Debug)]
pub enum Msg {
    /// Data that should be written to the PTY.
    Input(Cow<'static, [u8]>),

    /// Indicates that the event loop should shut down.
    Shutdown,

    /// Instruction to resize the PTY.
    Resize(WindowSize),
}

/// OSC sequence event for shell integration
#[derive(Debug, Clone)]
pub struct OscEvent {
    pub sequence: OscSequence,
    pub line: usize,
}

/// Custom PTY event loop with OSC scanning capability.
///
/// This is based on alacritty's EventLoop but adds OSC 133/633 scanning
/// before passing data to the VTE parser.
pub struct PtyEventLoop<T: EventedPty, U: EventListener> {
    poll: Arc<polling::Poller>,
    pty: T,
    rx: PeekableReceiver<Msg>,
    tx: Sender<Msg>,
    terminal: Arc<FairMutex<Term<U>>>,
    event_proxy: U,
    drain_on_exit: bool,
    /// OSC scanner for shell integration
    scanner: OscScanner,
    /// Channel for sending OSC events
    osc_tx: Sender<OscEvent>,
    /// Current line number (for OSC events)
    current_line: usize,
}

impl<T, U> PtyEventLoop<T, U>
where
    T: EventedPty + OnResize + Send + 'static,
    U: EventListener + Send + 'static,
{
    /// Create a new PTY event loop with OSC scanning.
    pub fn new(
        terminal: Arc<FairMutex<Term<U>>>,
        event_proxy: U,
        pty: T,
        drain_on_exit: bool,
        osc_tx: Sender<OscEvent>,
    ) -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let poll = polling::Poller::new()?.into();

        Ok(Self {
            poll,
            pty,
            tx,
            rx: PeekableReceiver::new(rx),
            terminal,
            event_proxy,
            drain_on_exit,
            scanner: OscScanner::new(),
            osc_tx,
            current_line: 0,
        })
    }

    /// Get a sender for this event loop.
    pub fn channel(&self) -> EventLoopSender {
        EventLoopSender {
            sender: self.tx.clone(),
            poller: self.poll.clone(),
        }
    }

    /// Drain the receive channel.
    ///
    /// Returns `false` when a shutdown message was received.
    fn drain_recv_channel(&mut self, state: &mut State) -> bool {
        while let Some(msg) = self.rx.recv() {
            match msg {
                Msg::Input(input) => state.write_list.push_back(input),
                Msg::Resize(window_size) => self.pty.on_resize(window_size),
                Msg::Shutdown => return false,
            }
        }
        true
    }

    /// Read from PTY with OSC scanning.
    #[inline]
    fn pty_read(&mut self, state: &mut State, buf: &mut [u8]) -> io::Result<()> {
        let mut unprocessed = 0;
        let mut processed = 0;

        // Reserve the next terminal lock for PTY reading.
        let _terminal_lease = Some(self.terminal.lease());
        let mut terminal: Option<parking_lot::MutexGuard<'_, Term<U>>> = None;

        loop {
            // Read from the PTY.
            match self.pty.reader().read(&mut buf[unprocessed..]) {
                Ok(0) if unprocessed == 0 => break,
                Ok(got) => unprocessed += got,
                Err(err) => match err.kind() {
                    ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                        if unprocessed == 0 {
                            break;
                        }
                    }
                    _ => return Err(err),
                },
            }

            // === OSC SCANNING: Scan for OSC sequences before VTE parsing ===
            let data = &buf[..unprocessed];
            let sequences = self.scanner.scan(data);
            if !sequences.is_empty() {
                tracing::info!("OSC sequences detected: {:?}", sequences);
            }
            for seq in sequences {
                // Update current line for certain sequences
                if matches!(seq, OscSequence::CommandExecuting | OscSequence::PromptStart) {
                    // Get current cursor line from terminal if possible
                    if let Some(ref term) = terminal {
                        self.current_line = term.grid().cursor.point.line.0 as usize;
                    }
                }

                // Send OSC event
                let _ = self.osc_tx.send(OscEvent {
                    sequence: seq,
                    line: self.current_line,
                });
            }
            // === END OSC SCANNING ===

            // Attempt to lock the terminal.
            let terminal = match &mut terminal {
                Some(terminal) => terminal,
                None => terminal.insert(match self.terminal.try_lock_unfair() {
                    None if unprocessed >= READ_BUFFER_SIZE => self.terminal.lock_unfair(),
                    None => continue,
                    Some(terminal) => terminal,
                }),
            };

            // Parse the incoming bytes with VTE.
            state.parser.advance(&mut **terminal, &buf[..unprocessed]);

            processed += unprocessed;
            unprocessed = 0;

            // Assure we're not blocking the terminal too long unnecessarily.
            if processed >= MAX_LOCKED_READ {
                break;
            }
        }

        // Queue terminal redraw unless all processed bytes were synchronized.
        if state.parser.sync_bytes_count() < processed && processed > 0 {
            self.event_proxy.send_event(AlacTermEvent::Wakeup);
        }

        Ok(())
    }

    /// Write to PTY.
    #[inline]
    fn pty_write(&mut self, state: &mut State) -> io::Result<()> {
        state.ensure_next();

        'write_many: while let Some(mut current) = state.take_current() {
            'write_one: loop {
                match self.pty.writer().write(current.remaining_bytes()) {
                    Ok(0) => {
                        state.set_current(Some(current));
                        break 'write_many;
                    }
                    Ok(n) => {
                        current.advance(n);
                        if current.finished() {
                            state.goto_next();
                            break 'write_one;
                        }
                    }
                    Err(err) => {
                        state.set_current(Some(current));
                        match err.kind() {
                            ErrorKind::Interrupted | ErrorKind::WouldBlock => break 'write_many,
                            _ => return Err(err),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Spawn the event loop in a background thread.
    pub fn spawn(mut self) -> JoinHandle<(Self, State)> {
        std::thread::Builder::new()
            .name("PTY reader".into())
            .spawn(move || {
                let mut state = State::default();
                let mut buf = [0u8; READ_BUFFER_SIZE];

                let poll_opts = PollMode::Level;
                let mut interest = PollingEvent::readable(0);

                // Register TTY through EventedRW interface.
                if let Err(err) = unsafe { self.pty.register(&self.poll, interest, poll_opts) } {
                    error!("Event loop registration error: {err}");
                    return (self, state);
                }

                let mut events = Events::with_capacity(NonZeroUsize::new(1024).unwrap());

                'event_loop: loop {
                    // Wakeup the event loop when a synchronized update timeout was reached.
                    let handler = state.parser.sync_timeout();
                    let timeout = handler
                        .sync_timeout()
                        .map(|st| st.saturating_duration_since(Instant::now()));

                    events.clear();
                    if let Err(err) = self.poll.wait(&mut events, timeout) {
                        match err.kind() {
                            ErrorKind::Interrupted => continue,
                            _ => {
                                error!("Event loop polling error: {err}");
                                break 'event_loop;
                            }
                        }
                    }

                    // Handle synchronized update timeout.
                    if events.is_empty() && self.rx.peek().is_none() {
                        state.parser.stop_sync(&mut *self.terminal.lock());
                        self.event_proxy.send_event(AlacTermEvent::Wakeup);
                        continue;
                    }

                    // Handle channel events.
                    if !self.drain_recv_channel(&mut state) {
                        break;
                    }

                    for event in events.iter() {
                        match event.key {
                            tty::PTY_CHILD_EVENT_TOKEN => {
                                if let Some(ChildEvent::Exited(code)) = self.pty.next_child_event() {
                                    if let Some(code) = code {
                                        self.event_proxy.send_event(AlacTermEvent::ChildExit(code));
                                    }
                                    if self.drain_on_exit {
                                        let _ = self.pty_read(&mut state, &mut buf);
                                    }
                                    self.terminal.lock().exit();
                                    self.event_proxy.send_event(AlacTermEvent::Wakeup);
                                    break 'event_loop;
                                }
                            }

                            tty::PTY_READ_WRITE_TOKEN => {
                                if event.is_interrupt() {
                                    continue;
                                }

                                if event.readable {
                                    if let Err(err) = self.pty_read(&mut state, &mut buf) {
                                        #[cfg(target_os = "linux")]
                                        if err.raw_os_error() == Some(libc::EIO) {
                                            continue;
                                        }

                                        error!("Error reading from PTY in event loop: {err}");
                                        break 'event_loop;
                                    }
                                }

                                if event.writable {
                                    if let Err(err) = self.pty_write(&mut state) {
                                        error!("Error writing to PTY in event loop: {err}");
                                        break 'event_loop;
                                    }
                                }
                            }
                            _ => (),
                        }
                    }

                    // Register write interest if necessary.
                    let needs_write = state.needs_write();
                    if needs_write != interest.writable {
                        interest.writable = needs_write;
                        self.pty.reregister(&self.poll, interest, poll_opts).unwrap();
                    }
                }

                // Deregister before dropping.
                let _ = self.pty.deregister(&self.poll);

                (self, state)
            })
            .expect("failed to spawn PTY reader thread")
    }
}

/// Helper type which tracks how much of a buffer has been written.
struct Writing {
    source: Cow<'static, [u8]>,
    written: usize,
}

impl Writing {
    #[inline]
    fn new(c: Cow<'static, [u8]>) -> Writing {
        Writing { source: c, written: 0 }
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.written += n;
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        &self.source[self.written..]
    }

    #[inline]
    fn finished(&self) -> bool {
        self.written >= self.source.len()
    }
}

/// Notifier for sending messages to the event loop.
pub struct Notifier(pub EventLoopSender);

impl alacritty_terminal::event::Notify for Notifier {
    fn notify<B>(&self, bytes: B)
    where
        B: Into<Cow<'static, [u8]>>,
    {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return;
        }
        let _ = self.0.send(Msg::Input(bytes));
    }
}

impl OnResize for Notifier {
    fn on_resize(&mut self, window_size: WindowSize) {
        let _ = self.0.send(Msg::Resize(window_size));
    }
}

/// Sender for the event loop.
#[derive(Clone)]
pub struct EventLoopSender {
    sender: Sender<Msg>,
    poller: Arc<polling::Poller>,
}

impl EventLoopSender {
    pub fn send(&self, msg: Msg) -> Result<(), EventLoopSendError> {
        self.sender.send(msg).map_err(EventLoopSendError::Send)?;
        self.poller.notify().map_err(EventLoopSendError::Io)
    }
}

/// Error sending to the event loop.
#[derive(Debug)]
pub enum EventLoopSendError {
    Io(io::Error),
    Send(mpsc::SendError<Msg>),
}

impl std::fmt::Display for EventLoopSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventLoopSendError::Io(err) => err.fmt(f),
            EventLoopSendError::Send(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for EventLoopSendError {}

/// State for the event loop.
#[derive(Default)]
pub struct State {
    write_list: VecDeque<Cow<'static, [u8]>>,
    writing: Option<Writing>,
    parser: ansi::Processor,
}

impl State {
    #[inline]
    fn ensure_next(&mut self) {
        if self.writing.is_none() {
            self.goto_next();
        }
    }

    #[inline]
    fn goto_next(&mut self) {
        self.writing = self.write_list.pop_front().map(Writing::new);
    }

    #[inline]
    fn take_current(&mut self) -> Option<Writing> {
        self.writing.take()
    }

    #[inline]
    fn needs_write(&self) -> bool {
        self.writing.is_some() || !self.write_list.is_empty()
    }

    #[inline]
    fn set_current(&mut self, new: Option<Writing>) {
        self.writing = new;
    }
}

/// Peekable receiver for the event loop.
struct PeekableReceiver<T> {
    rx: Receiver<T>,
    peeked: Option<T>,
}

impl<T> PeekableReceiver<T> {
    fn new(rx: Receiver<T>) -> Self {
        Self { rx, peeked: None }
    }

    fn peek(&mut self) -> Option<&T> {
        if self.peeked.is_none() {
            self.peeked = self.rx.try_recv().ok();
        }
        self.peeked.as_ref()
    }

    fn recv(&mut self) -> Option<T> {
        if self.peeked.is_some() {
            self.peeked.take()
        } else {
            match self.rx.try_recv() {
                Err(TryRecvError::Disconnected) => panic!("event loop channel closed"),
                res => res.ok(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writing_new() {
        let w = Writing::new(Cow::Borrowed(b"hello"));
        assert_eq!(w.remaining_bytes(), b"hello");
        assert!(!w.finished());
    }

    #[test]
    fn test_writing_advance() {
        let mut w = Writing::new(Cow::Borrowed(b"hello"));
        w.advance(2);
        assert_eq!(w.remaining_bytes(), b"llo");
        w.advance(3);
        assert!(w.finished());
    }

    #[test]
    fn test_state_write_list() {
        let mut state = State::default();
        state.write_list.push_back(Cow::Borrowed(b"test"));
        assert!(state.needs_write());
        state.ensure_next();
        assert!(state.writing.is_some());
    }
}

//! Workspace management

use futures::FutureExt;
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace};
use zterm_terminal::{PtyConfig, Terminal, TerminalConfig, TerminalEvent, TerminalSize};
use zterm_ui::TerminalScrollHandle;

/// 事件批处理间隔（毫秒）
/// 参考 Zed 使用 4ms，这里使用 8ms 提供更好的批处理效果
const EVENT_BATCH_INTERVAL_MS: u64 = 8;

/// 每批次最大事件数量
const MAX_EVENTS_PER_BATCH: usize = 100;

#[cfg(feature = "shell-integration")]
use zterm_terminal::shell_integration::{BlockManager, CommandBlock};

#[cfg(feature = "shell-integration")]
use zterm_terminal::event::ShellIntegrationEvent;

/// Information about a tab
pub struct TabInfo {
    /// Tab ID
    pub id: usize,
    /// Tab title
    pub title: String,
    /// Terminal instance
    pub terminal: Arc<Terminal>,
    /// Scroll handle for scrollbar
    pub scroll_handle: TerminalScrollHandle,
    /// Block manager for shell integration
    #[cfg(feature = "shell-integration")]
    pub block_manager: Arc<parking_lot::Mutex<BlockManager>>,
}

/// Workspace containing multiple tabs
pub struct Workspace {
    /// Tabs in this workspace
    tabs: Vec<TabInfo>,

    /// Index of the active tab
    active_tab: usize,

    /// Counter for generating unique tab IDs
    next_tab_id: usize,
}

impl Workspace {
    /// Create a new workspace with an initial tab
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut workspace = Self {
            tabs: vec![],
            active_tab: 0,
            next_tab_id: 1,
        };

        // Create initial tab
        workspace.new_tab(cx);

        workspace
    }

    /// Create a new tab
    pub fn new_tab(&mut self, cx: &mut Context<Self>) {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        // 创建 PTY 配置
        #[cfg(target_os = "windows")]
        let (shell, shell_args) = Self::create_windows_shell_config();

        #[cfg(not(target_os = "windows"))]
        let (shell, shell_args) = (None, vec![]);

        let pty_config = PtyConfig {
            shell,
            shell_args,
            working_directory: None,
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
            initial_size: TerminalSize::new(24, 80),
        };

        // 创建终端配置
        let term_config = TerminalConfig::default();

        // 创建终端实例
        let terminal = match Terminal::new(pty_config, term_config) {
            Ok(term) => {
                info!("Terminal created successfully for tab {}", tab_id);
                Arc::new(term)
            }
            Err(e) => {
                error!("Failed to create terminal for tab {}: {}", tab_id, e);
                return;
            }
        };

        // 创建 BlockManager
        #[cfg(feature = "shell-integration")]
        let block_manager = Arc::new(parking_lot::Mutex::new(BlockManager::new()));

        // 创建滚动句柄
        let scroll_handle = TerminalScrollHandle::new(terminal.clone());

        let tab_info = TabInfo {
            id: tab_id,
            title: format!("Tab {}", tab_id),
            terminal: terminal.clone(),
            scroll_handle,
            #[cfg(feature = "shell-integration")]
            block_manager: block_manager.clone(),
        };

        self.tabs.push(tab_info);
        self.active_tab = self.tabs.len() - 1;

        // 启动事件监听
        #[cfg(feature = "shell-integration")]
        self.start_event_listener(terminal, block_manager, cx);

        #[cfg(not(feature = "shell-integration"))]
        self.start_event_listener(terminal, cx);

        debug!("Created new tab {} with terminal", tab_id);
        cx.notify();
    }

    /// Close the active tab
    /// Returns true if this was the last tab (window should close)
    pub fn close_active_tab(&mut self, cx: &mut Context<Self>) -> bool {
        if self.tabs.len() <= 1 {
            // This is the last tab, signal to close window
            return true;
        }

        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }

        debug!("Closed tab, active is now {}", self.active_tab);
        cx.notify();
        false
    }

    /// Close a specific tab
    /// Returns true if this was the last tab (window should close)
    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        if self.tabs.len() <= 1 {
            // This is the last tab, signal to close window
            return true;
        }

        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }

        cx.notify();
        false
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        cx.notify();
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
        cx.notify();
    }

    /// Set the active tab by index
    pub fn set_active_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.active_tab = index;
            cx.notify();
        }
    }

    /// Get the active tab index
    pub fn active_tab_index(&self) -> usize {
        self.active_tab
    }

    /// Get all tabs as TabInfo for UI display
    pub fn get_tab_infos(&self) -> Vec<zterm_ui::TabInfo> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| zterm_ui::TabInfo::new(tab.id, tab.title.clone(), i == self.active_tab))
            .collect()
    }

    /// Get the active tab's command blocks
    #[cfg(feature = "shell-integration")]
    pub fn active_tab_blocks(&self) -> Vec<CommandBlock> {
        self.tabs
            .get(self.active_tab)
            .map(|t| t.block_manager.lock().get_blocks().to_vec())
            .unwrap_or_default()
    }

    /// Get the active tab terminal
    pub fn active_terminal(&self) -> Option<Arc<Terminal>> {
        self.tabs.get(self.active_tab).map(|t| t.terminal.clone())
    }

    /// Get the active tab scroll handle
    pub fn active_scroll_handle(&self) -> Option<TerminalScrollHandle> {
        self.tabs
            .get(self.active_tab)
            .map(|t| t.scroll_handle.clone())
    }

    /// Create Windows PowerShell configuration with shell integration
    #[cfg(target_os = "windows")]
    fn create_windows_shell_config() -> (Option<PathBuf>, Vec<String>) {
        // 获取 shell integration 脚本路径
        let script_path = Self::get_shell_integration_script_path();

        // 如果脚本存在，注入加载命令
        if script_path.exists() {
            // 转换路径为标准格式（移除 \\?\ 前缀）
            let path_str = Self::normalize_windows_path(&script_path);
            info!("[Shell Integration] Injecting script: {}", path_str);

            // 使用 PowerShell 并注入脚本
            // 注意：PowerShell 的 -Command 参数中，单引号内的路径不需要额外转义反斜杠
            let load_script = format!(". '{}'; Clear-Host", path_str);

            (
                Some(PathBuf::from("pwsh.exe")),
                vec![
                    "-NoLogo".to_string(),
                    "-NoExit".to_string(),
                    "-Command".to_string(),
                    load_script,
                ],
            )
        } else {
            info!("[Shell Integration] Script not found at {:?}, using default shell", script_path);
            (None, vec![])
        }
    }

    /// Normalize Windows path by removing \\?\ prefix
    #[cfg(target_os = "windows")]
    fn normalize_windows_path(path: &PathBuf) -> String {
        let path_str = path.to_string_lossy().to_string();
        // 移除 Windows 扩展路径前缀 \\?\
        if path_str.starts_with(r"\\?\") {
            path_str[4..].to_string()
        } else {
            path_str
        }
    }

    /// Get shell integration script path
    #[cfg(target_os = "windows")]
    fn get_shell_integration_script_path() -> PathBuf {
        // 尝试从可执行文件相对路径查找
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let script = exe_dir.join("examples/shell-integration/zterm-integration.ps1");
                if script.exists() {
                    return script;
                }
            }
        }

        // 尝试从当前工作目录查找
        let script = PathBuf::from("examples/shell-integration/zterm-integration.ps1");
        if script.exists() {
            // 转换为绝对路径，PowerShell 需要绝对路径
            if let Ok(abs_path) = std::fs::canonicalize(&script) {
                return abs_path;
            }
            return script;
        }

        // 回退到用户目录
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("zterm/shell-integration.ps1")
        } else {
            PathBuf::from("zterm-integration.ps1")
        }
    }

    /// Start event listener for a terminal with event batching and throttling
    ///
    /// This implementation follows Zed's pattern:
    /// 1. Process the first event immediately for low latency
    /// 2. Then batch subsequent events for up to EVENT_BATCH_INTERVAL_MS
    /// 3. Merge multiple Wakeup events into a single refresh
    #[cfg(feature = "shell-integration")]
    fn start_event_listener(
        &self,
        terminal: Arc<Terminal>,
        block_manager: Arc<parking_lot::Mutex<BlockManager>>,
        cx: &mut Context<Self>,
    ) {
        let event_rx = terminal.event_receiver();

        cx.spawn(async move |_this, cx| {
            info!("[EventLoop] Started event listener with batching ({}ms interval)", EVENT_BATCH_INTERVAL_MS);

            loop {
                // Wait for the first event
                let first_event = match event_rx.recv_async().await {
                    Ok(event) => event,
                    Err(_) => {
                        debug!("[EventLoop] Terminal event channel closed");
                        break;
                    }
                };

                // Process the first event immediately for low latency
                let should_break = Self::process_single_event(
                    &first_event,
                    &terminal,
                    &block_manager,
                    &cx,
                );
                if should_break {
                    break;
                }

                // Now enter batching mode: collect events for up to EVENT_BATCH_INTERVAL_MS
                'batch_loop: loop {
                    let mut batched_events: Vec<TerminalEvent> = Vec::new();
                    let mut needs_wakeup = matches!(first_event, TerminalEvent::Wakeup);

                    // Create a timer for batching interval
                    let mut timer = cx
                        .background_executor()
                        .timer(Duration::from_millis(EVENT_BATCH_INTERVAL_MS))
                        .fuse();

                    // Collect events until timer expires or we hit the limit
                    loop {
                        futures::select_biased! {
                            _ = timer => {
                                // Timer expired, process collected events
                                break;
                            }
                            event = event_rx.recv_async().fuse() => {
                                match event {
                                    Ok(evt) => {
                                        // Merge Wakeup events - only need one refresh
                                        if matches!(evt, TerminalEvent::Wakeup) {
                                            needs_wakeup = true;
                                        } else {
                                            batched_events.push(evt);
                                        }

                                        // Stop batching if we have too many events
                                        if batched_events.len() >= MAX_EVENTS_PER_BATCH {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        debug!("[EventLoop] Channel closed during batching");
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Process all batched events
                    let event_count = batched_events.len();
                    let mut should_exit = false;

                    for event in batched_events {
                        if Self::process_single_event(&event, &terminal, &block_manager, &cx) {
                            should_exit = true;
                            break;
                        }
                    }

                    // Single refresh for all merged Wakeup events
                    if needs_wakeup && !should_exit {
                        trace!("[EventLoop] Batch refresh: {} events processed", event_count);
                        let _ = cx.update(|_cx: &mut App| {
                            _cx.refresh_windows();
                        });
                    }

                    if should_exit {
                        break 'batch_loop;
                    }

                    // Yield to allow other tasks to run
                    smol::future::yield_now().await;

                    // Check if there are more events to process
                    if event_rx.is_empty() {
                        break 'batch_loop;
                    }
                }
            }

            info!("[EventLoop] Event listener stopped");
        })
        .detach();
    }

    /// Process a single terminal event
    /// Returns true if the event loop should break (terminal exit/error)
    #[cfg(feature = "shell-integration")]
    fn process_single_event(
        event: &TerminalEvent,
        terminal: &Arc<Terminal>,
        block_manager: &Arc<parking_lot::Mutex<BlockManager>>,
        cx: &AsyncApp,
    ) -> bool {
        match event {
            TerminalEvent::PtyOutput(data) => {
                // Process PTY data through VTE parser
                trace!("[EventLoop] Processing {} bytes from PTY", data.len());
                terminal.process_pty_data(data);
            }
            TerminalEvent::PtyWrite(data) => {
                // Write terminal response to PTY (e.g., DSR response)
                trace!("[EventLoop] PtyWrite: {} bytes", data.len());
                if let Err(e) = terminal.write(data) {
                    error!("[EventLoop] Failed to write terminal response: {}", e);
                }
            }
            TerminalEvent::ShellIntegration(shell_event) => {
                if let ShellIntegrationEvent::RawOscSequence(osc_seq) = shell_event {
                    debug!("[Shell Hook] Received OSC sequence: {:?}", osc_seq);

                    // Update BlockManager
                    {
                        let mut mgr = block_manager.lock();
                        mgr.handle_osc_sequence(osc_seq);
                        trace!("[Shell Hook] BlockManager now has {} blocks", mgr.block_count());
                    }
                }
            }
            TerminalEvent::Wakeup => {
                // Wakeup events are batched and processed together
                // The actual refresh happens after batch processing
            }
            TerminalEvent::TitleChanged(title) => {
                debug!("[EventLoop] Terminal title changed: {}", title);
            }
            TerminalEvent::ProcessExit { exit_code } => {
                debug!("[EventLoop] Terminal process exited with code: {:?}", exit_code);
                return true;
            }
            TerminalEvent::Error(err) => {
                error!("[EventLoop] Terminal error: {}", err);
                return true;
            }
            _ => {}
        }
        false
    }

    /// Start event listener for a terminal (without shell integration)
    /// Uses the same batching and throttling mechanism as the shell-integration version
    #[cfg(not(feature = "shell-integration"))]
    fn start_event_listener(&self, terminal: Arc<Terminal>, cx: &mut Context<Self>) {
        let event_rx = terminal.event_receiver();

        cx.spawn(async move |_this, cx| {
            info!(
                "[EventLoop] Started event listener with batching ({}ms interval)",
                EVENT_BATCH_INTERVAL_MS
            );

            loop {
                // Wait for the first event
                let first_event = match event_rx.recv_async().await {
                    Ok(event) => event,
                    Err(_) => {
                        debug!("[EventLoop] Terminal event channel closed");
                        break;
                    }
                };

                // Process the first event immediately for low latency
                let should_break =
                    Self::process_single_event_no_si(&first_event, &terminal, &cx);
                if should_break {
                    break;
                }

                // Now enter batching mode
                'batch_loop: loop {
                    let mut batched_events: Vec<TerminalEvent> = Vec::new();
                    let mut needs_wakeup = matches!(first_event, TerminalEvent::Wakeup);

                    // Create a timer for batching interval
                    let mut timer = cx
                        .background_executor()
                        .timer(Duration::from_millis(EVENT_BATCH_INTERVAL_MS))
                        .fuse();

                    // Collect events until timer expires or we hit the limit
                    loop {
                        futures::select_biased! {
                            _ = timer => break,
                            event = event_rx.recv_async().fuse() => {
                                match event {
                                    Ok(evt) => {
                                        if matches!(evt, TerminalEvent::Wakeup) {
                                            needs_wakeup = true;
                                        } else {
                                            batched_events.push(evt);
                                        }
                                        if batched_events.len() >= MAX_EVENTS_PER_BATCH {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        debug!("[EventLoop] Channel closed during batching");
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Process all batched events
                    let event_count = batched_events.len();
                    let mut should_exit = false;

                    for event in batched_events {
                        if Self::process_single_event_no_si(&event, &terminal, &cx) {
                            should_exit = true;
                            break;
                        }
                    }

                    // Single refresh for all merged Wakeup events
                    if needs_wakeup && !should_exit {
                        trace!("[EventLoop] Batch refresh: {} events processed", event_count);
                        let _ = cx.update(|_cx: &mut App| {
                            _cx.refresh_windows();
                        });
                    }

                    if should_exit {
                        break 'batch_loop;
                    }

                    smol::future::yield_now().await;

                    if event_rx.is_empty() {
                        break 'batch_loop;
                    }
                }
            }

            info!("[EventLoop] Event listener stopped");
        })
        .detach();
    }

    /// Process a single terminal event (no shell integration version)
    #[cfg(not(feature = "shell-integration"))]
    fn process_single_event_no_si(
        event: &TerminalEvent,
        terminal: &Arc<Terminal>,
        _cx: &AsyncApp,
    ) -> bool {
        match event {
            TerminalEvent::PtyOutput(data) => {
                trace!("[EventLoop] Processing {} bytes from PTY", data.len());
                terminal.process_pty_data(data);
            }
            TerminalEvent::PtyWrite(data) => {
                trace!("[EventLoop] PtyWrite: {} bytes", data.len());
                if let Err(e) = terminal.write(data) {
                    error!("[EventLoop] Failed to write: {}", e);
                }
            }
            TerminalEvent::Wakeup => {
                // Batched - refresh happens after batch processing
            }
            TerminalEvent::ProcessExit { exit_code } => {
                debug!("[EventLoop] Process exited with code: {:?}", exit_code);
                return true;
            }
            TerminalEvent::Error(err) => {
                error!("[EventLoop] Error: {:?}", err);
                return true;
            }
            _ => {}
        }
        false
    }
}

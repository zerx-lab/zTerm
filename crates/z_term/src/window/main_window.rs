//! Main application window

use crate::app::{
    CloseActiveTab, CommandPalette, GotoTab1, GotoTab2, GotoTab3, GotoTab4, GotoTab5, GotoTab6,
    GotoTab7, GotoTab8, GotoTab9, NewTab, NextTab, PrevTab, Quit, ToggleFullscreen,
};
use crate::workspace::Workspace;
use axon_ui::ThemeContext;
use gpui::*;
use zterm_ui::{TerminalElement, TerminalView, TitleBar, TitleBarEvent};

/// The main application window
pub struct MainWindow {
    /// The workspace containing terminals
    workspace: Entity<Workspace>,

    /// Title bar entity
    title_bar: Entity<TitleBar>,

    /// Focus handle for terminal input
    focus_handle: FocusHandle,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let title_bar = cx.new(|_| TitleBar::new());

        // Subscribe to title bar events
        cx.subscribe(&title_bar, Self::handle_title_bar_event)
            .detach();

        let focus_handle = cx.focus_handle();

        Self {
            workspace,
            title_bar,
            focus_handle,
        }
    }

    /// Handle events from the title bar
    fn handle_title_bar_event(
        &mut self,
        _title_bar: Entity<TitleBar>,
        event: &TitleBarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            TitleBarEvent::NewTab => {
                self.workspace.update(cx, |ws, cx| {
                    ws.new_tab(cx);
                });
                // Scroll to show the new tab (last one)
                let active_index = self.workspace.read(cx).active_tab_index();
                self.title_bar.update(cx, |title_bar, _| {
                    title_bar.scroll_to_tab(active_index);
                });
            }
            TitleBarEvent::SelectTab(tab_index) => {
                let tab_index = *tab_index;
                self.workspace.update(cx, |ws, cx| {
                    ws.set_active_tab(tab_index, cx);
                });
                // Scroll to show the selected tab
                self.title_bar.update(cx, |title_bar, _| {
                    title_bar.scroll_to_tab(tab_index);
                });
            }
            TitleBarEvent::CloseTab(tab_index) => {
                let tab_index = *tab_index;
                let should_close_window = self
                    .workspace
                    .update(cx, |ws, cx| ws.close_tab(tab_index, cx));

                if should_close_window {
                    // Dispatch Quit action to close the app
                    cx.defer(|cx| {
                        cx.dispatch_action(&Quit);
                    });
                }
            }
        }
    }

    fn handle_new_tab(&mut self, _: &NewTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.new_tab(cx);
        });
        // Scroll to show the new tab (last one)
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
    }

    fn handle_close_active_tab(
        &mut self,
        _: &CloseActiveTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let should_close_window = self.workspace.update(cx, |ws, cx| ws.close_active_tab(cx));

        if should_close_window {
            window.remove_window();
        }
    }

    fn handle_next_tab(&mut self, _: &NextTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.next_tab(cx);
        });
        // Scroll to show the active tab
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
    }

    fn handle_prev_tab(&mut self, _: &PrevTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |ws, cx| {
            ws.prev_tab(cx);
        });
        // Scroll to show the active tab
        let active_index = self.workspace.read(cx).active_tab_index();
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.scroll_to_tab(active_index);
        });
    }

    fn handle_toggle_fullscreen(
        &mut self,
        _: &ToggleFullscreen,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        window.toggle_fullscreen();
    }

    fn handle_command_palette(
        &mut self,
        _: &CommandPalette,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // TODO: Implement command palette
        tracing::info!("Command palette not yet implemented");
    }

    /// Go to a specific tab by index (0-based)
    fn goto_tab(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        let tab_count = self.workspace.read(cx).get_tab_infos().len();
        if index < tab_count {
            self.workspace.update(cx, |ws, cx| {
                ws.set_active_tab(index, cx);
            });
            self.title_bar.update(cx, |title_bar, _| {
                title_bar.scroll_to_tab(index);
            });
        }
    }

    fn handle_goto_tab_1(&mut self, _: &GotoTab1, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(0, window, cx);
    }

    fn handle_goto_tab_2(&mut self, _: &GotoTab2, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(1, window, cx);
    }

    fn handle_goto_tab_3(&mut self, _: &GotoTab3, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(2, window, cx);
    }

    fn handle_goto_tab_4(&mut self, _: &GotoTab4, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(3, window, cx);
    }

    fn handle_goto_tab_5(&mut self, _: &GotoTab5, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(4, window, cx);
    }

    fn handle_goto_tab_6(&mut self, _: &GotoTab6, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(5, window, cx);
    }

    fn handle_goto_tab_7(&mut self, _: &GotoTab7, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(6, window, cx);
    }

    fn handle_goto_tab_8(&mut self, _: &GotoTab8, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_tab(7, window, cx);
    }

    fn handle_goto_tab_9(&mut self, _: &GotoTab9, window: &mut Window, cx: &mut Context<Self>) {
        // Ctrl+9 goes to the last tab (like browsers)
        let last_index = self
            .workspace
            .read(cx)
            .get_tab_infos()
            .len()
            .saturating_sub(1);
        self.goto_tab(last_index, window, cx);
    }

    /// Handle keyboard input for terminal
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 获取当前活动终端
        let terminal = match self.workspace.read(cx).active_terminal() {
            Some(term) => term,
            None => {
                tracing::warn!("No active terminal for key input");
                return;
            }
        };

        // 将按键转换为字节流
        let bytes = self.key_event_to_bytes(event);

        if !bytes.is_empty() {
            tracing::debug!(
                "Writing {} bytes to terminal: {:?}",
                bytes.len(),
                String::from_utf8_lossy(&bytes)
            );
            // 写入到 PTY
            if let Err(e) = terminal.write(&bytes) {
                tracing::error!("Failed to write to terminal: {} (bytes: {:?})", e, bytes);
            } else {
                tracing::debug!("Successfully wrote to terminal");
                // 用户输入时自动滚动到底部
                terminal.scroll_to_bottom();
                // 触发重绘
                cx.notify();
            }
        } else {
            tracing::trace!("Key event produced no bytes: {:?}", event.keystroke.key);
        }
    }

    /// Convert key event to bytes for PTY
    fn key_event_to_bytes(&self, event: &KeyDownEvent) -> Vec<u8> {
        let keystroke = &event.keystroke;

        // 处理特殊键 (GPUI 使用字符串名称而不是字符)
        match keystroke.key.as_str() {
            "space" => return vec![b' '],
            "enter" => return vec![b'\r'],
            "backspace" => return vec![0x7f], // DEL
            "tab" => return vec![b'\t'],
            "escape" => return vec![0x1b],
            "up" => return b"\x1b[A".to_vec(),
            "down" => return b"\x1b[B".to_vec(),
            "right" => return b"\x1b[C".to_vec(),
            "left" => return b"\x1b[D".to_vec(),
            "home" => return b"\x1b[H".to_vec(),
            "end" => return b"\x1b[F".to_vec(),
            "pageup" => return b"\x1b[5~".to_vec(),
            "pagedown" => return b"\x1b[6~".to_vec(),
            "delete" => return b"\x1b[3~".to_vec(),
            "insert" => return b"\x1b[2~".to_vec(),
            _ => {}
        }

        // 处理 Ctrl 组合键
        if keystroke.modifiers.control {
            if let Some(ch) = keystroke.key.chars().next() {
                if ch >= 'a' && ch <= 'z' {
                    // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                    let ctrl_code = (ch as u8) - b'a' + 1;
                    return vec![ctrl_code];
                } else if ch >= 'A' && ch <= 'Z' {
                    let ctrl_code = (ch as u8) - b'A' + 1;
                    return vec![ctrl_code];
                }
            }
        }

        // 普通字符输入 (没有修饰键，或只有 Shift)
        if !keystroke.modifiers.control && !keystroke.modifiers.alt && !keystroke.modifiers.platform
        {
            keystroke.key.as_bytes().to_vec()
        } else {
            Vec::new()
        }
    }

    /// 渲染终端内容 - 使用 TerminalElement（高性能 Element trait）
    fn render_terminal_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let terminal = workspace.active_terminal();
        let theme = cx.current_theme();
        let colors = &theme.colors;

        // 使用新的高性能 TerminalElement（基于 Element trait）
        // 相比 TerminalView (RenderOnce)，TerminalElement 具有：
        // - 批量文本渲染 (BatchedTextRun)
        // - 视口裁剪 (viewport culling)
        // - 直接绘制 (paint_quad, shape_line)
        let use_terminal_element = true;

        // 获取命令块（如果启用了 shell-integration）
        #[cfg(feature = "shell-integration")]
        let blocks = workspace.active_tab_blocks();

        // 调试：输出块数量信息
        #[cfg(feature = "shell-integration")]
        {
            use zterm_terminal::shell_integration::BlockState;
            let finished_count = blocks
                .iter()
                .filter(|b| b.state == BlockState::Finished)
                .count();
            let executing_count = blocks
                .iter()
                .filter(|b| b.state == BlockState::Executing)
                .count();
            if !blocks.is_empty() {
                tracing::debug!(
                    "[BlockRender] Blocks: total={}, finished={}, executing={}",
                    blocks.len(),
                    finished_count,
                    executing_count
                );
            }
        }

        // 外层容器：处理焦点和键盘输入
        // 使用 overflow_hidden 让终端填满空间，滚动由终端内部 scrollback 处理
        let container = div()
            .id("terminal-container")
            .flex_1()
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down));

        if let Some(term) = terminal {
            if use_terminal_element {
                // 使用新的 TerminalElement（高性能）
                let terminal_element =
                    TerminalElement::new(term.clone(), self.focus_handle.clone(), colors.clone());
                container.child(terminal_element)
            } else {
                // 使用旧的 TerminalView（RenderOnce）作为备选
                #[cfg(feature = "shell-integration")]
                {
                    use zterm_terminal::shell_integration::BlockState;
                    let has_finished_blocks =
                        blocks.iter().any(|b| b.state == BlockState::Finished);
                    let use_block_mode = false; // 禁用块渲染

                    if use_block_mode {
                        tracing::debug!("[BlockRender] Using BLOCK mode");
                    }

                    let view = TerminalView::new()
                        .terminal(term)
                        .blocks(blocks)
                        .block_mode(use_block_mode);
                    container.child(view)
                }

                #[cfg(not(feature = "shell-integration"))]
                {
                    container.child(TerminalView::new().terminal(term))
                }
            }
        } else {
            tracing::warn!("No active terminal in workspace!");
            container
                .flex()
                .items_center()
                .justify_center()
                .bg(colors.background.to_rgb())
                .child("No active terminal")
                .text_color(colors.text_muted.to_rgb())
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get theme colors
        let theme = cx.current_theme();
        let colors = &theme.colors;
        let statusbar_bg = colors.statusbar_background.to_rgb();
        let border_color = colors.border.to_rgb();
        let text_muted = colors.text_muted.to_rgb();

        // Get tab information from workspace
        let (tabs, active_tab_index, tab_count) = {
            let workspace = self.workspace.read(cx);
            let tabs = workspace.get_tab_infos();
            let active_tab_index = workspace.active_tab_index();
            let tab_count = tabs.len();
            (tabs, active_tab_index, tab_count)
        };

        // Update title bar tabs
        self.title_bar.update(cx, |title_bar, _| {
            title_bar.tabs = tabs;
        });

        div()
            .id("main-window")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1a1a1a))
            .text_color(rgb(0xe0e0e0))
            // NOTE: 不要在窗口根节点 track_focus。
            // 在 Windows 上，自绘 titlebar 依赖 WM_NCHITTEST -> HTCAPTION 的默认行为来触发系统拖拽。
            // 而 `track_focus` 会在 MouseDown 时自动 focus 并调用 window.prevent_default()，
            // 进而导致 WM_NCLBUTTONDOWN 被 GPUI 标记为 handled，阻止系统开始拖拽。
            .key_context("MainWindow")
            .on_action(cx.listener(Self::handle_new_tab))
            .on_action(cx.listener(Self::handle_close_active_tab))
            .on_action(cx.listener(Self::handle_next_tab))
            .on_action(cx.listener(Self::handle_prev_tab))
            .on_action(cx.listener(Self::handle_toggle_fullscreen))
            .on_action(cx.listener(Self::handle_command_palette))
            // Tab switching (Ctrl+1-9)
            .on_action(cx.listener(Self::handle_goto_tab_1))
            .on_action(cx.listener(Self::handle_goto_tab_2))
            .on_action(cx.listener(Self::handle_goto_tab_3))
            .on_action(cx.listener(Self::handle_goto_tab_4))
            .on_action(cx.listener(Self::handle_goto_tab_5))
            .on_action(cx.listener(Self::handle_goto_tab_6))
            .on_action(cx.listener(Self::handle_goto_tab_7))
            .on_action(cx.listener(Self::handle_goto_tab_8))
            .on_action(cx.listener(Self::handle_goto_tab_9))
            // Title bar with integrated tabs
            .child(self.title_bar.clone())
            // Content area - Terminal View
            .child(self.render_terminal_content(cx))
            // Status bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(24.0))
                    .px_2()
                    .bg(statusbar_bg)
                    .border_t_1()
                    .border_color(border_color)
                    .text_xs()
                    .text_color(text_muted)
                    .child(div().flex_1().child("zTerm"))
                    .child(div().child(format!("Tab {}/{}", active_tab_index + 1, tab_count))),
            )
    }
}

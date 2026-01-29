//! Terminal view component with block-based rendering
//!
//! Renders terminal output as command blocks similar to Warp
//!
//! ## Performance Optimizations
//!
//! Based on Zed and WezTerm analysis, this module implements:
//! - **BatchedTextRun**: Merges adjacent cells with same style into single spans
//! - Reduces GPUI element count by 90%+

use axon_ui::ThemeContext;
use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;

#[cfg(feature = "shell-integration")]
use zterm_terminal::shell_integration::{BlockState, CommandBlock};

use zterm_terminal::{Cell, CellAttributes, Color, CursorShape, Terminal};

// ============================================================================
// Batched Text Run (Performance Optimization)
// ============================================================================

/// Style key for comparing adjacent cells
/// Used to determine if cells can be merged into a single text run
#[derive(Clone, PartialEq)]
struct TextRunStyle {
    foreground: Rgba,
    background: Rgba,
    is_bold: bool,
    is_italic: bool,
    has_underline: bool,
    is_invisible: bool,
    is_dim: bool,
}

impl TextRunStyle {
    fn from_attrs(attrs: &CellAttributes, view: &TerminalView) -> Self {
        let reverse = attrs.reverse();
        let mut foreground = if reverse {
            view.color_to_rgba_simple(&attrs.background)
        } else {
            view.color_to_rgba_simple(&attrs.foreground)
        };
        let background = if reverse {
            view.color_to_rgba_simple(&attrs.foreground)
        } else {
            view.color_to_rgba_simple(&attrs.background)
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
            is_invisible: attrs.invisible(),
            is_dim: attrs.is_dim(),
        }
    }
}

/// A batched text run combining multiple adjacent cells with the same style
/// This reduces the number of GPUI elements dramatically
struct BatchedTextRun {
    /// Combined text from all cells
    text: String,
    /// Number of cells (for width calculation)
    cell_count: usize,
    /// Total width in cell units (handles wide characters)
    total_width: usize,
    /// Style for this run
    style: TextRunStyle,
    /// Starting column index
    start_col: usize,
}

impl BatchedTextRun {
    fn new(cell: &Cell, col: usize, style: TextRunStyle) -> Self {
        Self {
            text: cell.text().to_string(),
            cell_count: 1,
            total_width: cell.width() as usize,
            style,
            start_col: col,
        }
    }

    /// Check if a cell can be appended to this run
    fn can_append(&self, style: &TextRunStyle) -> bool {
        self.style == *style
    }

    /// Append a cell to this run
    fn append(&mut self, cell: &Cell) {
        self.text.push_str(cell.text());
        self.cell_count += 1;
        self.total_width += cell.width() as usize;
    }
}

/// Terminal view component
#[derive(IntoElement)]
pub struct TerminalView {
    /// Terminal 实例（用于直接渲染）
    terminal: Option<Arc<Terminal>>,

    /// 命令块列表
    #[cfg(feature = "shell-integration")]
    blocks: Vec<CommandBlock>,

    /// 是否启用块状渲染
    #[cfg(feature = "shell-integration")]
    block_mode: bool,
}

impl TerminalView {
    /// 创建新的终端视图
    pub fn new() -> Self {
        Self {
            terminal: None,
            #[cfg(feature = "shell-integration")]
            blocks: Vec::new(),
            #[cfg(feature = "shell-integration")]
            block_mode: false, // 默认使用直接渲染
        }
    }

    /// 设置 Terminal 实例（用于直接渲染）
    pub fn terminal(mut self, terminal: Arc<Terminal>) -> Self {
        self.terminal = Some(terminal);
        self
    }

    /// 设置命令块
    #[cfg(feature = "shell-integration")]
    pub fn blocks(mut self, blocks: Vec<CommandBlock>) -> Self {
        self.blocks = blocks;
        self
    }

    /// 设置块状渲染模式
    #[cfg(feature = "shell-integration")]
    pub fn block_mode(mut self, enabled: bool) -> Self {
        self.block_mode = enabled;
        self
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TerminalView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.current_theme();
        let colors = &theme.colors;

        // 优先级 1: 如果启用块模式且有 Terminal 实例，渲染命令块（包含底部输入区域）
        #[cfg(feature = "shell-integration")]
        {
            if self.block_mode {
                if let Some(ref terminal) = self.terminal {
                    return self.render_blocks(terminal, colors).into_any_element();
                }
            }
        }

        // 优先级 2: 如果有 Terminal 实例，直接渲染终端内容
        if let Some(terminal) = &self.terminal {
            return self
                .render_terminal_content(terminal, colors)
                .into_any_element();
        }

        // 默认渲染 (无数据)
        let bg = colors.background.to_rgb();
        let text_color = colors.text.to_rgb();

        div()
            .id("terminal-view")
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .child("Terminal View - Waiting for content...")
                    .text_color(text_color),
            )
            .into_any_element()
    }
}

/// 终端渲染配置
struct TerminalRenderConfig {
    /// 单元格宽度（像素）
    cell_width: f32,
    /// 行高（像素）
    line_height: f32,
    /// 字体大小
    font_size: f32,
}

impl Default for TerminalRenderConfig {
    fn default() -> Self {
        Self {
            cell_width: 8.4, // 等宽字体的标准宽度
            line_height: 18.0,
            font_size: 13.0,
        }
    }
}

impl TerminalView {
    /// 渲染终端内容（直接从 Terminal 读取）
    ///
    /// 渲染所有行（scrollback + viewport），让 GPUI 的滚动机制处理滚动
    fn render_terminal_content(
        &self,
        terminal: &Arc<Terminal>,
        colors: &axon_ui::theme::ThemeColors,
    ) -> impl IntoElement {
        // 从 Terminal 获取所有 Cell 数据（包括 scrollback）
        let (cells, scrollback_lines, viewport_rows) = terminal.get_all_cells();
        let cursor_info = terminal.cursor_info();
        let config = TerminalRenderConfig::default();

        let bg = colors.background.to_rgb();
        let cursor_color = colors.tab_active_indicator.to_rgb();

        // 计算光标在全部行中的实际位置（scrollback_lines + cursor.y）
        let cursor_row_in_all = scrollback_lines + cursor_info.y;

        // 调整光标信息以反映在所有行中的位置
        let adjusted_cursor = zterm_terminal::CursorInfo {
            x: cursor_info.x,
            y: cursor_row_in_all,
            visible: cursor_info.visible,
            shape: cursor_info.shape,
        };

        // 计算内容高度
        let total_rows = cells.len();
        let content_height = total_rows as f32 * config.line_height;

        div()
            .id("terminal-view")
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .overflow_y_scroll()
            .p_2()
            .font_family("Consolas")
            .text_size(px(config.font_size))
            // 内容容器
            .child(
                div()
                    .id("terminal-content")
                    .flex()
                    .flex_col()
                    .min_h(px(content_height))
                    .children(cells.into_iter().enumerate().map(|(row_idx, row_cells)| {
                        self.render_terminal_row(
                            row_cells,
                            row_idx,
                            &adjusted_cursor,
                            &config,
                            cursor_color,
                        )
                    })),
            )
    }

    /// 渲染终端行（使用批处理优化）
    ///
    /// 将相同样式的相邻字符合并为一个 span，大幅减少 GPUI 元素数量
    fn render_terminal_row(
        &self,
        cells: Vec<Cell>,
        row_idx: usize,
        cursor_info: &zterm_terminal::CursorInfo,
        config: &TerminalRenderConfig,
        cursor_color: Rgba,
    ) -> impl IntoElement {
        let is_cursor_row = cursor_info.visible && cursor_info.y == row_idx;
        let cursor_x = cursor_info.x;
        let cursor_shape = cursor_info.shape;

        // 批处理：合并相同样式的相邻字符
        let batched_runs = self.batch_cells(&cells, is_cursor_row, cursor_x);

        div()
            .flex()
            .flex_row()
            .h(px(config.line_height))
            .children(batched_runs.into_iter().map(|run| {
                // 检查这个批次是否包含光标
                let contains_cursor = is_cursor_row
                    && cursor_x >= run.start_col
                    && cursor_x < run.start_col + run.cell_count;

                if contains_cursor {
                    // 光标在这个批次中，需要拆分渲染
                    self.render_run_with_cursor(run, cursor_x, cursor_shape, cursor_color, config)
                } else {
                    // 无光标，直接渲染整个批次
                    self.render_batched_run(run, config).into_any_element()
                }
            }))
    }

    /// 将单元格批处理为文本运行
    fn batch_cells(
        &self,
        cells: &[Cell],
        is_cursor_row: bool,
        cursor_x: usize,
    ) -> Vec<BatchedTextRun> {
        let mut runs: Vec<BatchedTextRun> = Vec::with_capacity(cells.len() / 5); // 估计每 5 个字符一个批次

        for (col, cell) in cells.iter().enumerate() {
            let style = TextRunStyle::from_attrs(cell.attrs(), self);

            // 如果是光标位置，强制开始新批次（确保光标可以单独渲染）
            let force_new_run = is_cursor_row && (col == cursor_x || col == cursor_x + 1);

            if let Some(last_run) = runs.last_mut() {
                if !force_new_run && last_run.can_append(&style) {
                    last_run.append(cell);
                    continue;
                }
            }

            // 开始新批次
            runs.push(BatchedTextRun::new(cell, col, style));
        }

        runs
    }

    /// 渲染一个批处理的文本运行
    fn render_batched_run(&self, run: BatchedTextRun, config: &TerminalRenderConfig) -> Div {
        let width = run.total_width as f32 * config.cell_width;

        let mut el = div()
            .w(px(width))
            .h(px(config.line_height))
            .flex()
            .items_center();

        // 背景色
        if run.style.background.a > 0.0 {
            el = el.bg(run.style.background);
        }

        // 前景色
        if run.style.is_invisible {
            el = el.text_color(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            });
        } else {
            el = el.text_color(run.style.foreground);
        }

        // 文本样式
        if run.style.is_bold {
            el = el.font_weight(FontWeight::BOLD);
        }
        if run.style.is_italic {
            el = el.italic();
        }
        if run.style.has_underline {
            el = el.underline();
        }

        // 添加文本
        el.child(run.text)
    }

    /// 渲染包含光标的批次（需要拆分）
    fn render_run_with_cursor(
        &self,
        run: BatchedTextRun,
        cursor_x: usize,
        cursor_shape: CursorShape,
        cursor_color: Rgba,
        config: &TerminalRenderConfig,
    ) -> AnyElement {
        // 计算光标在批次内的相对位置
        let cursor_offset = cursor_x - run.start_col;

        // 将文本按字符拆分
        let chars: Vec<char> = run.text.chars().collect();

        // 创建三部分：光标前、光标、光标后
        let mut children: Vec<AnyElement> = Vec::with_capacity(3);

        // 光标前的文本
        if cursor_offset > 0 {
            let before_text: String = chars[..cursor_offset.min(chars.len())].iter().collect();
            if !before_text.is_empty() {
                let before_width = cursor_offset as f32 * config.cell_width;
                let mut el = div()
                    .w(px(before_width))
                    .h(px(config.line_height))
                    .flex()
                    .items_center()
                    .text_color(run.style.foreground);

                if run.style.background.a > 0.0 {
                    el = el.bg(run.style.background);
                }
                if run.style.is_bold {
                    el = el.font_weight(FontWeight::BOLD);
                }
                if run.style.is_italic {
                    el = el.italic();
                }

                children.push(el.child(before_text).into_any_element());
            }
        }

        // 光标位置的字符
        if cursor_offset < chars.len() {
            let cursor_char = chars[cursor_offset].to_string();
            let cursor_width = config.cell_width;

            let mut el = div()
                .w(px(cursor_width))
                .h(px(config.line_height))
                .flex()
                .items_center()
                .justify_center();

            if cursor_shape == CursorShape::Block {
                el = el.bg(cursor_color).text_color(Rgba {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
            } else {
                el = el.text_color(run.style.foreground);
                if run.style.background.a > 0.0 {
                    el = el.bg(run.style.background);
                }
                // 添加光标装饰
                el = self.add_cursor_decoration(el, cursor_shape, cursor_color, config);
            }

            if run.style.is_bold {
                el = el.font_weight(FontWeight::BOLD);
            }
            if run.style.is_italic {
                el = el.italic();
            }

            children.push(el.child(cursor_char).into_any_element());
        }

        // 光标后的文本
        if cursor_offset + 1 < chars.len() {
            let after_text: String = chars[cursor_offset + 1..].iter().collect();
            let after_width = (chars.len() - cursor_offset - 1) as f32 * config.cell_width;

            let mut el = div()
                .w(px(after_width))
                .h(px(config.line_height))
                .flex()
                .items_center()
                .text_color(run.style.foreground);

            if run.style.background.a > 0.0 {
                el = el.bg(run.style.background);
            }
            if run.style.is_bold {
                el = el.font_weight(FontWeight::BOLD);
            }
            if run.style.is_italic {
                el = el.italic();
            }

            children.push(el.child(after_text).into_any_element());
        }

        // 包装所有子元素
        div()
            .flex()
            .flex_row()
            .children(children)
            .into_any_element()
    }

    /// 简单的颜色转换（不依赖 cx）
    fn color_to_rgba_simple(&self, color: &Color) -> Rgba {
        match color {
            Color::DefaultForeground => Rgba {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
            Color::DefaultBackground => Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            Color::Indexed(idx) => self.indexed_to_rgba(*idx),
            Color::Rgb { r, g, b } => Rgba {
                r: *r as f32 / 255.0,
                g: *g as f32 / 255.0,
                b: *b as f32 / 255.0,
                a: 1.0,
            },
        }
    }

    /// 索引颜色转换
    fn indexed_to_rgba(&self, idx: u8) -> Rgba {
        // ANSI 16 色
        const ANSI_COLORS: [(u8, u8, u8); 16] = [
            (0, 0, 0),       // 0: Black
            (205, 49, 49),   // 1: Red
            (13, 188, 121),  // 2: Green
            (229, 229, 16),  // 3: Yellow
            (36, 114, 200),  // 4: Blue
            (188, 63, 188),  // 5: Magenta
            (17, 168, 205),  // 6: Cyan
            (229, 229, 229), // 7: White
            (102, 102, 102), // 8: Bright Black
            (241, 76, 76),   // 9: Bright Red
            (35, 209, 139),  // 10: Bright Green
            (245, 245, 67),  // 11: Bright Yellow
            (59, 142, 234),  // 12: Bright Blue
            (214, 112, 214), // 13: Bright Magenta
            (41, 184, 219),  // 14: Bright Cyan
            (255, 255, 255), // 15: Bright White
        ];

        if idx < 16 {
            let (r, g, b) = ANSI_COLORS[idx as usize];
            Rgba {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }
        } else if idx < 232 {
            // 216 色立方体
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;

            let component = |c: u8| -> f32 {
                if c == 0 {
                    0.0
                } else {
                    (55 + c * 40) as f32 / 255.0
                }
            };

            Rgba {
                r: component(r),
                g: component(g),
                b: component(b),
                a: 1.0,
            }
        } else {
            // 灰度
            let gray = idx - 232;
            let level = (8 + gray * 10) as f32 / 255.0;
            Rgba {
                r: level,
                g: level,
                b: level,
                a: 1.0,
            }
        }
    }

    /// 添加非块状光标装饰
    fn add_cursor_decoration(
        &self,
        el: Div,
        shape: CursorShape,
        color: Rgba,
        _config: &TerminalRenderConfig,
    ) -> Div {
        match shape {
            CursorShape::Underline => {
                // 下划线光标
                el.border_b_2().border_color(color)
            }
            CursorShape::Bar => {
                // 竖线光标
                el.border_l_2().border_color(color)
            }
            _ => el,
        }
    }
}

#[cfg(feature = "shell-integration")]
impl TerminalView {
    /// 渲染命令块列表（包含底部输入区域）
    fn render_blocks(
        &self,
        terminal: &Arc<Terminal>,
        colors: &axon_ui::theme::ThemeColors,
    ) -> impl IntoElement {
        let blocks = self.blocks.clone();
        let bg = colors.background.to_rgb();
        let text_color = colors.text.to_rgb();
        let text_muted = colors.text_muted.to_rgb();
        let border_color = colors.border.to_rgb();
        let success_color = gpui::rgb(0x22c55e); // green-500
        let error_color = gpui::rgb(0xef4444); // red-500
        let input_bg = colors.surface_background.to_rgb();

        div()
            .id("terminal-view")
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            // 渲染已完成的命令块
            .children(blocks.into_iter().map(|block| {
                self.render_command_block(
                    block,
                    text_color,
                    text_muted,
                    border_color,
                    success_color,
                    error_color,
                )
            }))
            // 底部输入区域
            .child(self.render_input_block(
                terminal,
                text_color,
                text_muted,
                border_color,
                input_bg,
            ))
    }

    /// 渲染底部输入块（当前命令输入区域）
    ///
    /// 渲染所有终端内容（scrollback + viewport），让 GPUI 处理滚动
    fn render_input_block(
        &self,
        terminal: &Arc<Terminal>,
        text_color: Rgba,
        text_muted: Rgba,
        border_color: Rgba,
        input_bg: Rgba,
    ) -> impl IntoElement {
        let config = TerminalRenderConfig::default();
        let cursor_info = terminal.cursor_info();
        let cursor_color = gpui::rgb(0x3b82f6); // blue-500

        // 获取所有终端内容（包括 scrollback）
        let (all_cells, scrollback_lines, _viewport_rows) = terminal.get_all_cells();

        // 渲染所有行（不再限制行数）
        let total_rows = all_cells.len();

        // 计算光标在全部行中的实际位置
        let cursor_row_in_all = scrollback_lines + cursor_info.y;

        // 计算内容高度
        let content_height = total_rows as f32 * config.line_height;

        div()
            .id("input-block")
            .flex()
            .flex_col()
            .flex_1() // 占据剩余空间
            .rounded_lg()
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            // 输入区域头部
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .bg(input_bg)
                    // 提示符图标
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_muted)
                            .font_weight(FontWeight::BOLD)
                            .child(">_"),
                    )
                    // 提示文字
                    .child(div().text_xs().text_color(text_muted).child("Terminal")),
            )
            // 输入区域内容：渲染所有行
            .child(
                div()
                    .id("terminal-input-content")
                    .px_3()
                    .py_2()
                    .bg(input_bg)
                    .flex_1()
                    .overflow_y_scroll() // 启用滚动
                    .font_family("Consolas")
                    .text_size(px(config.font_size))
                    .child(div().flex().flex_col().min_h(px(content_height)).children({
                        // 预先收集所有行数据避免生命周期问题
                        let rows: Vec<_> = (0..total_rows)
                            .map(|row_idx| {
                                let row_cells = all_cells[row_idx].clone();
                                // 光标位置使用调整后的行号
                                let is_cursor_row =
                                    cursor_info.visible && cursor_row_in_all == row_idx;
                                (row_cells, row_idx, is_cursor_row)
                            })
                            .collect();

                        rows.into_iter().map(|(row_cells, row_idx, is_cursor_row)| {
                            self.render_input_line_row(
                                row_cells,
                                row_idx,
                                is_cursor_row,
                                cursor_info.x,
                                &config,
                                cursor_color,
                                text_color,
                            )
                        })
                    })),
            )
    }

    /// 渲染输入行（带光标）
    fn render_input_line_row(
        &self,
        cells: Vec<Cell>,
        _row_idx: usize,
        is_cursor_row: bool,
        cursor_x: usize,
        config: &TerminalRenderConfig,
        cursor_color: Rgba,
        text_color: Rgba,
    ) -> impl IntoElement {
        let cells_len = cells.len();
        div()
            .flex()
            .flex_row()
            .h(px(config.line_height))
            .children(cells.into_iter().enumerate().map(|(col, cell)| {
                let is_cursor = is_cursor_row && col == cursor_x;
                let text = cell.text().to_string();
                let cell_width = cell.width() as f32 * config.cell_width;

                let mut el = div()
                    .w(px(cell_width))
                    .h(px(config.line_height))
                    .flex()
                    .items_center();

                if is_cursor {
                    // 光标位置
                    el = el.bg(cursor_color).text_color(Rgba {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    });
                } else {
                    el = el.text_color(text_color);
                }

                el.child(text)
            }))
            // 如果光标在行末，添加光标块
            .when(is_cursor_row && cursor_x >= cells_len, |div| {
                div.child(
                    gpui::div()
                        .w(px(config.cell_width))
                        .h(px(config.line_height))
                        .bg(cursor_color),
                )
            })
    }

    /// 渲染单个命令块
    fn render_command_block(
        &self,
        block: CommandBlock,
        text_color: Rgba,
        text_muted: Rgba,
        border_color: Rgba,
        success_color: Rgba,
        error_color: Rgba,
    ) -> impl IntoElement {
        let block_id = block.id.clone();
        let is_finished = block.state == BlockState::Finished;
        let is_success = block.is_success();

        // 状态指示器颜色
        let status_color = if !is_finished {
            text_muted // 执行中
        } else if is_success {
            success_color // 成功
        } else {
            error_color // 失败
        };

        div()
            .id(ElementId::Name(format!("block-{}", block_id).into()))
            .flex()
            .flex_col()
            .rounded_lg()
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            // 块头部 (命令行)
            .child(self.render_block_header(&block, text_color, text_muted, status_color))
            // 块内容 (输出)
            .when(!block.outputs.is_empty(), |div| {
                div.child(self.render_block_output(&block, text_color))
            })
            // 块尾部 (元信息)
            .when(is_finished, |div| {
                div.child(self.render_block_footer(&block, text_muted))
            })
    }

    /// 渲染块头部
    fn render_block_header(
        &self,
        block: &CommandBlock,
        text_color: Rgba,
        text_muted: Rgba,
        status_color: Rgba,
    ) -> impl IntoElement {
        let command_text = block
            .command
            .as_ref()
            .map(|cmd| {
                if !block.args.is_empty() {
                    format!("{} {}", cmd, block.args.join(" "))
                } else {
                    cmd.clone()
                }
            })
            .unwrap_or_else(|| "(no command)".to_string());

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(gpui::rgb(0x1e293b)) // slate-800
            // 状态指示器
            .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(status_color))
            // 工作目录
            .when_some(block.cwd.as_ref(), |this, cwd| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(text_muted)
                        .child(format!("{}$", cwd)),
                )
            })
            // 命令
            .child(
                div()
                    .text_sm()
                    .font_family(".SystemUIFont")
                    .text_color(text_color)
                    .child(command_text),
            )
            // 退出码
            .when_some(block.exit_code, |this, code| {
                this.child(
                    div()
                        .ml_auto()
                        .text_xs()
                        .text_color(if code == 0 {
                            gpui::rgb(0x22c55e) // green-500
                        } else {
                            gpui::rgb(0xef4444) // red-500
                        })
                        .child(format!("exit {}", code)),
                )
            })
    }

    /// 渲染块输出
    fn render_block_output(&self, block: &CommandBlock, text_color: Rgba) -> impl IntoElement {
        let output_text = block.get_output_text();

        div()
            .px_3()
            .py_2()
            .bg(gpui::rgb(0x0f172a)) // slate-900
            .child(
                div()
                    .text_sm()
                    .font_family(".SystemUIFont")
                    .text_color(text_color)
                    .child(output_text),
            )
    }

    /// 渲染块尾部
    fn render_block_footer(&self, block: &CommandBlock, text_muted: Rgba) -> impl IntoElement {
        let duration_text = block
            .duration_ms
            .map(|ms| {
                if ms < 1000 {
                    format!("{}ms", ms)
                } else {
                    format!("{:.2}s", ms as f64 / 1000.0)
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_4()
            .px_3()
            .py_1()
            .bg(gpui::rgb(0x1e293b)) // slate-800
            .text_xs()
            .text_color(text_muted)
            .child(format!("Duration: {}", duration_text))
            .when(!block.outputs.is_empty(), |div| {
                div.child(format!("Output: {} lines", block.outputs.len()))
            })
    }
}

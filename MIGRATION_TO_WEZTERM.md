# 从 Alacritty Terminal 迁移到 WezTerm 核心

## 迁移动机

### 当前问题（Alacritty Terminal）
- **历史记录丢失**：窗口调整大小时 `grid::resize::grow_lines()` 会调用 `decrease_scroll_limit()` 删除历史记录
- **不可避免**：这是 alacritty_terminal 的核心设计，`reflow` 参数只影响水平调整，无法阻止垂直调整时的历史删除
- **架构限制**：`total_lines()` 返回值无法改变这一行为

### WezTerm 的优势
- **完整的库化设计**：`wezterm-term` 专为作为库使用而设计，API 稳定且已发布到 crates.io
- **更好的 scrollback 管理**：使用 `VecDeque<Line>` 统一管理 scrollback + visible lines
- **稳定的索引系统**：`StableRowIndex` 设计确保行索引不受 scrollback 变化影响
- **丰富的功能**：支持 Sixel/Kitty 图像、OSC 8 超链接、双向文本等
- **主动维护**：由 WezTerm 项目维护，活跃度高

## 架构对比

### Alacritty Terminal 架构
```
Term<T: EventListener>
  ├── Grid<Cell>
  │   ├── raw: Storage<Row<Cell>>  (scrollback + visible)
  │   ├── lines: usize             (visible lines)
  │   └── resize() → grow_lines() → decrease_scroll_limit() ❌ 删除历史
  ├── Parser (vte crate)
  └── Config
```

### WezTerm-Term 架构
```
Terminal
  ├── TerminalState
  │   ├── Screen
  │   │   ├── lines: VecDeque<Line>  ✅ 统一管理所有行
  │   │   ├── physical_rows: usize   (可见行数)
  │   │   └── stable_row_index_offset: usize
  │   └── Parser (wezterm-escape-parser)
  ├── config: Arc<dyn TerminalConfiguration>
  └── writer: Arc<Mutex<dyn Write>>
```

### 关键差异

| 特性 | Alacritty | WezTerm |
|------|----------|---------|
| **Scrollback 管理** | `Grid::raw::shrink_lines()` 物理删除 | `Screen::lines` 保留所有行 |
| **行索引** | `PhysicalRowIndex` (不稳定) | `StableRowIndex` (稳定) |
| **Resize 行为** | `grow_lines()` 删除历史 | ✅ **保留历史** |
| **库化程度** | 可用但非主要设计目标 | ✅ 完全为库设计 |
| **API 稳定性** | 快速迭代 | ✅ 发布到 crates.io |

## 迁移范围

### 需要替换的依赖

```toml
# 移除：
alacritty_terminal = "0.25"
vte = "0.13"

# 添加：
wezterm-term = "0.1"
wezterm-escape-parser = "0.1"
wezterm-cell = "0.1"
wezterm-color-types = "0.1"
wezterm-input-types = "0.1"
vtparse = "0.7"  # 可选，如果需要低级解析
```

### 需要修改的文件

#### 核心文件（必须修改）
1. **`crates/zterm_terminal/Cargo.toml`** - 依赖替换
2. **`crates/zterm_terminal/src/terminal/state.rs`** - Terminal 实体核心
   - 替换 `alacritty_terminal::Term` → `wezterm_term::Terminal`
   - 适配 API 差异
3. **`crates/zterm_terminal/src/terminal/pty_loop.rs`** - PTY 事件循环
   - 替换事件类型
   - 适配 `advance_bytes()` API
4. **`crates/zterm_ui/src/elements/terminal_element.rs`** - 渲染元素
   - 替换 `RenderableContent` API
   - 适配单元格类型

#### 可能需要修改的文件
5. **`crates/zterm_terminal/src/buffer/cell.rs`** - 单元格类型（如果有自定义）
6. **`crates/zterm_terminal/src/shell_integration/scanner.rs`** - OSC 序列扫描（可能保留）
7. **`crates/zterm_ui/src/components/terminal_view.rs`** - 视图组件（API 适配）

#### 可能保留不变的文件
- **`crates/zterm_terminal/src/platform/*.rs`** - Shell 检测（独立于终端核心）
- **`crates/zterm_terminal/src/shell_integration/*.rs`** - Shell 集成（如果 wezterm 已支持则可移除）

## 迁移步骤

### Phase 1: 依赖准备（1-2 小时）

#### Step 1.1: 添加 WezTerm 依赖

编辑 `Cargo.toml`（workspace root）：

```toml
[workspace.dependencies]
# 移除或注释掉：
# alacritty_terminal = "0.25"
# vte = "0.13"

# 添加 WezTerm 核心库：
wezterm-term = "0.1"
wezterm-escape-parser = "0.1"
wezterm-cell = "0.1"
wezterm-color-types = "0.1"
wezterm-input-types = "0.1"
```

#### Step 1.2: 更新 `zterm_terminal/Cargo.toml`

```toml
[dependencies]
# Terminal parsing - 替换为 WezTerm
wezterm-term.workspace = true
wezterm-escape-parser.workspace = true
wezterm-cell.workspace = true
wezterm-color-types.workspace = true
wezterm-input-types.workspace = true

# PTY - 保留（WezTerm 也使用）
portable-pty.workspace = true

# 其他依赖保持不变
```

#### Step 1.3: 验证依赖

```bash
cargo check -p zterm_terminal
```

预期：大量编译错误（正常，下一步修复）

---

### Phase 2: 核心 API 替换（3-4 小时）

#### Step 2.1: 创建 WezTerm 配置适配器

创建 `crates/zterm_terminal/src/wezterm_config.rs`：

```rust
use wezterm_term::config::*;
use std::sync::Arc;

/// WezTerm 配置适配器
pub struct ZTermConfig {
    scrollback_lines: usize,
}

impl ZTermConfig {
    pub fn new(scrollback_lines: usize) -> Arc<Self> {
        Arc::new(Self { scrollback_lines })
    }
}

impl TerminalConfiguration for ZTermConfig {
    fn scrollback_size(&self) -> usize {
        self.scrollback_lines
    }

    fn color_palette(&self) -> ColorPalette {
        ColorPalette::default()
    }

    // 实现其他必需方法...
}
```

#### Step 2.2: 替换 `Terminal` 类型

在 `terminal/state.rs` 中：

```rust
// 旧代码：
use alacritty_terminal::Term;
use alacritty_terminal::sync::FairMutex;

// 新代码：
use wezterm_term::Terminal;
use parking_lot::Mutex;  // WezTerm 使用标准 Mutex

pub struct TerminalState {
    // 旧：term: Arc<FairMutex<Term<ZedListener>>>,
    // 新：
    term: Arc<Mutex<Terminal>>,
    // ...
}
```

#### Step 2.3: 适配终端初始化

```rust
// 旧代码（Alacritty）：
let term = Term::new(
    config,
    &TerminalBounds::default(),
    listener,
);

// 新代码（WezTerm）：
let size = TerminalSize {
    rows: bounds.num_lines(),
    cols: bounds.num_columns(),
    pixel_width: bounds.bounds.size.width.0 as usize,
    pixel_height: bounds.bounds.size.height.0 as usize,
    dpi: 96,  // 可从系统获取
};

let writer = Arc::new(Mutex::new(Vec::new()));  // PTY 写入器
let term = Terminal::new(
    size,
    Arc::new(ZTermConfig::new(10000)),
    "zterm",
    env!("CARGO_PKG_VERSION"),
    writer,
);
```

#### Step 2.4: 替换事件处理

```rust
// 旧代码（Alacritty EventListener）：
impl EventListener for ZedListener {
    fn send_event(&self, event: AlacTermEvent) {
        // ...
    }
}

// 新代码（WezTerm AlertHandler）：
use wezterm_term::{Alert, AlertHandler};

impl AlertHandler for ZTermAlertHandler {
    fn alert(&mut self, alert: Alert) {
        match alert {
            Alert::Bell => { /* ... */ },
            Alert::WindowTitleChanged(title) => { /* ... */ },
            Alert::PaletteChanged => { /* ... */ },
            // ...
        }
    }
}
```

---

### Phase 3: PTY 循环适配（2-3 小时）

#### Step 3.1: 修改 `advance_bytes()` 调用

在 `pty_loop.rs` 中：

```rust
// 旧代码（Alacritty）：
let mut processor = Processor::new();
processor.advance(&mut *term, &bytes);

// 新代码（WezTerm）：
term.advance_bytes(bytes);  // ✅ 更简单！
```

#### Step 3.2: 适配 Resize 事件

```rust
// 旧代码：
PtyMsg::Resize(bounds) => {
    term.resize(bounds);
}

// 新代码：
PtyMsg::Resize(bounds) => {
    let size = TerminalSize {
        rows: bounds.num_lines(),
        cols: bounds.num_columns(),
        pixel_width: bounds.bounds.size.width.0 as usize,
        pixel_height: bounds.bounds.size.height.0 as usize,
        dpi: 96,
    };
    term.resize(size);  // ✅ 不会丢失历史！
}
```

---

### Phase 4: 渲染适配（2-3 小时）

#### Step 4.1: 替换 RenderableContent

在 `terminal_element.rs` 中：

```rust
// 旧代码（Alacritty）：
let content = term.renderable_content();
for indexed_cell in content.display_iter {
    let cell = indexed_cell.cell;
    let point = indexed_cell.point;
    // ...
}

// 新代码（WezTerm）：
use wezterm_term::{StableRowIndex, VisibleRowIndex};

// 获取可见行范围
let screen_lines = term.screen().physical_rows;
let first_row = term.screen().phys_row(VisibleRowIndex::from(0));

for row_idx in 0..screen_lines {
    let line = term.screen().line(first_row + row_idx);
    for (col_idx, cell) in line.cells().enumerate() {
        // 渲染 cell
        render_cell(row_idx, col_idx, cell);
    }
}
```

#### Step 4.2: 适配单元格颜色

```rust
// 旧代码（Alacritty Cell）：
let fg = cell.fg;
let bg = cell.bg;

// 新代码（WezTerm Cell）：
use wezterm_cell::CellAttributes;

let attrs = cell.attrs();
let fg = attrs.foreground();  // 返回 ColorAttribute
let bg = attrs.background();

// 转换为 GPUI 颜色
let fg_color = color_from_wezterm(fg);
let bg_color = color_from_wezterm(bg);
```

---

### Phase 5: Shell 集成评估（1-2 小时）

#### Option A: 保留现有实现
如果 WezTerm 的 shell 集成不满足需求，可以保留 `shell_integration/scanner.rs`：

```rust
// 在 pty_loop.rs 中拦截 OSC 序列
let mut osc_scanner = OscScanner::new();
for byte in &bytes {
    if let Some(osc_event) = osc_scanner.advance(*byte) {
        // 处理 OSC 133/633
    }
}
term.advance_bytes(bytes);  // 然后发送给 WezTerm
```

#### Option B: 使用 WezTerm 内置支持
检查 `wezterm-term` 是否已支持 OSC 133/633：

```rust
// WezTerm 可能通过 Alert 系统发送
impl AlertHandler for ZTermAlertHandler {
    fn alert(&mut self, alert: Alert) {
        if let Alert::ShellIntegration(event) = alert {
            // 处理 shell 集成事件
        }
    }
}
```

---

### Phase 6: 测试与验证（2-3 小时）

#### Step 6.1: 编译测试

```bash
# 检查编译错误
cargo check --workspace --all-targets --all-features

# 修复所有错误后
cargo build -p z_term
```

#### Step 6.2: 功能测试

**测试 1: 基本输出**
```bash
cargo run -p z_term
# 在终端中运行：echo "Hello, WezTerm!"
```

**测试 2: 历史记录保留（核心测试！）**
```bash
# 在终端中运行：
1..200 | ForEach-Object { "Line $_" }

# 操作步骤：
1. 滚动到历史顶部，确认有 200 行
2. 调整窗口大小（变大）
3. 再次滚动到顶部
4. ✅ 验证：所有 200 行仍然存在
```

**测试 3: ANSI 颜色**
```bash
# 在终端中运行：
ls --color=always
```

**测试 4: 输入处理**
- 键盘输入
- 鼠标选择
- 剪贴板操作

**测试 5: Resize 稳定性**
- 快速调整窗口大小
- 最大化/恢复
- 检查光标位置

#### Step 6.3: 性能测试

```bash
# 运行基准测试
cargo bench -p zterm_terminal
```

对比 Alacritty vs WezTerm 的性能：
- 解析速度
- 渲染帧率
- 内存使用

---

## API 对照表

### 核心类型映射

| Alacritty | WezTerm | 说明 |
|-----------|---------|------|
| `Term<T>` | `Terminal` | 终端实例 |
| `Grid<Cell>` | `Screen` | 屏幕缓冲区 |
| `Cell` | `wezterm_cell::Cell` | 单元格 |
| `Config` | `dyn TerminalConfiguration` | 配置 trait |
| `EventListener` | `AlertHandler` | 事件处理 |
| `Event` | `Alert` | 事件类型 |
| `RenderableContent` | `Screen::lines()` | 渲染内容 |
| `Point` | `(VisibleRowIndex, Column)` | 坐标 |
| `Line` (行索引) | `StableRowIndex` | 行索引 |

### 方法映射

| Alacritty 方法 | WezTerm 方法 | 说明 |
|---------------|-------------|------|
| `term.resize(bounds)` | `term.resize(size)` | 调整大小 |
| `processor.advance(&mut term, bytes)` | `term.advance_bytes(bytes)` | 处理输入 |
| `term.renderable_content()` | `term.screen().lines()` | 获取渲染内容 |
| `term.grid().history_size()` | `term.screen().scrollback_rows()` | 历史行数 |
| `term.selection` | `term.get_selection_text()` | 选择文本 |
| `term.scroll_display(scroll)` | `term.perform_actions(...)` | 滚动 |

### 颜色类型映射

| Alacritty | WezTerm | 说明 |
|-----------|---------|------|
| `Color` | `ColorAttribute` | 颜色类型 |
| `Rgb` | `RgbColor` | RGB 颜色 |
| `Named(...)` | `ColorSpec::Default` | 命名颜色 |
| `Indexed(n)` | `ColorSpec::PaletteIndex(n)` | 索引颜色 |

---

## 风险评估

### 高风险项
1. **API 不兼容**：WezTerm API 与 Alacritty 差异较大，需要大量适配
   - **缓解**：创建适配层，逐步迁移

2. **性能回退**：WezTerm 可能比 Alacritty 慢
   - **缓解**：运行基准测试，优化热点路径

3. **Shell 集成丢失**：现有的 OSC 133/633 支持可能需要重写
   - **缓解**：先保留现有扫描器，后续评估 WezTerm 内置支持

### 中风险项
4. **渲染问题**：单元格属性、颜色映射可能不准确
   - **缓解**：详细测试 ANSI 颜色、样式

5. **输入处理**：键盘/鼠标事件编码差异
   - **缓解**：测试常见键盘快捷键

### 低风险项
6. **依赖冲突**：WezTerm crate 可能与现有依赖冲突
   - **缓解**：检查依赖树，必要时更新版本

---

## 回滚计划

如果迁移失败或遇到阻塞问题：

1. **保留 Git 分支**：在新分支上进行迁移
   ```bash
   git checkout -b migration/wezterm
   ```

2. **保留 Alacritty 代码**：注释而非删除
   ```rust
   // // 旧代码（Alacritty - 保留备份）：
   // use alacritty_terminal::Term;

   // 新代码（WezTerm）：
   use wezterm_term::Terminal;
   ```

3. **Feature Flag 切换**：可选实现双后端支持
   ```toml
   [features]
   default = ["wezterm-backend"]
   alacritty-backend = ["alacritty_terminal"]
   wezterm-backend = ["wezterm-term"]
   ```

---

## 时间估算

| 阶段 | 预计时间 | 关键产出 |
|------|---------|---------|
| Phase 1: 依赖准备 | 1-2 小时 | 依赖更新完成 |
| Phase 2: 核心 API 替换 | 3-4 小时 | `terminal/state.rs` 编译通过 |
| Phase 3: PTY 循环适配 | 2-3 小时 | PTY 事件循环正常工作 |
| Phase 4: 渲染适配 | 2-3 小时 | 终端内容正确显示 |
| Phase 5: Shell 集成评估 | 1-2 小时 | Shell 集成功能完整 |
| Phase 6: 测试与验证 | 2-3 小时 | 所有测试通过 |
| **总计** | **11-17 小时** | 完整迁移 |

---

## 预期成果

### 功能改进
✅ **历史记录完整保留**：窗口调整大小时不再丢失 scrollback
✅ **更稳定的 API**：使用 crates.io 发布的稳定库
✅ **更多功能**：支持 Sixel/Kitty 图像、OSC 8 超链接等

### 性能目标
- 解析速度：≥ Alacritty 的 95%
- 内存占用：≤ Alacritty 的 110%
- 渲染帧率：≥ 60 FPS

### 代码质量
- 减少自定义代码量（使用成熟库）
- 更清晰的架构分层
- 更好的可维护性

---

## 参考资源

### WezTerm 文档
- [WezTerm 官方文档](https://wezfurlong.org/wezterm/)
- [wezterm-term crate 文档](https://docs.rs/wezterm-term)
- [源码：C:\Users\zero\Desktop\code\github\wezterm](file:///C:/Users/zero/Desktop/code/github/wezterm)

### 关键源码文件
- `wezterm/term/src/terminal.rs` - Terminal API
- `wezterm/term/src/screen.rs` - Screen 实现
- `wezterm/term/src/terminalstate/mod.rs` - 状态机
- `wezterm/mux/src/localpane.rs` - LocalPane 示例

### Alacritty 对比参考
- [Alacritty Terminal 文档](https://docs.rs/alacritty_terminal)
- 当前实现：`crates/zterm_terminal/src/terminal/state.rs`

---

## 下一步行动

### 立即开始
1. 创建迁移分支：`git checkout -b migration/wezterm`
2. 备份当前代码：`git commit -am "Backup before WezTerm migration"`
3. 执行 Phase 1：更新依赖

### 验证迁移价值
在正式开始前，建议先做一个小型 PoC（概念验证）：
1. 创建独立测试项目
2. 集成 `wezterm-term`
3. 验证 resize 时历史记录确实保留
4. 评估性能和 API 可用性

### 获取帮助
如遇到问题：
- 查阅 WezTerm 源码示例（`mux/localpane.rs`）
- 参考 `wezterm-gui` 的集成方式
- 在 WezTerm GitHub Discussions 提问

---

**准备好开始迁移了吗？我可以帮你：**
1. 执行 Phase 1（更新依赖）
2. 创建 PoC 验证项目
3. 或先回答任何关于迁移的问题

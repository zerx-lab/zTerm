# Shell Integration 与 Grid 集成路线图

## 当前状态（2026-01-26）

❌ **完全未集成** - Shell Integration 事件有发送但无存储

### 问题
1. 没有 Terminal Grid/Screen 实现
2. 所有行号硬编码为 0
3. 没有语义类型存储
4. 无法查询语义区域

---

## Phase 1: 基础 Grid 实现 (优先级 P0)

**目标**: 实现最小可用的 Terminal Grid

### 1.1 数据结构

```rust
// grid/cell.rs
pub struct Cell {
    pub grapheme: String,       // UTF-8 字符
    pub attrs: CellAttributes,  // 属性
}

pub struct CellAttributes {
    // 基础属性
    pub foreground: Color,
    pub background: Color,
    pub flags: CellFlags,       // 粗体、斜体等
    pub semantic_type: SemanticType,  // ← Shell Integration
}

bitflags! {
    pub struct CellFlags: u16 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const UNDERLINE = 1 << 2;
        const REVERSE = 1 << 3;
    }
}

pub enum SemanticType {
    Output = 0,  // 默认
    Input = 1,
    Prompt = 2,
}
```

### 1.2 Line 实现

```rust
// grid/line.rs
pub struct Line {
    cells: Vec<Cell>,
    width: usize,
    wrapped: bool,  // 行尾是否被包装
}

impl Line {
    pub fn new(width: usize) -> Self;
    pub fn set_cell(&mut self, x: usize, cell: Cell);
    pub fn get_cell(&self, x: usize) -> &Cell;
    pub fn resize(&mut self, new_width: usize);
}
```

### 1.3 Screen 实现

```rust
// grid/screen.rs
use std::collections::VecDeque;

pub struct Screen {
    lines: VecDeque<Line>,
    physical_rows: usize,
    physical_cols: usize,
    scrollback_limit: usize,
}

impl Screen {
    pub fn new(rows: usize, cols: usize, scrollback: usize) -> Self;
    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell);
    pub fn get_line(&self, y: usize) -> Option<&Line>;
    pub fn scroll_up(&mut self, n: usize);
    pub fn resize(&mut self, rows: usize, cols: usize);
}
```

### 1.4 光标和状态

```rust
// grid/cursor.rs
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
}

// terminal.rs 添加
pub struct TerminalState {
    pub screen: Screen,
    pub cursor: Cursor,
    pub pen: CellAttributes,  // ← 当前笔属性（用于 Shell Integration）
}
```

---

## Phase 2: VTE Performer 集成 (优先级 P0)

**目标**: VtePerformer 能访问和修改 Grid

### 2.1 重构 VtePerformer

```rust
// vte_bridge.rs
pub struct VtePerformer {
    event_listener: Arc<dyn TerminalEventListener>,

    // ← 新增：访问 Terminal State
    terminal_state: Arc<Mutex<TerminalState>>,
}

impl Perform for VtePerformer {
    fn print(&mut self, c: char) {
        let mut state = self.terminal_state.lock();

        // 获取当前 pen（包含 SemanticType）
        let pen = state.pen.clone();

        // 创建 Cell
        let cell = Cell {
            grapheme: c.to_string(),
            attrs: pen,
        };

        // 写入 Grid
        state.screen.set_cell(state.cursor.x, state.cursor.y, cell);

        // 移动光标
        state.cursor.x += 1;
        if state.cursor.x >= state.screen.physical_cols {
            // 换行处理
            state.cursor.x = 0;
            state.cursor.y += 1;
        }
    }

    // ... 其他方法
}
```

### 2.2 OSC 133 处理更新

```rust
fn handle_osc_133(&mut self, params: &[&[u8]]) {
    let mut state = self.terminal_state.lock();

    match subcommand.as_ref() {
        "A" => {
            // 设置 pen 的语义类型为 Prompt
            state.pen.semantic_type = SemanticType::Prompt;

            // 发送事件（现在有真实行号）
            let line = state.cursor.y;
            self.event_listener.on_event(TerminalEvent::ShellIntegration(
                ShellIntegrationEvent::PromptStart { line }
            ));
        }
        "B" => {
            state.pen.semantic_type = SemanticType::Input;
            let line = state.cursor.y;
            // ...
        }
        "C" => {
            state.pen.semantic_type = SemanticType::Output;
            // ...
        }
        // ...
    }
}
```

---

## Phase 3: 语义区域查询 (优先级 P1)

**目标**: 支持 `get_semantic_zones()` API

### 3.1 Line 级别的 Zone 计算

```rust
// grid/line.rs
pub struct ZoneRange {
    pub start: usize,
    pub end: usize,
    pub semantic_type: SemanticType,
}

impl Line {
    pub fn semantic_zones(&self) -> Vec<ZoneRange> {
        let mut zones = Vec::new();
        let mut current_type = None;
        let mut start = 0;

        for (i, cell) in self.cells.iter().enumerate() {
            let cell_type = cell.attrs.semantic_type;

            if current_type != Some(cell_type) {
                if let Some(typ) = current_type {
                    zones.push(ZoneRange {
                        start,
                        end: i,
                        semantic_type: typ,
                    });
                }
                current_type = Some(cell_type);
                start = i;
            }
        }

        // 最后一个 zone
        if let Some(typ) = current_type {
            zones.push(ZoneRange {
                start,
                end: self.cells.len(),
                semantic_type: typ,
            });
        }

        zones
    }
}
```

### 3.2 Screen 级别的聚合

```rust
// terminal.rs
pub struct SemanticZone {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
    pub semantic_type: SemanticType,
}

impl TerminalState {
    pub fn get_semantic_zones(&self) -> Vec<SemanticZone> {
        let mut zones = Vec::new();
        let mut current_zone: Option<SemanticZone> = None;

        for (y, line) in self.screen.lines.iter().enumerate() {
            for zone_range in line.semantic_zones() {
                // 检查是否需要新建 zone
                let new_zone = match current_zone.as_ref() {
                    None => true,
                    Some(z) => z.semantic_type != zone_range.semantic_type,
                };

                if new_zone {
                    if let Some(z) = current_zone.take() {
                        zones.push(z);
                    }
                    current_zone = Some(SemanticZone {
                        start_x: zone_range.start,
                        start_y: y,
                        end_x: zone_range.end,
                        end_y: y,
                        semantic_type: zone_range.semantic_type,
                    });
                } else {
                    // 扩展当前 zone
                    if let Some(z) = current_zone.as_mut() {
                        z.end_x = zone_range.end;
                        z.end_y = y;
                    }
                }
            }
        }

        if let Some(z) = current_zone {
            zones.push(z);
        }

        zones
    }
}
```

---

## Phase 4: StableRowIndex (优先级 P2)

**目标**: 支持 Scrollback 后仍能引用语义区域

### 4.1 添加稳定行索引

```rust
pub struct Screen {
    lines: VecDeque<Line>,
    stable_row_index_offset: usize,  // ← 已删除的行数累积
    // ...
}

impl Screen {
    pub fn phys_to_stable(&self, phys: usize) -> usize {
        phys + self.stable_row_index_offset
    }

    pub fn stable_to_phys(&self, stable: usize) -> Option<usize> {
        if stable < self.stable_row_index_offset {
            None  // 已被删除
        } else {
            Some(stable - self.stable_row_index_offset)
        }
    }
}
```

### 4.2 SemanticZone 使用 StableRowIndex

```rust
pub struct SemanticZone {
    pub start_y: usize,  // StableRowIndex
    pub end_y: usize,    // StableRowIndex
    // ...
}
```

---

## 实现顺序建议

### 立即实现（本周）
1. ✅ Phase 1.1-1.3: 基础 Grid 数据结构
2. ✅ Phase 2.1: VtePerformer 访问 Grid
3. ✅ Phase 2.2: 修复 OSC 133 行号

### 短期实现（下周）
4. ⏳ Phase 1.4: 完整的 VTE 动作实现（光标移动、清屏等）
5. ⏳ Phase 3.1-3.2: 语义区域查询

### 中期实现（两周内）
6. ⏳ Phase 4: StableRowIndex 支持

---

## 测试策略

### 单元测试

```rust
#[test]
fn test_semantic_type_inheritance() {
    let mut state = TerminalState::new(24, 80);

    // 设置 pen 为 Prompt
    state.pen.semantic_type = SemanticType::Prompt;

    // 打印字符
    state.screen.set_cell(0, 0, Cell {
        grapheme: "$".to_string(),
        attrs: state.pen.clone(),
    });

    // 验证 cell 继承了语义类型
    let cell = state.screen.get_line(0).unwrap().get_cell(0);
    assert_eq!(cell.attrs.semantic_type, SemanticType::Prompt);
}

#[test]
fn test_semantic_zones() {
    // 模拟 Shell Integration 序列
    // OSC 133;A → Prompt
    // OSC 133;B → Input
    // OSC 133;C → Output

    let zones = state.get_semantic_zones();
    assert_eq!(zones.len(), 3);
    assert_eq!(zones[0].semantic_type, SemanticType::Prompt);
    assert_eq!(zones[1].semantic_type, SemanticType::Input);
    assert_eq!(zones[2].semantic_type, SemanticType::Output);
}
```

### 集成测试

```rust
#[test]
fn test_real_shell_integration_sequence() {
    let mut performer = VtePerformer::new(state, listener);

    // 发送真实的 OSC 133 序列
    performer.osc_dispatch(&[b"133", b"A"]);
    performer.print('$');
    performer.print(' ');

    performer.osc_dispatch(&[b"133", b"B"]);
    performer.print('l');
    performer.print('s');

    // 验证语义区域
    let zones = state.get_semantic_zones();
    // ...
}
```

---

## WezTerm 对比检查清单

- [ ] Cell 有 SemanticType
- [ ] Line 有 semantic_zones() 方法
- [ ] Screen 有 get_semantic_zones() API
- [ ] Performer 能访问 TerminalState
- [ ] OSC 133 修改 pen.semantic_type
- [ ] print() 继承 pen 属性到 cell
- [ ] 换行时清除语义类型（Input until EOL）
- [ ] 使用 StableRowIndex
- [ ] 支持 Scrollback

---

## 参考文档

- WezTerm 完整集成分析（见之前的 agent 输出）
- `term/src/terminalstate/performer.rs` - OSC 133 处理
- `wezterm-surface/src/line/line.rs` - Line.semantic_zones()
- `term/src/terminalstate/mod.rs` - get_semantic_zones()

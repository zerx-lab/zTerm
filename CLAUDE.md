# Axon Terminal - AI 项目记忆

## 项目概述

Axon Terminal 是一个使用 Rust + GPUI 框架构建的现代跨平台终端工具。目标是打造一个类似 Warp Terminal 的产品，具有优秀的 UI 设计和卓越的性能表现。

## 技术栈

- **语言**: Rust (edition 2024, rust-version 1.85+)
- **UI 框架**: GPUI (来自 Zed 编辑器)
- **组件库**: gpui-component (https://github.com/longbridge/gpui-component)
- **终端解析**: vte (VT 解析器)
- **PTY 抽象**: portable-pty
- **构建系统**: Cargo Workspace

## 开发偏好

- **依赖管理**: 优先使用 `cargo add` 添加依赖，避免手动指定可能过时的版本号
- **Edition**: 使用 Rust 2024 edition
- **代码风格**: 遵循 rustfmt 默认配置

## 项目结构

```
axon_term/
├── Cargo.toml                    # Workspace 根配置
├── CLAUDE.md                     # AI 记忆文档 (本文件)
├── assets/                       # 资源文件
│   ├── fonts/                    # 字体
│   ├── icons/                    # 图标
│   └── themes/                   # 主题配置
├── crates/
│   ├── axon_app/                 # 主应用入口
│   ├── axon_terminal/            # 终端核心逻辑
│   ├── axon_ui/                  # UI 组件
│   ├── axon_input/               # 输入处理
│   └── axon_common/              # 公共工具
└── docs/                         # 文档
```

## Crate 依赖关系

```
axon_app (主应用)
   ├── axon_ui
   ├── axon_input
   └── axon_terminal
        └── axon_common
```

## 开发命令

```bash
# 构建项目
cargo build

# 运行应用
cargo run -p axon_app

# 运行测试
cargo test --workspace

# 检查代码
cargo clippy --workspace

# 格式化代码
cargo fmt --all
```

## 实现阶段

### Phase 1: 项目基础设施 ✅ 完成
- [x] Workspace 初始化
- [x] 各 crate 配置
- [x] 应用骨架搭建
- [x] 基本终端窗口可以启动

### Phase 2: 终端核心
- [ ] PTY 抽象层 (Unix/Windows)
- [ ] VT 解析器封装
- [ ] Terminal Entity

### Phase 3: 终端渲染
- [ ] 低级渲染元素
- [ ] 终端视图组件
- [ ] 主题系统

### Phase 4: 输入处理
- [ ] 键盘输入映射
- [ ] 历史记录
- [ ] 自动补全

### Phase 5: 高级 UI
- [ ] 多 Tab/Pane
- [ ] Block-based UI
- [ ] 命令面板

### Phase 6: 优化与发布
- [ ] 性能优化
- [ ] 跨平台测试
- [ ] 打包发布

## 关键设计决策

1. **使用 Entity 模式**: Terminal 作为 GPUI Entity 管理状态
2. **低级 Element**: TerminalElement 直接实现 Element trait 以获得最佳性能
3. **异步 PTY**: 使用 cx.spawn 处理 PTY I/O
4. **虚拟化列表**: 只渲染可见行以优化性能

## 注意事项

- GPUI API 可能不稳定，锁定特定 commit
- Windows 需要使用 ConPTY (Windows 10 1809+)
- 使用 `WeakEntity` 避免循环引用
- 在 `request_layout` 阶段预计算，避免在 `paint` 阶段分配内存

## 当前进度

- **当前阶段**: Phase 1 完成，准备进入 Phase 2
- **已完成**: 项目可以编译运行，基本终端窗口可启动
- **下一步**: 完善 PTY 输出解析和终端渲染

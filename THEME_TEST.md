# 主题系统测试指南

## 快速测试

### 1. 编译应用
```bash
cargo build --release -p z_term
```

### 2. 运行应用
```bash
cargo run --release -p z_term
```

### 3. 修改配置

编辑配置文件 (Windows: `C:\Users\<username>\AppData\Roaming\zterm\config.toml`):

```toml
[ui]
theme = "GitHub Dark"  # 尝试这个
```

### 4. 查看效果

保存配置后，应该立即看到：

**GitHub Dark 主题颜色**:
- 背景: 非常深的灰色 (#0d1117)
- 文本: 浅灰色 (#c9d1d9)
- 红色: 柔和的红色 (#ff7b72)
- 绿色: 柔和的绿色 (#7ee787)
- 蓝色: 亮蓝色 (#58a6ff)

### 5. 切换到 GitHub Light

修改配置:
```toml
[ui]
theme = "GitHub Light"
```

应该看到：
- 背景: 白色 (#ffffff)
- 文本: 深灰色 (#24292f)
- 红色: 深红色 (#cf222e)
- 绿色: 深绿色 (#1a7f37)

### 6. 查看日志

应该看到类似的日志：

```
INFO zterm::app: Loading theme from config: GitHub Dark
INFO axon_ui::theme::manager: ThemeManager initialized with theme: Default Dark
INFO axon_ui::theme::manager: Theme changed to: GitHub Dark
INFO zterm::app: Theme changed to: GitHub Dark
INFO zterm_ui::components::terminal_view: Terminal theme updated from axon_ui theme system
```

## 组件使用情况

### ThemeManager (核心 - **正在使用**)
- `ThemeManager::init(cx)` - app.rs:63
- `ThemeManager::set_theme_by_name()` - app.rs:74

### ThemeContext (辅助 - **正在使用**)
- `cx.current_theme()` - terminal_theme.rs:218

### TerminalTheme 方法分析

#### 正在使用
- ✅ `from_axon_theme()` - 从新主题系统转换 (**主要入口**)
- ✅ `hsla_to_rgba()` - 颜色转换辅助函数

#### 可能不再需要 (保留作为 fallback)
- ⚠️ `from_config()` - 旧的配置加载方式
- ⚠️ `update_from_config()` - 旧的热重载方式
- ⚠️ `dark()`, `light()`, `dracula()`, `one_dark()`, `nord()` - 硬编码主题
- ⚠️ `matches_config()`, `matches_theme_colors()` - 旧的比较方法

## 建议的清理

### 方案 1: 保留旧方法作为 fallback
**优点**: 兼容性好，如果新系统失败可以fallback
**缺点**: 代码冗余

### 方案 2: 移除旧方法
**优点**: 代码简洁，单一责任
**缺点**: 需要确保新系统完全稳定

### 方案 3: 标记为 deprecated (推荐)
```rust
#[deprecated(note = "Use from_axon_theme instead")]
pub fn from_config(config: &Config) -> Self {
    // ...
}
```

**优点**: 给用户迁移时间
**缺点**: 仍有代码冗余

## 当前状态

### 正在使用的代码 ✅
- `axon_ui` crate - 完整的主题系统
- `ThemeManager` - 全局主题管理
- `ThemeContext` - 便捷访问trait
- `TerminalTheme::from_axon_theme()` - 颜色转换

### 可能冗余的代码 ⚠️
- `TerminalTheme::from_config()` - 旧的加载方式（未使用）
- `TerminalTheme::dark/light/dracula/...()` - 硬编码主题（未使用）
- `matches_config()` - 旧的比较方法（未使用）

## 验证步骤

1. ✅ 编译通过
2. ⏳ 运行应用并测试主题切换
3. ⏳ 验证所有 3 个主题显示正确
4. ⏳ 确认配置热重载工作
5. ⏳ 检查日志输出正确

## 下一步

根据测试结果决定：
- [ ] 移除未使用的旧方法
- [ ] 或者保留作为 fallback
- [ ] 或者标记为 deprecated

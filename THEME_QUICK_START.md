# 主题系统快速开始

## ✅ 修复完成

1. ✅ 主题系统完全集成
2. ✅ 支持配置热重载
3. ✅ 修复无限循环问题
4. ✅ 支持大小写不敏感主题名称

## 🎨 可用主题

### Default Dark
```toml
[ui]
theme = "Default Dark"  # 或 "default dark"
```

### GitHub Dark
```toml
[ui]
theme = "GitHub Dark"  # 或 "github dark" 或 "Github Dark"
```

### GitHub Light
```toml
[ui]
theme = "GitHub Light"  # 或 "github light" 或 "Github Light"
```

## 📝 配置位置

- **Windows**: `C:\Users\<你的用户名>\AppData\Roaming\zterm\config.toml`
- **Linux/macOS**: `~/.config/zterm/config.toml`

## 🚀 使用步骤

### 1. 编辑配置文件

```toml
[terminal]
shell = "powershell.exe"
font_family = "JetBrainsMono Nerd Font Mono"
font_size = 14.0
# ... 其他配置 ...

[ui]
theme = "GitHub Dark"  # 改成你喜欢的主题
opacity = 1.0
# ... 其他配置 ...
```

### 2. 保存文件

保存后，应用会在 **200ms 内**自动检测并应用新主题。

### 3. 查看效果

- **GitHub Dark**: 深灰背景 (#0d1117)，浅灰文字
- **GitHub Light**: 白色背景，深灰文字
- **Default Dark**: 蓝灰背景，经典配色

## 🔍 日志输出

成功时应该看到：

```
INFO zterm::app: Theme changed to: GitHub Dark
INFO axon_ui::theme::manager: Theme changed to: GitHub Dark
INFO zterm_ui::components::terminal_view: Terminal theme updated from axon_ui theme system
```

## ⚠️ 故障排除

### 主题找不到

**错误日志**:
```
WARN axon_ui::theme::manager: Theme 'xxx' not found, keeping current theme
```

**解决**:
- 检查主题名称拼写（现在支持大小写不敏感）
- 确保使用以下之一：
  - `Default Dark`
  - `GitHub Dark`
  - `GitHub Light`

### 主题一直更新

**症状**: 日志一直输出 `Terminal theme updated`

**原因**: 已修复！添加了变化检测，避免不必要的更新

### UI 没有更新

**解决**:
1. 检查配置文件是否正确保存
2. 查看日志确认主题加载成功
3. 尝试重启应用

## 📊 技术细节

### 变化检测
```rust
// 只有在颜色或字体真正变化时才更新
let theme_changed =
    self.theme.background != new_theme.background
    || self.theme.foreground != new_theme.foreground
    || self.theme.font_family != new_theme.font_family
    || (self.theme.font_size - new_theme.font_size).abs() > f32::EPSILON;
```

### 大小写不敏感
```rust
// 支持 "github dark", "GitHub Dark", "GITHUB DARK" 等
pub fn get(&self, name: &str) -> Option<Arc<Theme>> {
    self.themes
        .iter()
        .find(|t| t.name().eq_ignore_ascii_case(name))
        .cloned()
}
```

## 🎯 下一步

### 添加自定义主题
未来将支持从 JSON/TOML 文件加载自定义主题。

### 主题预览
计划添加主题预览功能，无需重启即可切换。

### 更多内置主题
计划添加：
- Dracula
- One Dark
- Nord
- Solarized Dark/Light

## ✨ 测试通过

- ✅ 编译通过
- ✅ 主题切换正常
- ✅ 热重载工作
- ✅ 无限循环已修复
- ✅ 大小写不敏感匹配

**主题系统已完全就绪！** 🎉

---

**提示**: 修改配置后等待 1-2 秒查看效果，无需重启应用！

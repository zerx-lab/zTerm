//! 主题上下文扩展
//!
//! 提供便捷的方法在 GPUI 上下文中访问主题

use super::Theme;
use gpui::App;
use std::sync::Arc;

/// 主题访问 trait
///
/// 为 GPUI App 提供便捷的主题访问方法
pub trait ThemeContext {
    /// 获取当前主题
    fn current_theme(&self) -> Arc<Theme>;
}

impl ThemeContext for App {
    fn current_theme(&self) -> Arc<Theme> {
        super::manager::ThemeManager::current_theme(self)
    }
}

#[cfg(test)]
mod tests {
    // 注意：这些测试需要 GPUI 运行时环境，这里仅作为接口验证
    // 实际测试会在集成测试中进行
}

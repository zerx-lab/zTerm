//! 右键上下文菜单组件
//!
//! 提供终端的右键菜单功能，支持复制、粘贴等操作。

use crate::shell_integration::ContextMenuAction;

/// 上下文菜单项
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuItem {
    /// 菜单项的操作类型
    action: ContextMenuAction,
    /// 菜单项的显示标签
    label: String,
    /// 是否启用（可点击）
    enabled: bool,
}

impl ContextMenuItem {
    /// 创建新的菜单项
    pub fn new(action: ContextMenuAction, label: impl Into<String>) -> Self {
        Self {
            action,
            label: label.into(),
            enabled: true,
        }
    }

    /// 设置菜单项的启用状态
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 获取菜单项的操作
    pub fn action(&self) -> &ContextMenuAction {
        &self.action
    }

    /// 获取菜单项的标签
    pub fn label(&self) -> &str {
        &self.label
    }

    /// 检查菜单项是否可用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 上下文菜单状态
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// 菜单是否可见
    visible: bool,
    /// 菜单位置 (x, y)
    position: (f32, f32),
    /// 是否有选择的文本
    has_selection: bool,
    /// 菜单项列表
    items: Vec<ContextMenuItem>,
}

impl ContextMenuState {
    /// 创建新的上下文菜单状态
    pub fn new() -> Self {
        Self {
            visible: false,
            position: (0.0, 0.0),
            has_selection: false,
            items: Self::create_default_items(false),
        }
    }

    /// 创建默认的菜单项
    fn create_default_items(has_selection: bool) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::new(ContextMenuAction::Copy, "复制").enabled(has_selection),
            ContextMenuItem::new(ContextMenuAction::Paste, "粘贴").enabled(true),
        ]
    }

    /// 显示菜单
    pub fn show(&mut self, x: f32, y: f32) {
        self.visible = true;
        self.position = (x, y);
    }

    /// 隐藏菜单
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 检查菜单是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 获取菜单位置
    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    /// 设置是否有选择的文本
    pub fn set_has_selection(&mut self, has_selection: bool) {
        self.has_selection = has_selection;
        // 更新菜单项的启用状态
        self.items = Self::create_default_items(has_selection);
    }

    /// 检查是否有选择的文本
    pub fn has_selection(&self) -> bool {
        self.has_selection
    }

    /// 获取所有菜单项
    pub fn items(&self) -> &[ContextMenuItem] {
        &self.items
    }

    /// 根据操作类型查找菜单项
    pub fn find_item(&self, action: ContextMenuAction) -> Option<&ContextMenuItem> {
        self.items.iter().find(|item| *item.action() == action)
    }

    /// 检查是否可以执行指定的操作
    pub fn can_execute_action(&self, action: ContextMenuAction) -> bool {
        match action {
            ContextMenuAction::Copy => self.has_selection,
            ContextMenuAction::Paste => true,
            _ => false,
        }
    }

    /// 执行操作（执行后菜单会关闭）
    pub fn execute_action(&mut self, action: ContextMenuAction) {
        if self.can_execute_action(action) {
            self.hide();
        }
    }
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_item_creation() {
        let item = ContextMenuItem::new(ContextMenuAction::Copy, "复制");
        assert_eq!(item.label(), "复制");
        assert!(item.is_enabled());
    }

    #[test]
    fn test_context_menu_item_disabled() {
        let item = ContextMenuItem::new(ContextMenuAction::Copy, "复制").enabled(false);
        assert!(!item.is_enabled());
    }

    #[test]
    fn test_context_menu_state_default() {
        let state = ContextMenuState::new();
        assert!(!state.is_visible());
        assert!(!state.has_selection());
        assert_eq!(state.items().len(), 2);
    }
}

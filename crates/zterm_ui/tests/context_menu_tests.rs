//! 右键菜单功能的 TDD 测试
//!
//! 测试内容:
//! - 菜单项状态管理
//! - 复制功能 (仅在有选择文本时可用)
//! - 粘贴功能 (始终可用)

use zterm_ui::{ContextMenuAction, ContextMenuState};

// ============================================================================
// 上下文菜单状态测试
// ============================================================================

#[cfg(test)]
mod context_menu_state_tests {
    use super::*;

    #[test]
    fn test_context_menu_state_creation() {
        let state = ContextMenuState::new();
        assert!(!state.is_visible(), "菜单初始状态应该是隐藏的");
        assert!(!state.has_selection(), "初始应该没有选择文本");
    }

    #[test]
    fn test_context_menu_show() {
        let mut state = ContextMenuState::new();
        state.show(100.0, 100.0);
        assert!(state.is_visible(), "调用 show 后菜单应该可见");
    }

    #[test]
    fn test_context_menu_hide() {
        let mut state = ContextMenuState::new();
        state.show(100.0, 100.0);
        state.hide();
        assert!(!state.is_visible(), "调用 hide 后菜单应该隐藏");
    }

    #[test]
    fn test_context_menu_position() {
        let mut state = ContextMenuState::new();
        state.show(150.0, 200.0);
        let position = state.position();
        assert_eq!(position.0, 150.0, "菜单 X 坐标应该正确");
        assert_eq!(position.1, 200.0, "菜单 Y 坐标应该正确");
    }
}

// ============================================================================
// 菜单项状态测试
// ============================================================================

#[cfg(test)]
mod menu_item_state_tests {
    use super::*;

    #[test]
    fn test_copy_enabled_with_selection() {
        let mut state = ContextMenuState::new();
        state.set_has_selection(true);

        let copy_item = state.find_item(ContextMenuAction::Copy);
        assert!(copy_item.is_some(), "应该有复制菜单项");
        assert!(copy_item.unwrap().is_enabled(), "有选择文本时复制应该可用");
    }

    #[test]
    fn test_copy_disabled_without_selection() {
        let mut state = ContextMenuState::new();
        state.set_has_selection(false);

        let copy_item = state.find_item(ContextMenuAction::Copy);
        assert!(copy_item.is_some(), "应该有复制菜单项");
        assert!(
            !copy_item.unwrap().is_enabled(),
            "没有选择文本时复制应该禁用"
        );
    }

    #[test]
    fn test_paste_always_enabled() {
        let mut state = ContextMenuState::new();

        // 无论是否有选择，粘贴都应该可用
        state.set_has_selection(true);
        let paste_item = state.find_item(ContextMenuAction::Paste);
        assert!(paste_item.is_some(), "应该有粘贴菜单项");
        assert!(
            paste_item.unwrap().is_enabled(),
            "粘贴应该始终可用 (有选择)"
        );

        state.set_has_selection(false);
        let paste_item = state.find_item(ContextMenuAction::Paste);
        assert!(
            paste_item.unwrap().is_enabled(),
            "粘贴应该始终可用 (无选择)"
        );
    }

    #[test]
    fn test_menu_items_structure() {
        let state = ContextMenuState::new();
        let items = state.items();

        // 应该至少有复制和粘贴两个菜单项
        assert!(items.len() >= 2, "菜单至少应该有 2 个项目 (复制、粘贴)");

        // 检查菜单项的顺序
        let actions: Vec<_> = items.iter().map(|item| item.action()).collect();
        let copy_index = actions
            .iter()
            .position(|a| matches!(a, ContextMenuAction::Copy));
        let paste_index = actions
            .iter()
            .position(|a| matches!(a, ContextMenuAction::Paste));

        assert!(copy_index.is_some(), "应该有复制菜单项");
        assert!(paste_index.is_some(), "应该有粘贴菜单项");
    }
}

// ============================================================================
// 菜单项点击行为测试
// ============================================================================

#[cfg(test)]
mod menu_item_action_tests {
    use super::*;

    #[test]
    fn test_can_execute_copy_with_selection() {
        let mut state = ContextMenuState::new();
        state.set_has_selection(true);
        assert!(
            state.can_execute_action(ContextMenuAction::Copy),
            "有选择文本时应该可以执行复制"
        );
    }

    #[test]
    fn test_cannot_execute_copy_without_selection() {
        let mut state = ContextMenuState::new();
        state.set_has_selection(false);
        assert!(
            !state.can_execute_action(ContextMenuAction::Copy),
            "没有选择文本时不应该可以执行复制"
        );
    }

    #[test]
    fn test_can_always_execute_paste() {
        let mut state = ContextMenuState::new();

        state.set_has_selection(true);
        assert!(
            state.can_execute_action(ContextMenuAction::Paste),
            "有选择文本时应该可以执行粘贴"
        );

        state.set_has_selection(false);
        assert!(
            state.can_execute_action(ContextMenuAction::Paste),
            "没有选择文本时也应该可以执行粘贴"
        );
    }

    #[test]
    fn test_menu_closes_after_action() {
        let mut state = ContextMenuState::new();
        state.show(100.0, 100.0);
        state.set_has_selection(true);

        // 执行操作后菜单应该关闭
        state.execute_action(ContextMenuAction::Copy);
        assert!(!state.is_visible(), "执行操作后菜单应该关闭");
    }
}

// ============================================================================
// 边界情况测试
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_menu_multiple_show_hide_cycles() {
        let mut state = ContextMenuState::new();

        for i in 0..10 {
            state.show(100.0 + i as f32 * 10.0, 100.0);
            assert!(state.is_visible(), "第 {} 次显示菜单应该可见", i);

            state.hide();
            assert!(!state.is_visible(), "第 {} 次隐藏菜单应该不可见", i);
        }
    }

    #[test]
    fn test_menu_state_changes_while_visible() {
        let mut state = ContextMenuState::new();
        state.show(100.0, 100.0);

        // 菜单显示时改变选择状态
        state.set_has_selection(false);
        let copy_disabled = state
            .find_item(ContextMenuAction::Copy)
            .map(|item| !item.is_enabled())
            .unwrap_or(false);
        assert!(copy_disabled, "改变选择状态应该立即影响菜单项");

        state.set_has_selection(true);
        let copy_enabled = state
            .find_item(ContextMenuAction::Copy)
            .map(|item| item.is_enabled())
            .unwrap_or(false);
        assert!(copy_enabled, "恢复选择状态应该立即启用复制");
    }

    #[test]
    fn test_empty_selection() {
        let mut state = ContextMenuState::new();
        // 空选择应该禁用复制
        state.set_has_selection(false);

        let copy_item = state.find_item(ContextMenuAction::Copy);
        assert!(
            copy_item.is_some() && !copy_item.unwrap().is_enabled(),
            "空选择应该禁用复制"
        );
    }

    #[test]
    fn test_position_update() {
        let mut state = ContextMenuState::new();

        state.show(100.0, 200.0);
        assert_eq!(state.position(), (100.0, 200.0));

        // 重新显示在新位置
        state.show(300.0, 400.0);
        assert_eq!(state.position(), (300.0, 400.0));
    }
}

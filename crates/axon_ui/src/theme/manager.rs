//! 全局主题管理器

use super::{Theme, ThemeRegistry};
use gpui::{App, BorrowAppContext, Global};
use std::sync::Arc;
use tracing::{info, warn};

/// 全局主题管理器
///
/// 管理当前活动主题和主题注册表
pub struct ThemeManager {
    /// 主题注册表
    registry: ThemeRegistry,
    /// 当前活动主题
    current_theme: Arc<Theme>,
}

impl Global for ThemeManager {}

impl ThemeManager {
    /// 创建新的主题管理器
    ///
    /// 使用内置主题注册表初始化，默认主题为 "Default Dark"
    pub fn new() -> Self {
        let registry = super::builtin::create_builtin_registry();
        let current_theme = registry
            .get("Default Dark")
            .expect("Default Dark theme should exist");

        Self {
            registry,
            current_theme,
        }
    }

    /// 初始化全局主题管理器
    ///
    /// 应该在应用启动时调用一次
    pub fn init(cx: &mut App) {
        let manager = Self::new();
        info!(
            "ThemeManager initialized with theme: {}",
            manager.current_theme.name()
        );
        cx.set_global(manager);
    }

    /// 获取当前主题
    pub fn current_theme(cx: &App) -> Arc<Theme> {
        cx.global::<Self>().current_theme.clone()
    }

    /// 设置当前主题（通过名称）
    ///
    /// 如果主题不存在，返回 false 并保持当前主题不变
    pub fn set_theme_by_name(theme_name: &str, cx: &mut App) -> bool {
        let theme = match cx.global::<Self>().registry.get(theme_name) {
            Some(t) => t,
            None => {
                warn!("Theme '{}' not found, keeping current theme", theme_name);
                return false;
            }
        };

        cx.update_global::<Self, _>(|manager, _| {
            manager.current_theme = theme.clone();
            info!("Theme changed to: {}", theme_name);
        });

        // 刷新所有窗口以应用新主题
        cx.refresh_windows();
        true
    }

    /// 获取主题注册表（用于列出所有可用主题）
    pub fn registry(cx: &App) -> &ThemeRegistry {
        &cx.global::<Self>().registry
    }

    /// 注册新主题
    pub fn register_theme(theme: Theme, cx: &mut App) {
        cx.update_global::<Self, _>(|manager, _| {
            manager.registry.register(theme);
        });
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Appearance;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert_eq!(manager.current_theme.name(), "Default Dark");
        assert_eq!(manager.registry.all().len(), 5); // Default Dark, GitHub Dark, GitHub Light, Tokyo Night, Tokyo Night Light
    }

    #[test]
    fn test_theme_manager_default() {
        let manager = ThemeManager::default();
        assert_eq!(manager.current_theme.name(), "Default Dark");
    }

    #[test]
    fn test_registry_contains_builtin_themes() {
        let manager = ThemeManager::new();
        assert!(manager.registry.get("Default Dark").is_some());
        assert!(manager.registry.get("GitHub Dark").is_some());
        assert!(manager.registry.get("GitHub Light").is_some());
        assert!(manager.registry.get("Tokyo Night").is_some());
        assert!(manager.registry.get("Tokyo Night Light").is_some());
    }

    #[test]
    fn test_registry_filters_by_appearance() {
        let manager = ThemeManager::new();
        let dark_themes = manager.registry.by_appearance(Appearance::Dark);
        let light_themes = manager.registry.by_appearance(Appearance::Light);

        assert_eq!(dark_themes.len(), 3); // Default Dark, GitHub Dark, Tokyo Night
        assert_eq!(light_themes.len(), 2); // GitHub Light, Tokyo Night Light
    }
}

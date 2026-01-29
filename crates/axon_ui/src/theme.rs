//! 主题系统
//!
//! 提供主题颜色定义和管理功能

pub mod builtin;
pub mod context;
pub mod loader;
pub mod manager;
pub mod theme_serde;

use gpui::{Hsla, SharedString};
use std::sync::Arc;

/// 主题外观模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Appearance {
    /// 浅色模式
    Light,
    /// 深色模式
    Dark,
}

impl Appearance {
    /// 判断是否为浅色模式
    pub fn is_light(&self) -> bool {
        matches!(self, Self::Light)
    }

    /// 判断是否为深色模式
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }
}

/// 主题颜色
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    /// 主背景色
    pub background: Hsla,
    /// 表面背景色 (用于面板、卡片等)
    pub surface_background: Hsla,
    /// 边框颜色
    pub border: Hsla,
    /// 边框变体颜色 (用于分隔线)
    pub border_variant: Hsla,
    /// 主文本颜色
    pub text: Hsla,
    /// 次要文本颜色
    pub text_muted: Hsla,
    /// 占位符文本颜色
    pub text_placeholder: Hsla,

    // 图标颜色
    /// 默认图标颜色
    pub icon: Hsla,
    /// 次要图标颜色
    pub icon_muted: Hsla,

    // 语义化颜色
    /// 危险操作主色 (用于删除、关闭等破坏性操作)
    pub danger: Hsla,
    /// 危险操作前景色 (通常为白色)
    pub danger_foreground: Hsla,

    // UI 组件颜色
    /// Title bar 背景色
    pub titlebar_background: Hsla,
    /// Tab bar 背景色
    pub tab_bar_background: Hsla,
    /// 激活的 tab 背景色
    pub tab_active_background: Hsla,
    /// 非激活的 tab 背景色
    pub tab_inactive_background: Hsla,
    /// Tab hover 背景色
    pub tab_hover_background: Hsla,
    /// 激活的 tab 指示器颜色
    pub tab_active_indicator: Hsla,
    /// 按钮 hover 背景色
    pub button_hover_background: Hsla,
    /// 按钮 active 背景色
    pub button_active_background: Hsla,
    /// Status bar 背景色
    pub statusbar_background: Hsla,

    // 菜单颜色
    /// 菜单背景色
    pub menu_background: Hsla,
    /// 菜单边框色
    pub menu_border: Hsla,
    /// 菜单项悬停背景色
    pub menu_item_hover_background: Hsla,
    /// 菜单项悬停文本色
    pub menu_item_hover_text: Hsla,
    /// 禁用菜单项文本色
    pub menu_item_disabled_text: Hsla,

    // 滚动条颜色 (参考 Zed 主题系统)
    /// 滚动条滑块背景色
    pub scrollbar_thumb_background: Hsla,
    /// 滚动条滑块悬停背景色
    pub scrollbar_thumb_hover_background: Hsla,
    /// 滚动条滑块激活(拖拽中)背景色
    pub scrollbar_thumb_active_background: Hsla,
    /// 滚动条滑块边框色
    pub scrollbar_thumb_border: Hsla,
    /// 滚动条轨道背景色
    pub scrollbar_track_background: Hsla,
    /// 滚动条轨道边框色
    pub scrollbar_track_border: Hsla,
}

/// 主题
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// 主题名称
    pub name: SharedString,
    /// 主题外观模式
    pub appearance: Appearance,
    /// 主题颜色
    pub colors: ThemeColors,
}

impl Theme {
    /// 创建新主题
    pub fn new(name: impl Into<SharedString>, appearance: Appearance, colors: ThemeColors) -> Self {
        Self {
            name: name.into(),
            appearance,
            colors,
        }
    }

    /// 获取主题名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取主题外观模式
    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// 获取主题颜色
    pub fn colors(&self) -> &ThemeColors {
        &self.colors
    }
}

/// 主题注册表
#[derive(Debug, Clone, Default)]
pub struct ThemeRegistry {
    themes: Vec<Arc<Theme>>,
}

impl ThemeRegistry {
    /// 创建新的主题注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册主题
    pub fn register(&mut self, theme: Theme) {
        self.themes.push(Arc::new(theme));
    }

    /// 根据名称获取主题（大小写不敏感）
    pub fn get(&self, name: &str) -> Option<Arc<Theme>> {
        self.themes
            .iter()
            .find(|t| t.name().eq_ignore_ascii_case(name))
            .cloned()
    }

    /// 获取所有主题
    pub fn all(&self) -> &[Arc<Theme>] {
        &self.themes
    }

    /// 根据外观模式获取主题列表
    pub fn by_appearance(&self, appearance: Appearance) -> Vec<Arc<Theme>> {
        self.themes
            .iter()
            .filter(|t| t.appearance() == appearance)
            .cloned()
            .collect()
    }
}

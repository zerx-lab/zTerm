//! 右键上下文菜单视图组件
//!
//! 使用 GPUI 渲染可交互的上下文菜单

use crate::shell_integration::ContextMenuAction;
use axon_ui::ThemeContext;
use gpui::*;

// 菜单导航 actions
actions!(
    context_menu,
    [SelectPrevious, SelectNext, Confirm, Cancel]
);

/// 菜单项渲染数据
#[derive(Clone)]
pub struct MenuItemData {
    pub label: SharedString,
    pub action: ContextMenuAction,
    pub enabled: bool,
    pub shortcut: Option<SharedString>,
}

/// 上下文菜单视图
pub struct ContextMenuView {
    /// 焦点句柄
    focus_handle: FocusHandle,
    /// 菜单项列表
    items: Vec<MenuItemData>,
    /// 当前选中的菜单项索引
    selected_index: Option<usize>,
    /// 菜单项点击回调
    on_action: Option<Box<dyn Fn(ContextMenuAction, &mut Window, &mut Context<Self>)>>,
}

impl ContextMenuView {
    /// 创建新的上下文菜单视图
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            items: Vec::new(),
            selected_index: None,
            on_action: None,
        }
    }

    /// 添加菜单项
    pub fn item(
        mut self,
        label: impl Into<SharedString>,
        action: ContextMenuAction,
        enabled: bool,
    ) -> Self {
        self.items.push(MenuItemData {
            label: label.into(),
            action,
            enabled,
            shortcut: None,
        });
        self
    }

    /// 添加带快捷键提示的菜单项
    pub fn item_with_shortcut(
        mut self,
        label: impl Into<SharedString>,
        action: ContextMenuAction,
        enabled: bool,
        shortcut: impl Into<SharedString>,
    ) -> Self {
        self.items.push(MenuItemData {
            label: label.into(),
            action,
            enabled,
            shortcut: Some(shortcut.into()),
        });
        self
    }

    /// 设置菜单项点击回调
    pub fn on_action(
        mut self,
        handler: impl Fn(ContextMenuAction, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_action = Some(Box::new(handler));
        self
    }

    /// 选择上一个菜单项
    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }

        self.selected_index = Some(match self.selected_index {
            None => self.items.len() - 1,
            Some(0) => self.items.len() - 1,
            Some(i) => i - 1,
        });
        cx.notify();
    }

    /// 选择下一个菜单项
    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }

        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(i) if i >= self.items.len() - 1 => 0,
            Some(i) => i + 1,
        });
        cx.notify();
    }

    /// 确认当前选中的菜单项
    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.selected_index {
            if let Some(item) = self.items.get(index) {
                if item.enabled {
                    let action = item.action.clone();
                    if let Some(handler) = &self.on_action {
                        handler(action, window, cx);
                    }
                    cx.emit(DismissEvent);
                }
            }
        }
    }

    /// 取消菜单
    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    /// 渲染单个菜单项
    fn render_menu_item(
        &self,
        index: usize,
        item: &MenuItemData,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected_index == Some(index);
        let action = item.action.clone();
        let enabled = item.enabled;

        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;

        let mut base = div()
            .id(("menu-item", index))
            .flex()
            .items_center()
            .justify_between()
            .px_2()
            .py_1()
            .cursor(if enabled {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            });

        // 根据状态应用样式
        if is_selected && enabled {
            base = base
                .bg(colors.menu_item_hover_background)
                .text_color(colors.menu_item_hover_text);
        } else if !enabled {
            base = base.text_color(colors.menu_item_disabled_text);
        }

        base = base.on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
            if !enabled {
                return;
            }
            if let Some(handler) = &this.on_action {
                handler(action.clone(), window, cx);
            }
            cx.emit(DismissEvent);
        }));

        base = base.on_hover(cx.listener(move |this, hovered, _, cx| {
            if *hovered && enabled {
                this.selected_index = Some(index);
                cx.notify();
            }
        }));

        let mut content = div().flex().gap_2().child(div().child(item.label.clone()));

        if let Some(ref shortcut) = item.shortcut {
            content = content.child(
                div()
                    .text_color(if is_selected && enabled {
                        colors.menu_item_hover_text
                    } else {
                        colors.text_muted
                    })
                    .text_size(px(12.))
                    .child(shortcut.clone()),
            );
        }

        base.child(content)
    }
}

impl EventEmitter<DismissEvent> for ContextMenuView {}

impl Focusable for ContextMenuView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ContextMenuView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 获取主题颜色
        let theme = cx.current_theme();
        let colors = &theme.colors;

        // 先设置所有 action listeners，避免借用冲突
        let container = div()
            .id("context-menu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            // 添加外部点击监听器，点击菜单外部时关闭菜单
            .on_mouse_down_out(cx.listener(|_this, _event, _window, cx| {
                cx.emit(DismissEvent);
            }))
            .bg(colors.menu_background)
            .border_1()
            .border_color(colors.menu_border)
            .rounded_md()
            .shadow_lg()
            .min_w(px(200.))
            .max_w(px(300.))
            .py_1();

        // 现在渲染菜单项
        let mut list = div().flex().flex_col();
        for (i, item) in self.items.iter().enumerate() {
            list = list.child(self.render_menu_item(i, item, window, cx));
        }

        container.child(list)
    }
}

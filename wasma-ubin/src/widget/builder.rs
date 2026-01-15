// src/widget/builder.rs
// UBIN Fluent Widget Builder – Zincirleme, okunaklı API
// Geliştirici kolayca karmaşık UI'lar oluşturabilsin
// Tüm widget'lar builder ile tanımlanır – WASMA ruhuna yakışır şekilde

use super::primitives::{UbinAction, UbinLayoutDirection};
use crate::UbinWidget;
use super::advanced::{UbinAdvancedWidget, TabItem, MenuItem, DialogButton};

/// UBIN Builder – Ana entry point
pub struct UbinBuilder;

impl UbinBuilder {
    /// Yeni pencere başlat
    pub fn window(title: impl Into<String>) -> WindowBuilder {
        WindowBuilder {
            title: title.into(),
            width: 1280,
            height: 720,
            child: None,
        }
    }

    /// Column layout başlat
    pub fn column() -> LayoutBuilder {
        LayoutBuilder {
            direction: UbinLayoutDirection::Vertical,
            spacing: 10,
            children: vec![],
        }
    }

    /// Row layout başlat
    pub fn row() -> LayoutBuilder {
        LayoutBuilder {
            direction: UbinLayoutDirection::Horizontal,
            spacing: 10,
            children: vec![],
        }
    }

    /// Button hızlı oluştur
    pub fn button(label: impl Into<String>, action: UbinAction) -> UbinWidget {
        UbinWidget::Button {
            label: label.into(),
            action,
            enabled: true,
        }
    }

    /// Label hızlı oluştur
    pub fn label(text: impl Into<String>) -> UbinWidget {
        UbinWidget::Label { text: text.into() }
    }
}

/// Window Builder
pub struct WindowBuilder {
    title: String,
    width: u32,
    height: u32,
    child: Option<UbinWidget>,
}

impl WindowBuilder {
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn child(mut self, child: UbinWidget) -> Self {
        self.child = Some(child);
        self
    }

    pub fn build(self) -> UbinWidget {
        UbinWidget::Window {
            title: self.title,
            width: self.width,
            height: self.height,
            child: Box::new(self.child.unwrap_or(UbinWidget::Label { text: "Empty Window".to_string() })),
        }
    }
}

/// Layout Builder – Row / Column
pub struct LayoutBuilder {
    direction: UbinLayoutDirection,
    spacing: u32,
    children: Vec<UbinWidget>,
}

impl LayoutBuilder {
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn push(mut self, child: UbinWidget) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<UbinWidget>) -> Self {
        self.children = children;
        self
    }

    pub fn build(self) -> UbinWidget {
        UbinWidget::Layout {
            direction: self.direction,
            spacing: self.spacing,
            children: self.children,
        }
    }
}

/// Advanced Widget Builder'ları – Zincirleme kullanım için
impl UbinAdvancedWidget {
    /// TabView builder
    pub fn tab_view() -> TabViewBuilder {
        TabViewBuilder { tabs: vec![], active_tab: 0 }
    }

    /// MenuBar builder
    pub fn menu_bar() -> MenuBarBuilder {
        MenuBarBuilder { items: vec![] }
    }

    /// Dialog builder
    pub fn dialog(title: impl Into<String>) -> DialogBuilder {
        DialogBuilder {
            title: title.into(),
            content: None,
            buttons: vec![],
            on_close: UbinAction::CloseWindow,
        }
    }

    /// Card builder
    pub fn card() -> CardBuilder {
        CardBuilder {
            title: None,
            content: None,
            elevation: 4.0,
            rounded: true,
        }
    }

    /// Dropdown builder
    pub fn dropdown(placeholder: impl Into<String>) -> DropdownBuilder {
        DropdownBuilder {
            placeholder: placeholder.into(),
            items: vec![],
            selected: 0,
            on_select: UbinAction::NoOp,
        }
    }
}

/// TabView Builder
pub struct TabViewBuilder {
    tabs: Vec<TabItem>,
    active_tab: usize,
}

impl TabViewBuilder {
    pub fn tab(mut self, title: impl Into<String>, content: UbinWidget) -> Self {
        self.tabs.push(TabItem {
            title: title.into(),
            content,
            icon: None,
        });
        self
    }

    pub fn active(mut self, index: usize) -> Self {
        self.active_tab = index;
        self
    }

    pub fn build(self) -> UbinAdvancedWidget {
        UbinAdvancedWidget::TabView {
            tabs: self.tabs,
            active_tab: self.active_tab,
        }
    }
}

/// MenuBar Builder
pub struct MenuBarBuilder {
    items: Vec<MenuItem>,
}

impl MenuBarBuilder {
    pub fn item(mut self, label: impl Into<String>, action: UbinAction) -> Self {
        self.items.push(MenuItem {
            label: label.into(),
            submenu: None,
            action: Some(action),
            shortcut: None,
        });
        self
    }

    pub fn submenu(mut self, label: impl Into<String>, submenu: Vec<MenuItem>) -> Self {
        self.items.push(MenuItem {
            label: label.into(),
            submenu: Some(submenu),
            action: None,
            shortcut: None,
        });
        self
    }

    pub fn build(self) -> UbinAdvancedWidget {
        UbinAdvancedWidget::MenuBar { items: self.items }
    }
}

/// Dialog Builder
pub struct DialogBuilder {
    title: String,
    content: Option<UbinWidget>,
    buttons: Vec<DialogButton>,
    on_close: UbinAction,
}

impl DialogBuilder {
    pub fn content(mut self, content: UbinWidget) -> Self {
        self.content = Some(content);
        self
    }

    pub fn button(mut self, label: impl Into<String>, action: UbinAction, primary: bool) -> Self {
        self.buttons.push(DialogButton {
            label: label.into(),
            action,
            primary,
        });
        self
    }

    pub fn build(self) -> UbinAdvancedWidget {
        UbinAdvancedWidget::Dialog {
            title: self.title,
            content: Box::new(self.content.unwrap_or(UbinWidget::Label { text: "No content".to_string() })),
            buttons: self.buttons,
            on_close: self.on_close,
        }
    }
}

/// Card Builder
pub struct CardBuilder {
    title: Option<String>,
    content: Option<UbinWidget>,
    elevation: f32,
    rounded: bool,
}

impl CardBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: UbinWidget) -> Self {
        self.content = Some(content);
        self
    }

    pub fn elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn build(self) -> UbinAdvancedWidget {
        UbinAdvancedWidget::Card {
            title: self.title,
            content: Box::new(self.content.unwrap_or(UbinWidget::Label { text: "Empty card".to_string() })),
            elevation: self.elevation,
            rounded: self.rounded,
        }
    }
}

/// Dropdown Builder
pub struct DropdownBuilder {
    placeholder: String,
    items: Vec<String>,
    selected: usize,
    on_select: UbinAction,
}

impl DropdownBuilder {
    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn on_select(mut self, action: UbinAction) -> Self {
        self.on_select = action;
        self
    }

    pub fn build(self) -> UbinAdvancedWidget {
        UbinAdvancedWidget::Dropdown {
            placeholder: self.placeholder,
            items: self.items,
            selected: self.selected,
            on_select: self.on_select,
        }
    }
}

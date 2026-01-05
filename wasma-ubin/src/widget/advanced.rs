// src/widget/advanced.rs
// UBIN Advanced Portable Widget Kütüphanesi
// İleri seviye widget'lar – ScrollView, TabView, MenuBar, Dialog, Card vs.
// Tek UBIN ABI – tüm platformlarda native gibi render edilir
// Eksik özellik kalmayacak – UBIN polyfill ile tamamlar

use super::primitives::{UbinWidget, UbinAction, UbinLayoutDirection};
use std::fmt;

/// İleri seviye widget – daha karmaşık UI elemanları
#[derive(Debug, Clone)]
pub enum UbinAdvancedWidget {
    /// Kaydırılabilir alan – scrollable content
    ScrollView {
        child: Box<UbinWidget>,
        horizontal: bool,
        vertical: bool,
        scroll_position: (f32, f32),  // x, y offset
    },

    /// TabView – sekme yapısı
    TabView {
        tabs: Vec<TabItem>,
        active_tab: usize,
    },

    /// Menü çubuğu – üst menü
    MenuBar {
        items: Vec<MenuItem>,
    },

    /// Modal diyalog – sheet veya popup
    Dialog {
        title: String,
        content: Box<UbinWidget>,
        buttons: Vec<DialogButton>,
        on_close: UbinAction,
    },

    /// Kart – shadowed container
    Card {
        title: Option<String>,
        content: Box<UbinWidget>,
        elevation: f32,
        rounded: bool,
    },

    /// İlerleme halkası (circular progress)
    ProgressRing {
        progress: f32,  // 0.0 - 1.0
        size: f32,
        label: Option<String>,
    },

    /// Tooltip – hover'da çıkan bilgi
    Tooltip {
        child: Box<UbinWidget>,
        text: String,
        position: TooltipPosition,
    },

    /// Dropdown / ComboBox
    Dropdown {
        placeholder: String,
        items: Vec<String>,
        selected: usize,
        on_select: UbinAction,
    },

    /// Liste görünümü – scrollable item list
    ListView {
        items: Vec<ListItem>,
        selectable: bool,
        on_select: UbinAction,
    },
}

/// TabView için tab item
#[derive(Debug, Clone)]
pub struct TabItem {
    pub title: String,
    pub content: UbinWidget,
    pub icon: Option<String>,  // emoji veya icon name
}

/// Menü item
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub submenu: Option<Vec<MenuItem>>,
    pub action: Option<UbinAction>,
    pub shortcut: Option<String>,
}

/// Dialog buton
#[derive(Debug, Clone)]
pub struct DialogButton {
    pub label: String,
    pub action: UbinAction,
    pub primary: bool,
}

/// Tooltip pozisyonu
#[derive(Debug, Clone, Copy)]
pub enum TooltipPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Liste item
#[derive(Debug, Clone)]
pub struct ListItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub selected: bool,
}

// Fluent builder örnekleri – geliştirici kolay kullansın
impl UbinAdvancedWidget {
    pub fn scroll_view(child: UbinWidget) -> Self {
        UbinAdvancedWidget::ScrollView {
            child: Box::new(child),
            horizontal: true,
            vertical: true,
            scroll_position: (0.0, 0.0),
        }
    }

    pub fn progress_ring(progress: f32) -> Self {
        UbinAdvancedWidget::ProgressRing {
            progress,
            size: 48.0,
            label: None,
        }
    }

    pub fn tooltip(child: UbinWidget, text: impl Into<String>) -> Self {
        UbinAdvancedWidget::Tooltip {
            child: Box::new(child),
            text: text.into(),
            position: TooltipPosition::Top,
        }
    }

    pub fn list_view(items: Vec<ListItem>) -> Self {
        UbinAdvancedWidget::ListView {
            items,
            selectable: true,
            on_select: UbinAction::NoOp,
        }
    }
}

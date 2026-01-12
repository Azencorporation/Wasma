// src/core/abi.rs
// UBIN Portable Unified Widget ABI – Tek otorite, tek dil
// Tüm platformlar bu ABI'yi konuşacak
// Eksik özellik kalmayacak – UBIN tamamlayacak

/// UBIN Action – Widget'ların tetikleyeceği olaylar
#[derive(Debug, Clone)]
pub enum UbinAction {
    NoOp,
    CloseWindow,
    RenewLease(u32),
    CustomCallback(u64),
    ToggleDarkMode,
    OpenUrl(String),
}

/// UBIN Layout Yönü
#[derive(Debug, Clone, Copy)]
pub enum UbinLayoutDirection {
    Horizontal,
    Vertical,
    Grid(u32, u32),
}

/// UBIN Portable Widget – Tek ABI, tüm platformlarda aynı
#[derive(Debug, Clone)]
pub enum UbinWidget {
    /// Ana pencere
    Window {
        title: String,
        width: u32,
        height: u32,
        child: Box<UbinWidget>,
    },
    /// Temel buton
    Button {
        label: String,
        action: UbinAction,
        enabled: bool,
    },
    /// Metin etiketi
    Label {
        text: String,
    },
    /// Metin giriş alanı
    TextInput {
        placeholder: String,
        value: String,
        on_change: UbinAction,
    },
    /// Checkbox
    Checkbox {
        label: String,
        checked: bool,
        on_toggle: UbinAction,
    },
    /// Slider
    Slider {
        min: f32,
        max: f32,
        value: f32,
        step: f32,
        on_change: UbinAction,
    },
    /// Kaydırılabilir alan
    ScrollView {
        child: Box<UbinWidget>,
    },
    /// Yatay/dikey layout
    Layout {
        direction: UbinLayoutDirection,
        spacing: u32,
        children: Vec<UbinWidget>,
    },
    /// Boş alan (spacer)
    Spacer {
        size: u32,
    },
    /// İlerleme çubuğu
    ProgressBar {
        progress: f32,  // 0.0 - 1.0
        label: Option<String>,
    },
    /// DÜZELTME: Divider eklendi
    Divider {
        vertical: bool,
        thickness: u32,
    },
}

impl UbinWidget {
    /// Fluent builder örnekleri
    pub fn window(title: impl Into<String>, width: u32, height: u32, child: UbinWidget) -> Self {
        UbinWidget::Window {
            title: title.into(),
            width,
            height,
            child: Box::new(child),
        }
    }

    pub fn button(label: impl Into<String>, action: UbinAction) -> Self {
        UbinWidget::Button {
            label: label.into(),
            action,
            enabled: true,
        }
    }

    pub fn label(text: impl Into<String>) -> Self {
        UbinWidget::Label { text: text.into() }
    }

    pub fn column(children: Vec<UbinWidget>) -> Self {
        UbinWidget::Layout {
            direction: UbinLayoutDirection::Vertical,
            spacing: 10,
            children,
        }
    }

    pub fn row(children: Vec<UbinWidget>) -> Self {
        UbinWidget::Layout {
            direction: UbinLayoutDirection::Horizontal,
            spacing: 10,
            children,
        }
    }

    pub fn divider(vertical: bool, thickness: u32) -> Self {
        UbinWidget::Divider { vertical, thickness }
    }
}

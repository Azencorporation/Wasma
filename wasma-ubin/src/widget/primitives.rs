// src/widget/primitives.rs
// UBIN Primitive Portable Widget Kütüphanesi
// Temel widget'lar – Button, Label, TextInput, Checkbox, Slider vs.
// Tek UBIN ABI – tüm platformlarda native gibi render edilir
// Eksik özellik kalmayacak – UBIN polyfill ile tamamlar
// src/widget/mod.rs-də
pub use crate::core::abi::{UbinAction, UbinLayoutDirection};
use std::fmt;

/// Temel UBIN widget'ları – primitive set
#[derive(Debug, Clone)]
pub enum UbinPrimitiveWidget {
    /// Metin etiketi
    Label {
        text: String,
        size: f32,           // font size
        color: Option<(f32, f32, f32)>,  // RGB – platform native fallback
    },

    /// Buton
    Button {
        label: String,
        action: UbinAction,
        enabled: bool,
        primary: bool,       // accent color
    },

    /// Metin giriş alanı
    TextInput {
        placeholder: String,
        value: String,
        on_change: UbinAction,
        password: bool,
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

    /// İlerleme çubuğu
    ProgressBar {
        progress: f32,       // 0.0 - 1.0
        label: Option<String>,
        indeterminate: bool,
    },

    /// Boş alan – spacer
    Spacer {
        size: u32,           // pixel
        flexible: bool,      // fill available space
    },

    /// Divider – ayırıcı çizgi
    Divider {
        vertical: bool,
        thickness: f32,
    },

    /// İkon
    Icon {
        name: String,        // emoji veya icon identifier
        size: f32,
    },

    /// Image
    Image {
        source: String,      // path veya data URL
        width: Option<u32>,
        height: Option<u32>,
        fit: ImageFit,
    },
}

/// Image fit modu
#[derive(Debug, Clone, Copy)]
pub enum ImageFit {
    Contain,
    Cover,
    Fill,
    ScaleDown,
}

// Fluent builder örnekleri – geliştirici hızlı kullansın
impl UbinPrimitiveWidget {
    pub fn label(text: impl Into<String>) -> Self {
        UbinPrimitiveWidget::Label {
            text: text.into(),
            size: 20.0,
            color: None,
        }
    }

    pub fn button(label: impl Into<String>, action: UbinAction) -> Self {
        UbinPrimitiveWidget::Button {
            label: label.into(),
            action,
            enabled: true,
            primary: false,
        }
    }

    pub fn primary_button(label: impl Into<String>, action: UbinAction) -> Self {
        UbinPrimitiveWidget::Button {
            label: label.into(),
            action,
            enabled: true,
            primary: true,
        }
    }

    pub fn text_input(placeholder: impl Into<String>) -> Self {
        UbinPrimitiveWidget::TextInput {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: UbinAction::NoOp,
            password: false,
        }
    }

    pub fn password_input(placeholder: impl Into<String>) -> Self {
        UbinPrimitiveWidget::TextInput {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: UbinAction::NoOp,
            password: true,
        }
    }

    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        UbinPrimitiveWidget::Checkbox {
            label: label.into(),
            checked,
            on_toggle: UbinAction::NoOp,
        }
    }

    pub fn slider(min: f32, max: f32, value: f32) -> Self {
        UbinPrimitiveWidget::Slider {
            min,
            max,
            value,
            step: 1.0,
            on_change: UbinAction::NoOp,
        }
    }

    pub fn progress_bar(progress: f32) -> Self {
        UbinPrimitiveWidget::ProgressBar {
            progress,
            label: None,
            indeterminate: false,
        }
    }

    pub fn indeterminate_progress() -> Self {
        UbinPrimitiveWidget::ProgressBar {
            progress: 0.0,
            label: None,
            indeterminate: true,
        }
    }

    pub fn spacer(size: u32) -> Self {
        UbinPrimitiveWidget::Spacer {
            size,
            flexible: false,
        }
    }

    pub fn flexible_spacer() -> Self {
        UbinPrimitiveWidget::Spacer {
            size: 0,
            flexible: true,
        }
    }

    pub fn divider() -> Self {
        UbinPrimitiveWidget::Divider {
            vertical: false,
            thickness: 1.0,
        }
    }

    pub fn vertical_divider() -> Self {
        UbinPrimitiveWidget::Divider {
            vertical: true,
            thickness: 1.0,
        }
    }

    pub fn icon(name: impl Into<String>) -> Self {
        UbinPrimitiveWidget::Icon {
            name: name.into(),
            size: 24.0,
        }
    }
}

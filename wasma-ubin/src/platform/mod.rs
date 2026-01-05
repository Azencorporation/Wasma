// src/platform/mod.rs
// UBIN Platform Adaptörleri – Tek giriş noktası
// Runtime burada hangi platformda olduğumuza bakar ve doğru adaptörü seçer

pub mod linux;
pub mod windows;
pub mod macos;
pub mod fallback;

use crate::core::runtime::UbinRuntimeWindow;

/// Platform tipi – runtime'da otomatik tespit edilir
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UbinPlatform {
    Linux,
    Windows,
    MacOS,
    Unknown,
}

/// Aktif platformu tespit et
pub fn detect_current_platform() -> UbinPlatform {
    if cfg!(target_os = "linux") {
        UbinPlatform::Linux
    } else if cfg!(target_os = "windows") {
        UbinPlatform::Windows
    } else if cfg!(target_os = "macos") {
        UbinPlatform::MacOS
    } else {
        UbinPlatform::Unknown
    }
}

/// Runtime window'ı aktif platforma uyarla
pub fn adapt_window_to_platform(window: &mut UbinRuntimeWindow) {
    let platform = detect_current_platform();
    println!("🔄 UBIN adapting window '{}' to platform {:?}", window.title, platform);

    match platform {
        UbinPlatform::Linux => linux::UbinLinuxAdaptor::adapt_runtime_window(window),
        UbinPlatform::Windows => windows::UbinWindowsAdaptor::adapt_runtime_window(window),
        UbinPlatform::MacOS => macos::UbinMacOSAdaptor::adapt_runtime_window(window),
        UbinPlatform::Unknown => {
            println!("⚠️ Unknown platform – falling back to ghost mode");
            fallback::adapt_to_fallback(window);
        }
    }
}

/// Tüm platformlardan özellikleri topla – convergence için
pub fn collect_all_platform_features() -> Vec<String> {
    let mut features = vec![];

    features.extend(linux::UbinLinuxAdaptor{}.extract_linux_features());
    features.extend(windows::UbinWindowsAdaptor{}.extract_windows_features());
    features.extend(macos::UbinMacOSAdaptor{}.extract_macos_features());

    println!("🟢 UBIN collected {} cross-platform features for convergence", features.len());
    features
}

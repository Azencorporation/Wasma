// src/lib.rs
// WASMA-UBIN – Public API
// Tek giriş noktası – tüm modüller burada export edilir
// Dış dünya sadece bu API'yi kullanır – iç yapı gizli kalır
// Tarih: 6 Ocak 2026 – Alpha 1.0

pub mod core;
pub mod platform;
pub mod widget;
pub mod transmutation;
pub mod utils;

pub use core::abi::*;
pub use core::runtime::UbinRuntime;
pub use core::convergence::UbinConvergenceEngine;
pub use core::assignment_bridge::UbinAssignmentBridge;

pub use widget::builder::UbinBuilder;
pub use widget::primitives::*;
pub use widget::advanced::*;

pub use utils::logging::*;
pub use utils::safety::*;

// Re-export temel tipler – kullanıcı kolay erişsin
pub use crate::assignment::{Assignment, ExecutionMode};
pub use crate::resource_manager::ResourceMode;

pub use core::runtime::UbinRuntimeWindow;
pub mod assignment;
pub mod resource_manager;
// Ana UBIN başlatma fonksiyonu – kullanıcı ilk bunu çağırır
/// UBIN sistemini başlatır – safety, logging, runtime hazır hale gelir
pub fn initialize_ubin() {
    // Safety bastion
    UbinSafetyGuard::establish_safety_bastion();

    // Logger – info seviyesi, renkli
    UbinLogger::init(LogLevel::Info, true);

    info("🌀 WASMA-UBIN crate initialized – Public API ready");
    info("🏴‍☠️ Use UbinRuntime::initialize() to start the eternal dominion");
}

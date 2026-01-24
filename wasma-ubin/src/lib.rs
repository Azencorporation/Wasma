// src/lib.rs
// WASMA-UBIN – Public API

pub mod core;
pub mod platform;
pub mod widget;
#[cfg(feature = "transmutation")]
pub mod transmutation;
pub mod utils;

// DÜZELTME: wbackend'den import et
pub use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};

// Re-export core types
pub use core::abi::*;
pub use core::runtime::UbinRuntime;
pub use core::convergence::UbinConvergenceEngine;
pub use core::runtime::UbinRuntimeWindow;

pub use widget::builder::UbinBuilder;
pub use widget::primitives::*;
pub use widget::advanced::*;
pub use utils::logging::*;
pub use utils::safety::*;

/// UBIN sistemini başlatır
pub fn initialize_ubin() {
    UbinSafetyGuard::establish_safety_bastion();
    UbinLogger::init(LogLevel::Info, true);
    
    info("🌀 WASMA-UBIN crate initialized – Public API ready");
    info("🏴‍☠️ Use UbinRuntime::initialize() to start the eternal dominion");
}

// src/transmutation/mod.rs
// UBIN Transmutation – Binary Dönüştürme Motoru

#[cfg(feature = "transmutation")]
pub mod disassembler;
#[cfg(feature = "transmutation")]
pub mod feature_extractor;
#[cfg(feature = "transmutation")]
pub mod patcher;
#[cfg(feature = "transmutation")]
pub mod rebuilder;

#[cfg(feature = "transmutation")]
pub use disassembler::*;
#[cfg(feature = "transmutation")]
pub use feature_extractor::*;
#[cfg(feature = "transmutation")]
pub use patcher::*;
#[cfg(feature = "transmutation")]
pub use rebuilder::*;
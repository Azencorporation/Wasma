// src/transmutation/mod.rs
// UBIN Transmutation – Binary Dönüştürme Motoru

pub mod disassembler;
pub mod feature_extractor;
pub mod patcher;
pub mod rebuilder;

pub use disassembler::*;
pub use feature_extractor::*;
pub use patcher::*;
pub use rebuilder::*;
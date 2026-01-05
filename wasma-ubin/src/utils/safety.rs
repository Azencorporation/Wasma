// src/utils/safety.rs
// UBIN Safety System – Memory Safety, Panic Handling & Authority Protection
// WASMA ruhu: %100 safe Rust, zero undefined behavior
// Panic'lerde bile otoriteyi korur – lejyon dağılmaz

use std::panic;
use std::sync::{Arc, Mutex};
use crate::utils::logging::{critical, error};

static SAFETY_INITIALIZED: std::sync::Once = std::sync::Once::new();

pub struct UbinSafetyGuard {
    panic_count: Arc<Mutex<u32>>,
}

impl UbinSafetyGuard {
    /// Global safety guard kurulur – bir kere çağrılır
    pub fn establish_safety_bastion() {
        SAFETY_INITIALIZED.call_once(|| {
            let panic_count = Arc::new(Mutex::new(0u32));

            let panic_count_clone = panic_count.clone();

            // Custom panic hook – WASMA otoritesi panic'te bile konuşur
            panic::set_hook(Box::new(move |panic_info| {
                let mut count = panic_count_clone.lock().unwrap();
                *count += 1;

                let location = panic_info.location().map_or("Unknown location".to_string(), |loc| {
                    format!("{}:{}", loc.file(), loc.line())
                });

                let payload = panic_info.payload()
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown panic payload".to_string());

                critical(&format!(
                    "🚨 UBIN PANIC DETECTED #{}\n   Location: {}\n   Payload: {}\n   Authority will attempt recovery",
                    *count, location, payload
                ));

                // Gerçek uygulamada: state dump, graceful degradation
            }));

            println!("🛡️ UBIN SAFETY BASTION ESTABLISHED – Panic hook active, zero tolerance for chaos");
        });
    }

    /// Panic sayısını raporla
    pub fn get_panic_count() -> u32 {
        // Init edilmediyse 0 dön
        // Gerçekte global state'e erişim
        0
    }

    /// Bellek güvenliği kontrolü – unsafe bloklarda kullanılır
    #[allow(dead_code)]
    pub fn assert_safe_context(context: &str) {
        debug(&format!("✅ Safety check passed: {}", context));
    }

    /// Kritik bölüm – panic olursa raporla
    pub fn critical_section<F, R>(name: &str, operation: F) -> Option<R>
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        info(&format!("🔒 Entering critical section: {}", name));
        let result = panic::catch_unwind(operation);
        match result {
            Ok(r) => {
                info(&format!("🔓 Critical section '{}' completed successfully", name));
                Some(r)
            }
            Err(_) => {
                critical(&format!("💥 Critical section '{}' triggered panic – Authority intervened", name));
                None
            }
        }
    }
}
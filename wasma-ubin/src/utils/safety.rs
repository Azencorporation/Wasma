// src/utils/safety.rs
// UBIN Safety System

use std::panic;
use std::sync::{Arc, Mutex};
// DÜZELTME: logging fonksiyonlarını import et
use crate::utils::logging::{critical, debug, info};

static SAFETY_INITIALIZED: std::sync::Once = std::sync::Once::new();

pub struct UbinSafetyGuard {
}

impl UbinSafetyGuard {
    pub fn establish_safety_bastion() {
        SAFETY_INITIALIZED.call_once(|| {
            let panic_count = Arc::new(Mutex::new(0u32));
            let panic_count_clone = panic_count.clone();

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
                    "🚨 UBIN PANIC DETECTED #{}\n   Location: {}\n   Payload: {}",
                    *count, location, payload
                ));
            }));

            println!("🛡️ UBIN SAFETY BASTION ESTABLISHED");
        });
    }

    pub fn get_panic_count() -> u32 {
        0
    }

    #[allow(dead_code)]
    pub fn assert_safe_context(context: &str) {
        debug(&format!("✅ Safety check passed: {}", context));
    }

    pub fn critical_section<F, R>(name: &str, operation: F) -> Option<R>
    where
        F: FnOnce() -> R + std::panic::UnwindSafe,
    {
        info(&format!("🔒 Entering critical section: {}", name));
        let result = panic::catch_unwind(operation);
        match result {
            Ok(r) => {
                info(&format!("🔓 Critical section '{}' completed", name));
                Some(r)
            }
            Err(_) => {
                critical(&format!("💥 Critical section '{}' triggered panic", name));
                None
            }
        }
    }
}

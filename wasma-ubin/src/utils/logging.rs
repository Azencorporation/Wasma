// src/utils/logging.rs
// UBIN Centralized Logging System – WASMA Lejyon Günlüğü
// Tüm modüller (core, platform, transmutation, widget) buradan log yazar
// Zaman damgalı, renkli, seviyeli – debug'dan critical'a
// WASMA ruhuna yakışır şekilde detaylı ama kontrollü

use chrono::Local;
use std::sync::{Mutex, OnceLock};

/// Log seviyesi – WASMA otoritesi gibi hiyerarşik
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum LogLevel {
    Debug,    // Geliştirici detayları
    Info,     // Normal operasyonlar
    Warning,  // Potansiyel sorunlar
    Error,    // Hata ama devam
    Critical, // Sistem durabilir – lejyon alarmı
}

/// Global logger – tek instance
static LOGGER: OnceLock<Mutex<UbinLogger>> = OnceLock::new();

pub struct UbinLogger {
    current_level: LogLevel,
    enable_colors: bool,
}

impl UbinLogger {
    /// Global logger'ı başlat – bir kere çağrılır
    pub fn init(level: LogLevel, colors: bool) {
        let logger = UbinLogger {
            current_level: level,
            enable_colors: colors,
        };
        let _ = LOGGER.set(Mutex::new(logger));
        log!(LogLevel::Info, "🌀 UBIN LOGGER INITIALIZED – Lejyon günlüğü aktif");
    }

    /// Log yaz – tüm modüller bunu kullanır
    pub fn write(level: LogLevel, message: &str) {
        let logger = match LOGGER.get() {
            Some(l) => l.lock().unwrap(),
            None => {
                // Init edilmediyse fallback console
                println!("{} [{}] {}", timestamp(), level_to_str(level), message);
                return;
            }
        };

        if level < logger.current_level {
            return; // düşük seviye loglar filtrelenir
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let level_str = level_to_str_colored(level, logger.enable_colors);
        let prefix = format!("{} {} {}", timestamp, level_str, "UBIN");

        println!("{} | {}", prefix, message);
    }
}

/// Makro – tüm crate'lerde kolay log

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        crate::utils::logging::UbinLogger::write($level, &format!($($arg)*))
    };
}

/// Seviye string'i – renkli/renksiz
fn level_to_str(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Debug => "[DEBUG]",
        LogLevel::Info => "[INFO]",
        LogLevel::Warning => "[WARN]",
        LogLevel::Error => "[ERROR]",
        LogLevel::Critical => "[CRIT]",
    }
}

fn level_to_str_colored(level: LogLevel, colors: bool) -> String {
    if !colors {
        return level_to_str(level).to_string();
    }

    match level {
        LogLevel::Debug => "\x1b[36m[DEBUG]\x1b[0m".to_string(),    // cyan
        LogLevel::Info => "\x1b[32m[INFO]\x1b[0m".to_string(),      // green
        LogLevel::Warning => "\x1b[33m[WARN]\x1b[0m".to_string(),   // yellow
        LogLevel::Error => "\x1b[31m[ERROR]\x1b[0m".to_string(),    // red
        LogLevel::Critical => "\x1b[41;37m[CRIT]\x1b[0m".to_string(), // red background white text
    }
}

/// Zaman damgası yardımcı
fn timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

/// Kolay kullanım fonksiyonları – diğer modüller bunları çağırır
pub fn debug(msg: &str) {
    UbinLogger::write(LogLevel::Debug, msg);
}

pub fn info(msg: &str) {
    UbinLogger::write(LogLevel::Info, msg);
}

pub fn warn(msg: &str) {
    UbinLogger::write(LogLevel::Warning, msg);
}

pub fn error(msg: &str) {
    UbinLogger::write(LogLevel::Error, msg);
}

pub fn critical(msg: &str) {
    UbinLogger::write(LogLevel::Critical, msg);
}

// src/transmutation/feature_extractor.rs
// UBIN Feature Extractor – Binary Analiz & Özellik Çıkarma Motoru
// Binary'leri disassemble eder, UI framework'ünü, rendering çağrılarını, platform özel özelliklerini tespit eder
// Çıkarılan özellikler convergence engine'e gönderilir – eksiklikler tamamlanır

use crate::platform::{UbinPlatform, detect_current_platform};
use std::path::PathBuf;
use std::fs;
use std::collections::{HashSet, HashMap};

#[cfg(target_arch = "x86_64")]
use capstone::prelude::*;
use object::{Object, ObjectSection, ObjectSymbol};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtractedFeature {
    // Framework tespiti
    UsesGtk,
    UsesGtk4,
    UsesQt5,
    UsesQt6,
    UsesWin32Api,
    UsesAppKit,
    UsesIcedWgpu,

    // Rendering özellikleri
    UsesVulkan,
    UsesDirectX,
    UsesMetal,
    UsesOpenGL,
    UsesWgpu,

    // Görsel efektler
    HasBlurEffect,
    HasRoundedCorners,
    HasShadowEffect,
    HasAcrylicMaterial,
    HasMicaMaterial,
    HasVibrancy,
    HasRevealHighlight,
    HasAnimations,

    // Layout & Decoration
    HasCsd,
    HasHeaderBar,
    HasUnifiedToolbar,
    HasCustomTitlebar,
    HasNativeMenuBar,

    // Diğer
    HasDarkModeSupport,
    HasHighDpiScaling,
    HasAccessibility,
    HasTouchSupport,
}

#[derive(Debug)]
pub struct BinaryFeatureReport {
    pub path: PathBuf,
    pub platform: UbinPlatform,
    pub detected_framework: String,
    pub extracted_features: HashSet<ExtractedFeature>,
    pub symbol_count: usize,
    pub string_hints: Vec<String>,
    pub analysis_success: bool,
    pub error_msg: Option<String>,
}

pub struct UbinFeatureExtractor;

impl UbinFeatureExtractor {
    /// Yeni extractor – statik
    pub fn new() -> Self {
        UbinFeatureExtractor
    }

    /// Binary'yi analiz et ve feature report üret
    pub fn extract_features_from_binary(&self, path: PathBuf) -> BinaryFeatureReport {
        let data = match fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                return BinaryFeatureReport {
                    path,
                    platform: detect_current_platform(),
                    detected_framework: "Unknown".to_string(),
                    extracted_features: HashSet::new(),
                    symbol_count: 0,
                    string_hints: vec![],
                    analysis_success: false,
                    error_msg: Some(format!("File read error: {}", e)),
                };
            }
        };

        let platform = detect_current_platform();
        let mut report = BinaryFeatureReport {
            path,
            platform,
            detected_framework: "Analyzing...".to_string(),
            extracted_features: HashSet::new(),
            symbol_count: 0,
            string_hints: vec![],
            analysis_success: true,
            error_msg: None,
        };

        // String bazlı hızlı tespit
        let data_str = String::from_utf8_lossy(&data);
        let mut hints = vec![];

        if data_str.contains("gtk") || data_str.contains("Gtk") || data_str.contains("libgtk") {
            report.extracted_features.insert(ExtractedFeature::UsesGtk);
            if data_str.contains("gtk4") || data_str.contains("Gtk4") {
                report.extracted_features.insert(ExtractedFeature::UsesGtk4);
                report.detected_framework = "GTK4".to_string();
            } else {
                report.extracted_features.insert(ExtractedFeature::UsesGtk);
                report.detected_framework = "GTK3".to_string();
            }
            hints.push("GTK detected".to_string());
        }

        if data_str.contains("Qt5") || data_str.contains("libQt5") {
            report.extracted_features.insert(ExtractedFeature::UsesQt5);
            report.detected_framework = "Qt5".to_string();
            hints.push("Qt5 detected".to_string());
        }
        if data_str.contains("Qt6") || data_str.contains("libQt6") {
            report.extracted_features.insert(ExtractedFeature::UsesQt6);
            report.detected_framework = "Qt6".to_string();
            hints.push("Qt6 detected".to_string());
        }

        if data_str.contains("iced") || data_str.contains("wgpu") {
            report.extracted_features.insert(ExtractedFeature::UsesIcedWgpu);
            report.extracted_features.insert(ExtractedFeature::UsesWgpu);
            report.detected_framework = "iced (Rust)".to_string();
            hints.push("iced + wgpu detected".to_string());
        }

        if data_str.contains("AppKit") || data_str.contains("NSWindow") || data_str.contains("NSView") {
            report.extracted_features.insert(ExtractedFeature::UsesAppKit);
            report.detected_framework = "AppKit (macOS)".to_string();
            hints.push("AppKit detected".to_string());
        }

        if data_str.contains("USER32.dll") || data_str.contains("GDI32.dll") || data_str.contains("CreateWindowEx") {
            report.extracted_features.insert(ExtractedFeature::UsesWin32Api);
            report.detected_framework = "Win32".to_string();
            hints.push("Win32 API detected".to_string());
        }

        // Rendering API tespiti
        if data_str.contains("vulkan") || data_str.contains("vkCreateInstance") {
            report.extracted_features.insert(ExtractedFeature::UsesVulkan);
            hints.push("Vulkan detected".to_string());
        }
        if data_str.contains("Direct3DCreate9") || data_str.contains("DXGI") || data_str.contains("d3d12") {
            report.extracted_features.insert(ExtractedFeature::UsesDirectX);
            hints.push("DirectX detected".to_string());
        }
        if data_str.contains("MTLCreateSystemDefaultDevice") || data_str.contains("Metal") {
            report.extracted_features.insert(ExtractedFeature::UsesMetal);
            hints.push("Metal detected".to_string());
        }

        // Görsel efekt string'leri
        if data_str.contains("blur") || data_str.contains("Acrylic") || data_str.contains("Mica") {
            report.extracted_features.insert(ExtractedFeature::HasBlurEffect);
        }
        if data_str.contains("vibrancy") || data_str.contains("NSVisualEffectView") {
            report.extracted_features.insert(ExtractedFeature::HasVibrancy);
        }
        if data_str.contains("rounded") || data_str.contains("cornerRadius") {
            report.extracted_features.insert(ExtractedFeature::HasRoundedCorners);
        }
        if data_str.contains("shadow") || data_str.contains("elevation") {
            report.extracted_features.insert(ExtractedFeature::HasShadowEffect);
        }

        // Symbol tablosu analizi (object crate ile)
        if let Ok(obj) = object::File::parse(&*data) {
            report.symbol_count = obj.symbols().count();

            for symbol in obj.symbols() {
                if let Ok(name) = symbol.name() {
                    let name_str = name.to_lowercase();
                    if name_str.contains("dark") || name_str.contains("theme") {
                        report.extracted_features.insert(ExtractedFeature::HasDarkModeSupport);
                    }
                    if name_str.contains("highdpi") || name_str.contains("scaling") {
                        report.extracted_features.insert(ExtractedFeature::HasHighDpiScaling);
                    }
                }
            }
        }

        report.string_hints = hints;

        if report.extracted_features.is_empty() {
            report.detected_framework = "Unknown / Minimal".to_string();
        }

        println!("🔍 Feature extraction complete for {:?} – {} features detected", report.path.file_name().unwrap(), report.extracted_features.len());

        report
    }

    /// Toplu analiz – dizindeki tüm binary'leri tara
    pub fn extract_from_directory(&self, dir: &str) -> Vec<BinaryFeatureReport> {
        let mut reports = vec![];

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(true, |ext| ext != "ubin-converged") {
                    let report = self.extract_features_from_binary(path);
                    reports.push(report);
                }
            }
        }

        println!("📂 Directory scan complete – {} binaries analyzed", reports.len());
        reports
    }
}
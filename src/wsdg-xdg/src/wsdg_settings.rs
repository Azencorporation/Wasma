// WSDG Settings - Settings Manager
// Provides environment for WASMA window settings
// Basic GUI settings: font, icon, and other configurations
// Supports manifest_rrt for configuration, requires manifest_gui_elshadow for full manifest support
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use thiserror::Error;

use crate::wsdg_env::WsdgEnv;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Settings file not found: {0}")]
    NotFound(String),
    
    #[error("Invalid settings format: {0}")]
    InvalidFormat(String),
    
    #[error("Failed to save settings: {0}")]
    SaveFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// GUI Theme settings
#[derive(Debug, Clone)]
pub struct ThemeSettings {
    pub name: String,
    pub dark_mode: bool,
    pub accent_color: String,
    pub background_color: String,
    pub foreground_color: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            dark_mode: false,
            accent_color: "#3584e4".to_string(),
            background_color: "#ffffff".to_string(),
            foreground_color: "#000000".to_string(),
        }
    }
}

/// Font settings
#[derive(Debug, Clone)]
pub struct FontSettings {
    pub family: String,
    pub size: u32,
    pub weight: String,
    pub monospace_family: String,
    pub monospace_size: u32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "Sans".to_string(),
            size: 11,
            weight: "normal".to_string(),
            monospace_family: "Monospace".to_string(),
            monospace_size: 10,
        }
    }
}

/// Icon settings
#[derive(Debug, Clone)]
pub struct IconSettings {
    pub theme: String,
    pub size: u32,
    pub use_symbolic: bool,
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            theme: "hicolor".to_string(),
            size: 48,
            use_symbolic: false,
        }
    }
}

/// Window settings
#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub default_width: u32,
    pub default_height: u32,
    pub decorations: bool,
    pub transparency: bool,
    pub opacity: f32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            default_width: 800,
            default_height: 600,
            decorations: true,
            transparency: false,
            opacity: 1.0,
        }
    }
}

/// WSDG Settings - Complete settings configuration
#[derive(Debug, Clone)]
pub struct WsdgSettings {
    pub theme: ThemeSettings,
    pub font: FontSettings,
    pub icon: IconSettings,
    pub window: WindowSettings,
    pub custom: HashMap<String, String>,
}

impl Default for WsdgSettings {
    fn default() -> Self {
        Self {
            theme: ThemeSettings::default(),
            font: FontSettings::default(),
            icon: IconSettings::default(),
            window: WindowSettings::default(),
            custom: HashMap::new(),
        }
    }
}

/// WSDG Settings Manager
pub struct WsdgSettingsManager {
    #[allow(dead_code)]
    env: WsdgEnv,
    settings: WsdgSettings,
    settings_path: PathBuf,
    manifest_rrt_support: bool,
    /// WASMA integration callback
    wasma_sync_callback: Option<Box<dyn Fn(&WsdgSettings) + Send + Sync>>,
}

impl WsdgSettingsManager {
    pub fn new(env: WsdgEnv) -> Self {
        let settings_path = Self::get_settings_path(&env);
        
        Self {
            env,
            settings: WsdgSettings::default(),
            settings_path,
            manifest_rrt_support: false,
            wasma_sync_callback: None,
        }
    }
    
    /// Get settings file path
    fn get_settings_path(env: &WsdgEnv) -> PathBuf {
        if let Ok(config_dir) = env.config_dir() {
            config_dir.join("wsdg/settings.conf")
        } else {
            PathBuf::from("/etc/wsdg/settings.conf")
        }
    }
    
    /// Enable manifest_rrt support
    pub fn enable_manifest_rrt(&mut self) {
        self.manifest_rrt_support = true;
    }
    
    /// Load settings from file
    pub fn load(&mut self) -> Result<(), SettingsError> {
        if !self.settings_path.exists() {
            // Use defaults if file doesn't exist
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.settings_path)?;
        self.parse_settings(&content)?;
        
        Ok(())
    }
    
    /// Parse settings content
    fn parse_settings(&mut self, content: &str) -> Result<(), SettingsError> {
        let mut current_section = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("*//") || line.starts_with("#") {
                continue;
            }
            
            // Section headers
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line.trim_matches(|c| c == '[' || c == ']').to_string();
                continue;
            }
            
            // Parse key-value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.split("*//").next().unwrap_or(value).trim();
                let value = value.trim_matches('"').trim_matches('\'');
                
                self.apply_setting(&current_section, key, value)?;
            }
        }
        
        Ok(())
    }
    
    /// Apply a setting value
    fn apply_setting(&mut self, section: &str, key: &str, value: &str) -> Result<(), SettingsError> {
        match section {
            "theme" => {
                match key {
                    "name" => self.settings.theme.name = value.to_string(),
                    "dark_mode" => self.settings.theme.dark_mode = value == "true" || value == "yes",
                    "accent_color" => self.settings.theme.accent_color = value.to_string(),
                    "background_color" => self.settings.theme.background_color = value.to_string(),
                    "foreground_color" => self.settings.theme.foreground_color = value.to_string(),
                    _ => {}
                }
            }
            "font" => {
                match key {
                    "family" => self.settings.font.family = value.to_string(),
                    "size" => self.settings.font.size = value.parse().unwrap_or(11),
                    "weight" => self.settings.font.weight = value.to_string(),
                    "monospace_family" => self.settings.font.monospace_family = value.to_string(),
                    "monospace_size" => self.settings.font.monospace_size = value.parse().unwrap_or(10),
                    _ => {}
                }
            }
            "icon" => {
                match key {
                    "theme" => self.settings.icon.theme = value.to_string(),
                    "size" => self.settings.icon.size = value.parse().unwrap_or(48),
                    "use_symbolic" => self.settings.icon.use_symbolic = value == "true" || value == "yes",
                    _ => {}
                }
            }
            "window" => {
                match key {
                    "default_width" => self.settings.window.default_width = value.parse().unwrap_or(800),
                    "default_height" => self.settings.window.default_height = value.parse().unwrap_or(600),
                    "decorations" => self.settings.window.decorations = value == "true" || value == "yes",
                    "transparency" => self.settings.window.transparency = value == "true" || value == "yes",
                    "opacity" => self.settings.window.opacity = value.parse().unwrap_or(1.0),
                    _ => {}
                }
            }
            "custom" | "" => {
                self.settings.custom.insert(key.to_string(), value.to_string());
            }
            _ => {
                // Store unknown sections in custom
                self.settings.custom.insert(
                    format!("{}.{}", section, key),
                    value.to_string()
                );
            }
        }
        
        Ok(())
    }
    
    /// Save settings to file
    pub fn save(&self) -> Result<(), SettingsError> {
        // Create directory if needed
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut content = String::new();
        content.push_str("*// WSDG Settings Configuration\n");
        content.push_str("*// Part of WASMA (Windows Assignment System Monitoring Architecture)\n\n");
        
        // Theme section
        content.push_str("[theme]\n");
        content.push_str(&format!("name = \"{}\"\n", self.settings.theme.name));
        content.push_str(&format!("dark_mode = {}\n", self.settings.theme.dark_mode));
        content.push_str(&format!("accent_color = \"{}\"\n", self.settings.theme.accent_color));
        content.push_str(&format!("background_color = \"{}\"\n", self.settings.theme.background_color));
        content.push_str(&format!("foreground_color = \"{}\"\n", self.settings.theme.foreground_color));
        content.push_str("\n");
        
        // Font section
        content.push_str("[font]\n");
        content.push_str(&format!("family = \"{}\"\n", self.settings.font.family));
        content.push_str(&format!("size = {}\n", self.settings.font.size));
        content.push_str(&format!("weight = \"{}\"\n", self.settings.font.weight));
        content.push_str(&format!("monospace_family = \"{}\"\n", self.settings.font.monospace_family));
        content.push_str(&format!("monospace_size = {}\n", self.settings.font.monospace_size));
        content.push_str("\n");
        
        // Icon section
        content.push_str("[icon]\n");
        content.push_str(&format!("theme = \"{}\"\n", self.settings.icon.theme));
        content.push_str(&format!("size = {}\n", self.settings.icon.size));
        content.push_str(&format!("use_symbolic = {}\n", self.settings.icon.use_symbolic));
        content.push_str("\n");
        
        // Window section
        content.push_str("[window]\n");
        content.push_str(&format!("default_width = {}\n", self.settings.window.default_width));
        content.push_str(&format!("default_height = {}\n", self.settings.window.default_height));
        content.push_str(&format!("decorations = {}\n", self.settings.window.decorations));
        content.push_str(&format!("transparency = {}\n", self.settings.window.transparency));
        content.push_str(&format!("opacity = {:.2}\n", self.settings.window.opacity));
        content.push_str("\n");
        
        // Custom settings
        if !self.settings.custom.is_empty() {
            content.push_str("[custom]\n");
            for (key, value) in &self.settings.custom {
                content.push_str(&format!("{} = \"{}\"\n", key, value));
            }
        }
        
        fs::write(&self.settings_path, content)
            .map_err(|e| SettingsError::SaveFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get current settings
    pub fn settings(&self) -> &WsdgSettings {
        &self.settings
    }
    
    /// Get mutable settings
    pub fn settings_mut(&mut self) -> &mut WsdgSettings {
        &mut self.settings
    }
    
    /// Reset to defaults
    pub fn reset_to_defaults(&mut self) {
        self.settings = WsdgSettings::default();
    }
    
    /// Get custom setting
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.settings.custom.get(key)
    }
    
    /// Set custom setting
    pub fn set_custom(&mut self, key: &str, value: &str) {
        self.settings.custom.insert(key.to_string(), value.to_string());
    }
    
    /// Load from manifest_rrt file (if supported)
    pub fn load_from_manifest_rrt(&mut self, manifest_path: &Path) -> Result<(), SettingsError> {
        if !self.manifest_rrt_support {
            return Err(SettingsError::InvalidFormat(
                "manifest_rrt support not enabled. Call enable_manifest_rrt() first.".to_string()
            ));
        }
        
        if !manifest_path.exists() {
            return Err(SettingsError::NotFound(manifest_path.display().to_string()));
        }
        
        // Parse manifest_rrt format (similar to regular settings but with extended support)
        let content = fs::read_to_string(manifest_path)?;
        self.parse_settings(&content)?;
        
        Ok(())
    }
    
    // ============================================================================
    // WASMA INTEGRATION
    // ============================================================================
    
    /// Enable WASMA integration with sync callback
    /// 
    /// This allows WASMA window manager to receive notifications when settings change
    /// 
    /// # Example
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use std::sync::Mutex;
    /// 
    /// let wasma_handler = Arc::new(WindowHandler::new(ResourceMode::Auto));
    /// let handler_clone = wasma_handler.clone();
    /// 
    /// settings_manager.enable_wasma_sync(move |settings| {
    ///     // Apply settings to all WASMA windows
    ///     let windows: Vec<u64> = handler_clone.windows.lock().unwrap().keys().cloned().collect();
    ///     for window_id in windows {
    ///         handler_clone.apply_wsdg_theme(window_id).ok();
    ///     }
    /// });
    /// ```
    pub fn enable_wasma_sync<F>(&mut self, callback: F)
    where
        F: Fn(&WsdgSettings) + Send + Sync + 'static,
    {
        self.wasma_sync_callback = Some(Box::new(callback));
        println!("✅ WASMA integration enabled for WSDG Settings");
    }
    
    /// Disable WASMA integration
    pub fn disable_wasma_sync(&mut self) {
        self.wasma_sync_callback = None;
        println!("⚠️  WASMA integration disabled");
    }
    
    /// Check if WASMA integration is enabled
    pub fn is_wasma_sync_enabled(&self) -> bool {
        self.wasma_sync_callback.is_some()
    }
    
    /// Trigger WASMA sync manually
    /// 
    /// This notifies WASMA about current settings without reloading from file
    pub fn trigger_wasma_sync(&self) {
        if let Some(ref callback) = self.wasma_sync_callback {
            callback(&self.settings);
            println!("🔄 WASMA sync triggered");
        }
    }
    
    /// Load settings and notify WASMA
    pub fn load_and_sync(&mut self) -> Result<(), SettingsError> {
        self.load()?;
        
        // Notify WASMA if callback is registered
        if let Some(ref callback) = self.wasma_sync_callback {
            callback(&self.settings);
            println!("🔄 Settings loaded and synced to WASMA");
        }
        
        Ok(())
    }
    
    /// Save settings and notify WASMA
    pub fn save_and_sync(&self) -> Result<(), SettingsError> {
        self.save()?;
        
        // Notify WASMA if callback is registered
        if let Some(ref callback) = self.wasma_sync_callback {
            callback(&self.settings);
            println!("💾 Settings saved and synced to WASMA");
        }
        
        Ok(())
    }
    
    /// Update a theme setting and sync to WASMA
    pub fn update_theme(&mut self, 
        name: Option<String>,
        dark_mode: Option<bool>,
        accent_color: Option<String>,
    ) {
        if let Some(n) = name {
            self.settings.theme.name = n;
        }
        if let Some(dm) = dark_mode {
            self.settings.theme.dark_mode = dm;
        }
        if let Some(ac) = accent_color {
            self.settings.theme.accent_color = ac;
        }
        
        // Trigger WASMA sync
        self.trigger_wasma_sync();
    }
    
    /// Update font settings and sync to WASMA
    pub fn update_font(&mut self,
        family: Option<String>,
        size: Option<u32>,
    ) {
        if let Some(f) = family {
            self.settings.font.family = f;
        }
        if let Some(s) = size {
            self.settings.font.size = s;
        }
        
        // Trigger WASMA sync
        self.trigger_wasma_sync();
    }
    
    /// Update window settings and sync to WASMA
    pub fn update_window(&mut self,
        width: Option<u32>,
        height: Option<u32>,
        decorations: Option<bool>,
        opacity: Option<f32>,
    ) {
        if let Some(w) = width {
            self.settings.window.default_width = w;
        }
        if let Some(h) = height {
            self.settings.window.default_height = h;
        }
        if let Some(d) = decorations {
            self.settings.window.decorations = d;
        }
        if let Some(o) = opacity {
            self.settings.window.opacity = o;
        }
        
        // Trigger WASMA sync
        self.trigger_wasma_sync();
    }
    
    /// Get settings for WASMA consumption
    /// Returns a clone of current settings for thread-safe access
    pub fn get_wasma_settings(&self) -> WsdgSettings {
        self.settings.clone()
    }
    
    /// Export settings as WASMA-compatible format
    /// Returns settings that can be directly applied to WASMA windows
    pub fn export_for_wasma(&self) -> WasmaCompatibleSettings {
        WasmaCompatibleSettings {
            window_width: self.settings.window.default_width,
            window_height: self.settings.window.default_height,
            window_decorations: self.settings.window.decorations,
            window_opacity: self.settings.window.opacity,
            theme_dark_mode: self.settings.theme.dark_mode,
            theme_accent_color: self.settings.theme.accent_color.clone(),
            font_family: self.settings.font.family.clone(),
            font_size: self.settings.font.size,
            icon_theme: self.settings.icon.theme.clone(),
            icon_size: self.settings.icon.size,
        }
    }
}

/// WASMA-compatible settings structure
/// 
/// This structure contains only the settings that WASMA can directly apply
#[derive(Debug, Clone)]
pub struct WasmaCompatibleSettings {
    pub window_width: u32,
    pub window_height: u32,
    pub window_decorations: bool,
    pub window_opacity: f32,
    pub theme_dark_mode: bool,
    pub theme_accent_color: String,
    pub font_family: String,
    pub font_size: u32,
    pub icon_theme: String,
    pub icon_size: u32,
}

impl WasmaCompatibleSettings {
    /// Convert hex color to RGB tuple (0.0-1.0 range)
    pub fn accent_color_rgb(&self) -> (f32, f32, f32) {
        let hex = self.theme_accent_color.trim_start_matches('#');
        
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                );
            }
        }
        
        // Default blue
        (0.2, 0.6, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wsdg_env::WsdgEnvBuilder;
    
    #[test]
    fn test_settings_creation() {
        let env = WsdgEnvBuilder::new().build();
        let manager = WsdgSettingsManager::new(env);
        
        assert_eq!(manager.settings.font.family, "Sans");
        assert_eq!(manager.settings.icon.size, 48);
    }
    
    #[test]
    fn test_settings_parsing() {
        let env = WsdgEnvBuilder::new().build();
        let mut manager = WsdgSettingsManager::new(env);
        
        let content = r#"
[theme]
name = "dark"
dark_mode = true

[font]
family = "Ubuntu"
size = 12
        "#;
        
        manager.parse_settings(content).unwrap();
        
        assert_eq!(manager.settings.theme.name, "dark");
        assert_eq!(manager.settings.theme.dark_mode, true);
        assert_eq!(manager.settings.font.family, "Ubuntu");
        assert_eq!(manager.settings.font.size, 12);
    }
    
    #[test]
    fn test_custom_settings() {
        let env = WsdgEnvBuilder::new().build();
        let mut manager = WsdgSettingsManager::new(env);
        
        manager.set_custom("my_key", "my_value");
        assert_eq!(manager.get_custom("my_key"), Some(&"my_value".to_string()));
    }
    
    #[test]
    fn test_wasma_integration() {
        use std::sync::{Arc, Mutex};
        
        let env = WsdgEnvBuilder::new().build();
        let mut manager = WsdgSettingsManager::new(env);
        
        // Track sync calls
        let sync_count = Arc::new(Mutex::new(0));
        let sync_count_clone = sync_count.clone();
        
        // Enable WASMA sync
        manager.enable_wasma_sync(move |settings| {
            *sync_count_clone.lock().unwrap() += 1;
            println!("WASMA sync triggered with theme: {}", settings.theme.name);
        });
        
        assert!(manager.is_wasma_sync_enabled());
        
        // Trigger sync
        manager.trigger_wasma_sync();
        assert_eq!(*sync_count.lock().unwrap(), 1);
        
        // Update theme (should trigger sync)
        manager.update_theme(Some("dark".to_string()), Some(true), None);
        assert_eq!(*sync_count.lock().unwrap(), 2);
        
        println!("✅ WASMA integration test passed");
    }
    
    #[test]
    fn test_wasma_compatible_export() {
        let env = WsdgEnvBuilder::new().build();
        let manager = WsdgSettingsManager::new(env);
        
        let wasma_settings = manager.export_for_wasma();
        
        assert_eq!(wasma_settings.window_width, 800);
        assert_eq!(wasma_settings.window_height, 600);
        assert_eq!(wasma_settings.font_family, "Sans");
        
        // Test color conversion
        let (r, g, b) = wasma_settings.accent_color_rgb();
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
        
        println!("✅ WASMA export test passed");
    }
    
    #[test]
    fn test_load_and_sync() {
        use std::sync::{Arc, Mutex};
        
        let env = WsdgEnvBuilder::new().build();
        let mut manager = WsdgSettingsManager::new(env);
        
        let synced = Arc::new(Mutex::new(false));
        let synced_clone = synced.clone();
        
        manager.enable_wasma_sync(move |_| {
            *synced_clone.lock().unwrap() = true;
        });
        
        // load_and_sync should trigger callback even if file doesn't exist
        manager.load_and_sync().ok();
        assert!(*synced.lock().unwrap());
        
        println!("✅ Load and sync test passed");
    }
}

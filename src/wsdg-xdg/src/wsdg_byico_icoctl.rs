// WSDG ByIco IcoCtl - Icon Discovery and Control
// Automatically finds icons in /usr/share/pixmaps/ico/12-256
// Supports sizes from 12x12 to 256x256
// Finds icons for manifest files and appstarter by name
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IconError {
    #[error("Icon not found: {0}")]
    IconNotFound(String),
    
    #[error("Invalid icon size: {0}")]
    InvalidSize(u32),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Standard icon sizes supported by WSDG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Size12,
    Size16,
    Size24,
    Size32,
    Size48,
    Size64,
    Size96,
    Size128,
    Size256,
}

impl IconSize {
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Size12 => 12,
            Self::Size16 => 16,
            Self::Size24 => 24,
            Self::Size32 => 32,
            Self::Size48 => 48,
            Self::Size64 => 64,
            Self::Size96 => 96,
            Self::Size128 => 128,
            Self::Size256 => 256,
        }
    }
    
    pub fn from_u32(size: u32) -> Result<Self, IconError> {
        match size {
            12 => Ok(Self::Size12),
            16 => Ok(Self::Size16),
            24 => Ok(Self::Size24),
            32 => Ok(Self::Size32),
            48 => Ok(Self::Size48),
            64 => Ok(Self::Size64),
            96 => Ok(Self::Size96),
            128 => Ok(Self::Size128),
            256 => Ok(Self::Size256),
            _ => Err(IconError::InvalidSize(size)),
        }
    }
    
    pub fn all_sizes() -> Vec<IconSize> {
        vec![
            Self::Size12,
            Self::Size16,
            Self::Size24,
            Self::Size32,
            Self::Size48,
            Self::Size64,
            Self::Size96,
            Self::Size128,
            Self::Size256,
        ]
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::Size12 => "12",
            Self::Size16 => "16",
            Self::Size24 => "24",
            Self::Size32 => "32",
            Self::Size48 => "48",
            Self::Size64 => "64",
            Self::Size96 => "96",
            Self::Size128 => "128",
            Self::Size256 => "256",
        }
    }
}

/// Icon information
#[derive(Debug, Clone)]
pub struct IconInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: IconSize,
    pub format: IconFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconFormat {
    Png,
    Svg,
    Xpm,
    Ico,
}

impl IconFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "xpm" => Some(Self::Xpm),
            "ico" => Some(Self::Ico),
            _ => None,
        }
    }
    
    pub fn extension(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Xpm => "xpm",
            Self::Ico => "ico",
        }
    }
}

/// WSDG Icon Controller - Icon discovery and management
pub struct WsdgIcoCtl {
    icon_dirs: Vec<PathBuf>,
    cache: HashMap<String, Vec<IconInfo>>,
}

impl WsdgIcoCtl {
    pub fn new() -> Self {
        Self {
            icon_dirs: Self::get_icon_directories(),
            cache: HashMap::new(),
        }
    }
    
    /// Get standard icon directories
    fn get_icon_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // WSDG standard icon directory
        dirs.push(PathBuf::from("/usr/share/pixmaps/ico"));
        
        // Hicolor icon theme (XDG standard)
        for size in IconSize::all_sizes() {
            let size_str = size.to_u32().to_string();
            dirs.push(PathBuf::from(format!(
                "/usr/share/icons/hicolor/{}x{}/apps",
                size_str, size_str
            )));
        }
        
        // Additional standard directories
        dirs.push(PathBuf::from("/usr/share/pixmaps"));
        dirs.push(PathBuf::from("/usr/share/icons"));
        
        // User directories
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/icons/hicolor"));
            dirs.push(home.join(".local/share/pixmaps"));
        }
        
        dirs
    }
    
    /// Find icon by name, searching in all directories and sizes
    pub fn find_icon(&mut self, name: &str, preferred_size: Option<IconSize>) -> Option<IconInfo> {
        // Check cache
        if let Some(cached_icons) = self.cache.get(name) {
            return self.select_best_icon(cached_icons, preferred_size);
        }
        
        // Search for icons
        let mut found_icons = Vec::new();
        
        for dir in &self.icon_dirs {
            if !dir.exists() {
                continue;
            }
            
            // Try different sizes
            for size in IconSize::all_sizes() {
                let size_dir = dir.join(size.as_str());
                
                if size_dir.exists() {
                    if let Some(icon) = self.search_in_directory(&size_dir, name, size) {
                        found_icons.push(icon);
                    }
                }
            }
            
            // Try root directory (for SVG and size-independent icons)
            if let Some(icon) = self.search_in_directory(dir, name, IconSize::Size48) {
                found_icons.push(icon);
            }
        }
        
        if !found_icons.is_empty() {
            self.cache.insert(name.to_string(), found_icons.clone());
            self.select_best_icon(&found_icons, preferred_size)
        } else {
            None
        }
    }
    
    /// Search for icon in specific directory
    fn search_in_directory(&self, dir: &Path, name: &str, size: IconSize) -> Option<IconInfo> {
        // Try different extensions
        let extensions = ["png", "svg", "xpm", "ico"];
        
        for ext in &extensions {
            let icon_path = dir.join(format!("{}.{}", name, ext));
            
            if icon_path.exists() {
                return Some(IconInfo {
                    name: name.to_string(),
                    path: icon_path,
                    size,
                    format: IconFormat::from_extension(ext).unwrap(),
                });
            }
        }
        
        None
    }
    
    /// Select best icon from available options
    fn select_best_icon(&self, icons: &[IconInfo], preferred_size: Option<IconSize>) -> Option<IconInfo> {
        if icons.is_empty() {
            return None;
        }
        
        if let Some(pref_size) = preferred_size {
            // Try exact match first
            if let Some(icon) = icons.iter().find(|i| i.size == pref_size) {
                return Some(icon.clone());
            }
            
            // Find closest size
            let target = pref_size.to_u32();
            let mut closest = icons[0].clone();
            let mut min_diff = (closest.size.to_u32() as i32 - target as i32).abs();
            
            for icon in icons.iter().skip(1) {
                let diff = (icon.size.to_u32() as i32 - target as i32).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest = icon.clone();
                }
            }
            
            Some(closest)
        } else {
            // Prefer PNG, then SVG, then others
            icons.iter()
                .find(|i| i.format == IconFormat::Png)
                .or_else(|| icons.iter().find(|i| i.format == IconFormat::Svg))
                .or_else(|| icons.first())
                .cloned()
        }
    }
    
    /// Find icon for application by desktop file or manifest name
    pub fn find_app_icon(&mut self, app_name: &str, size: Option<IconSize>) -> Option<IconInfo> {
        // Remove .desktop or .manifest extension if present
        let clean_name = app_name
            .trim_end_matches(".desktop")
            .trim_end_matches(".manifest");
        
        self.find_icon(clean_name, size)
    }
    
    /// Get all available sizes for an icon
    pub fn get_available_sizes(&mut self, name: &str) -> Vec<IconSize> {
        if let Some(icons) = self.cache.get(name) {
            icons.iter().map(|i| i.size).collect()
        } else {
            // Force search
            self.find_icon(name, None);
            self.cache.get(name)
                .map(|icons| icons.iter().map(|i| i.size).collect())
                .unwrap_or_default()
        }
    }
    
    /// List all icons in a directory
    pub fn list_icons_in_dir(&self, dir: &Path) -> Vec<String> {
        let mut icon_names = Vec::new();
        
        if !dir.exists() {
            return icon_names;
        }
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if IconFormat::from_extension(ext).is_some() {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                icon_names.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        icon_names.sort();
        icon_names.dedup();
        icon_names
    }
    
    /// Search for icons matching a pattern
    pub fn search_icons(&self, pattern: &str) -> Vec<String> {
        let mut results = Vec::new();
        let pattern = pattern.to_lowercase();
        
        for dir in &self.icon_dirs {
            let icons = self.list_icons_in_dir(dir);
            for icon in icons {
                if icon.to_lowercase().contains(&pattern) {
                    results.push(icon);
                }
            }
        }
        
        results.sort();
        results.dedup();
        results
    }
    
    /// Clear icon cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Add custom icon directory
    pub fn add_icon_directory(&mut self, dir: PathBuf) {
        if !self.icon_dirs.contains(&dir) {
            self.icon_dirs.push(dir);
        }
    }
}

impl Default for WsdgIcoCtl {
    fn default() -> Self {
        Self::new()
    }
}

/// Icon theme manager for handling icon themes
pub struct IconThemeManager {
    current_theme: String,
    theme_dirs: Vec<PathBuf>,
}

impl IconThemeManager {
    pub fn new(theme_name: &str) -> Self {
        Self {
            current_theme: theme_name.to_string(),
            theme_dirs: Self::get_theme_directories(),
        }
    }
    
    fn get_theme_directories() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/usr/share/icons"),
            dirs::home_dir().map(|h| h.join(".local/share/icons")).unwrap_or_default(),
        ]
    }
    
    pub fn set_theme(&mut self, theme: &str) {
        self.current_theme = theme.to_string();
    }
    
    pub fn get_theme_path(&self) -> Option<PathBuf> {
        for dir in &self.theme_dirs {
            let theme_path = dir.join(&self.current_theme);
            if theme_path.exists() {
                return Some(theme_path);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_icon_size_conversion() {
        assert_eq!(IconSize::Size48.to_u32(), 48);
        assert_eq!(IconSize::from_u32(48).unwrap(), IconSize::Size48);
        assert!(IconSize::from_u32(50).is_err());
    }
    
    #[test]
    fn test_icon_format_detection() {
        assert_eq!(IconFormat::from_extension("png"), Some(IconFormat::Png));
        assert_eq!(IconFormat::from_extension("PNG"), Some(IconFormat::Png));
        assert_eq!(IconFormat::from_extension("svg"), Some(IconFormat::Svg));
        assert_eq!(IconFormat::from_extension("unknown"), None);
    }
    
    #[test]
    fn test_icon_controller_creation() {
        let ico_ctl = WsdgIcoCtl::new();
        assert!(!ico_ctl.icon_dirs.is_empty());
    }
    
    #[test]
    fn test_app_name_cleaning() {
        // These should all search for the same icon
        let names = vec![
            "firefox",
            "firefox.desktop",
            "firefox.manifest",
        ];
        
        for name in names {
            let clean = name.trim_end_matches(".desktop").trim_end_matches(".manifest");
            assert_eq!(clean, "firefox");
        }
    }
}

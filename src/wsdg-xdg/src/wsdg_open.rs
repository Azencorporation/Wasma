// WSDG Open - Application Launcher
// Provides basic application support for WSDG environment
// Supports WSDG environment variables and XDG-translated applications
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::{Path, PathBuf};
use std::process::{Command, Child};
use std::collections::HashMap;
use thiserror::Error;

use crate::xdg_wsdg_translate::XdgWsdgTranslator;
use crate::wsdg_env::WsdgEnv;

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Application not found: {0}")]
    AppNotFound(String),
    
    #[error("Failed to launch: {0}")]
    LaunchFailed(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Translation error: {0}")]
    TranslationError(String),
    
    #[error("No handler for file type: {0}")]
    NoHandler(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Application info from desktop files or manifests
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub mime_types: Vec<String>,
    pub terminal: bool,
}

/// WSDG Open - Application launcher
pub struct WsdgOpen {
    pub(crate) env: WsdgEnv,
    translator: Option<XdgWsdgTranslator>,
    app_cache: HashMap<String, AppInfo>,
    desktop_dirs: Vec<PathBuf>,
}

impl WsdgOpen {
    pub fn new(env: WsdgEnv) -> Self {
        let desktop_dirs = Self::get_desktop_dirs(&env);
        
        Self {
            env,
            translator: None,
            app_cache: HashMap::new(),
            desktop_dirs,
        }
    }
    
    pub fn with_translator(mut self, translator: XdgWsdgTranslator) -> Self {
        self.translator = Some(translator);
        self
    }
    
    /// Get desktop file directories
    fn get_desktop_dirs(env: &WsdgEnv) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // WSDG directories
        if let Ok(share) = env.share_dir() {
            dirs.push(share.join("applications"));
        }
        
        if let Ok(local) = env.local_dir() {
            dirs.push(local.join("share/applications"));
        }
        
        // System directories
        dirs.push(PathBuf::from("/usr/share/applications"));
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        
        dirs
    }
    
    /// Open a file with appropriate application
    pub fn open(&mut self, path: &str) -> Result<Child, OpenError> {
        let path = self.resolve_path(path)?;
        
        // Determine how to open based on path type
        if path.is_file() {
            self.open_file(&path)
        } else if path.is_dir() {
            self.open_directory(&path)
        } else {
            Err(OpenError::InvalidPath(path.to_string_lossy().to_string()))
        }
    }
    
    /// Open a file with default application
    fn open_file(&mut self, path: &Path) -> Result<Child, OpenError> {
        // Get MIME type
        let mime_type = self.get_mime_type(path)?;
        
        // Find handler for MIME type
        let handler = self.find_mime_handler(&mime_type)
            .ok_or_else(|| OpenError::NoHandler(mime_type.clone()))?;
        
        // Launch with handler
        self.launch_app(&handler.exec, &[path.to_string_lossy().as_ref()])
    }
    
    /// Open directory with file manager
    fn open_directory(&mut self, path: &Path) -> Result<Child, OpenError> {
        // Try common file managers
        let file_managers = vec![
            "nautilus",
            "dolphin",
            "thunar",
            "pcmanfm",
            "nemo",
        ];
        
        for fm in file_managers {
            if let Ok(child) = self.launch_app(fm, &[path.to_string_lossy().as_ref()]) {
                return Ok(child);
            }
        }
        
        Err(OpenError::AppNotFound("file manager".to_string()))
    }
    
    /// Open application by name
    pub fn open_app(&mut self, app_name: &str, args: &[&str]) -> Result<Child, OpenError> {
        // Try to find in cache first
        if let Some(app_info) = self.app_cache.get(app_name) {
            return self.launch_app(&app_info.exec, args);
        }
        
        // Search for desktop file
        if let Some(app_info) = self.find_desktop_file(app_name)? {
            self.app_cache.insert(app_name.to_string(), app_info.clone());
            return self.launch_app(&app_info.exec, args);
        }
        
        // Try direct execution
        self.launch_app(app_name, args)
    }
    
    /// Find desktop file for application
    fn find_desktop_file(&self, app_name: &str) -> Result<Option<AppInfo>, OpenError> {
        let desktop_filename = if app_name.ends_with(".desktop") {
            app_name.to_string()
        } else {
            format!("{}.desktop", app_name)
        };
        
        for dir in &self.desktop_dirs {
            let desktop_path = dir.join(&desktop_filename);
            if desktop_path.exists() {
                return Ok(Some(self.parse_desktop_file(&desktop_path)?));
            }
        }
        
        Ok(None)
    }
    
    /// Parse desktop file
    fn parse_desktop_file(&self, path: &Path) -> Result<AppInfo, OpenError> {
        let content = std::fs::read_to_string(path)?;
        
        let mut name = String::new();
        let mut exec = String::new();
        let mut icon = None;
        let mut categories = Vec::new();
        let mut mime_types = Vec::new();
        let mut terminal = false;
        
        let mut in_desktop_entry = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }
            
            if line.starts_with('[') && line != "[Desktop Entry]" {
                in_desktop_entry = false;
                continue;
            }
            
            if !in_desktop_entry {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => name = value.trim().to_string(),
                    "Exec" => exec = value.trim().to_string(),
                    "Icon" => icon = Some(value.trim().to_string()),
                    "Categories" => {
                        categories = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    "MimeType" => {
                        mime_types = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    "Terminal" => terminal = value.trim() == "true",
                    _ => {}
                }
            }
        }
        
        Ok(AppInfo {
            name,
            exec,
            icon,
            categories,
            mime_types,
            terminal,
        })
    }
    
    /// Launch application with arguments
    fn launch_app(&self, exec: &str, args: &[&str]) -> Result<Child, OpenError> {
        // Parse exec line (handle %f, %F, %u, %U placeholders)
        let exec_parts = self.parse_exec_line(exec, args);
        
        if exec_parts.is_empty() {
            return Err(OpenError::LaunchFailed("Empty exec command".to_string()));
        }
        
        let mut cmd = Command::new(&exec_parts[0]);
        
        if exec_parts.len() > 1 {
            cmd.args(&exec_parts[1..]);
        }
        
        // Set WSDG environment
        for (key, value) in self.env.all_vars() {
            cmd.env(key, value);
        }
        
        cmd.spawn()
            .map_err(|e| OpenError::LaunchFailed(format!("{}: {}", exec_parts[0], e)))
    }
    
    /// Parse desktop file Exec line
    fn parse_exec_line(&self, exec: &str, args: &[&str]) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut chars = exec.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next) = chars.peek() {
                    chars.next(); // consume
                    match next {
                        'f' | 'F' | 'u' | 'U' => {
                            // Add file arguments
                            if !current.is_empty() {
                                result.push(current.clone());
                                current.clear();
                            }
                            for arg in args {
                                result.push(arg.to_string());
                            }
                        }
                        '%' => current.push('%'),
                        _ => {} // Ignore other placeholders
                    }
                } else {
                    current.push(ch);
                }
            } else if ch.is_whitespace() {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        
        if !current.is_empty() {
            result.push(current);
        }
        
        result
    }
    
    /// Resolve path (handle WSDG variables and XDG translation)
    fn resolve_path(&mut self, path: &str) -> Result<PathBuf, OpenError> {
        // Handle XDG paths
        if path.starts_with("XDG_") {
            if let Some(translator) = &mut self.translator {
                return translator.translate_xdg(path)
                    .map_err(|e| OpenError::TranslationError(e.to_string()));
            }
        }
        
        // Handle WSDG variables
        if path.starts_with('$') {
            let expanded = self.expand_wsdg_path(path)?;
            return Ok(expanded);
        }
        
        // Regular path
        Ok(PathBuf::from(path))
    }
    
    /// Expand WSDG path variables
    fn expand_wsdg_path(&self, path: &str) -> Result<PathBuf, OpenError> {
        if path.starts_with("$HOME") {
            let rest = path.strip_prefix("$HOME").unwrap_or("");
            Ok(self.env.home_dir()
                .map_err(|e| OpenError::TranslationError(e.to_string()))?
                .join(rest.trim_start_matches('/')))
        } else if path.starts_with("$CONFIG") {
            let rest = path.strip_prefix("$CONFIG").unwrap_or("");
            Ok(self.env.config_dir()
                .map_err(|e| OpenError::TranslationError(e.to_string()))?
                .join(rest.trim_start_matches('/')))
        } else {
            Ok(PathBuf::from(path))
        }
    }
    
    /// Get MIME type for file
    fn get_mime_type(&self, _path: &Path) -> Result<String, OpenError> {
        // This would integrate with wsdg-mime-array.rs
        // For now, simple extension-based detection
        Ok("application/octet-stream".to_string())
    }
    
    /// Find handler for MIME type
    fn find_mime_handler(&self, _mime_type: &str) -> Option<AppInfo> {
        // This would search through desktop files for MIME handlers
        // For now, return None (will use system default)
        None
    }
    
    /// Get all installed applications
    pub fn list_applications(&self) -> Vec<AppInfo> {
        let mut apps = Vec::new();
        
        for dir in &self.desktop_dirs {
            if !dir.exists() {
                continue;
            }
            
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Ok(app_info) = self.parse_desktop_file(&path) {
                            apps.push(app_info);
                        }
                    }
                }
            }
        }
        
        apps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_exec_line() {
        let env = WsdgEnv::new();
        let opener = WsdgOpen::new(env);
        
        let result = opener.parse_exec_line("myapp %f", &["file.txt"]);
        assert_eq!(result, vec!["myapp", "file.txt"]);
        
        let result = opener.parse_exec_line("viewer %F", &["a.jpg", "b.jpg"]);
        assert_eq!(result, vec!["viewer", "a.jpg", "b.jpg"]);
    }
    
    #[test]
    fn test_expand_wsdg_path() {
        let env = crate::wsdg_env::WsdgEnvBuilder::new()
            .home("/home/user")
            .config("/home/user/.config")
            .build();
        
        let opener = WsdgOpen::new(env);
        
        let path = opener.expand_wsdg_path("$HOME/Documents").unwrap();
        assert_eq!(path, PathBuf::from("/home/user/Documents"));
        
        let path = opener.expand_wsdg_path("$CONFIG/app").unwrap();
        assert_eq!(path, PathBuf::from("/home/user/.config/app"));
    }
}

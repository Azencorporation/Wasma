// WSDG Manifest Open - Manifest-based Application Opener
// Opens applications via manifest files with custom app:// URI support
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::{Path, PathBuf};
use std::process::{Command, Child};
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

use crate::wsdg_open::{WsdgOpen, OpenError};
use crate::wsdg_env::WsdgEnv;

// Import manifest parser if available
#[cfg(feature = "manifest")]
use wsdg_app_manifest::{WasmaManifest, ManifestParser};

#[derive(Debug, Error)]
pub enum ManifestOpenError {
    #[error("Manifest not found: {0}")]
    ManifestNotFound(String),
    
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("App URI not defined in manifest")]
    NoAppUri,
    
    #[error("Failed to launch: {0}")]
    LaunchFailed(String),
    
    #[error("Open error: {0}")]
    OpenError(#[from] OpenError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Manifest directories to search
const MANIFEST_DIRS: &[&str] = &[
    "/usr/share/manifest_app",
    "/usr/share/manifest_rrt",
    "/usr/share/applications/manifest",
    "/usr/share/applications/manifest_rrt",
];

/// User manifest directories (relative to home)
const USER_MANIFEST_DIRS: &[&str] = &[
    ".local/share/manifest_app",
    ".local/share/manifest_rrt",
    ".local/share/applications/manifest",
];

/// Manifest info extracted from .manifest files
#[derive(Debug, Clone)]
pub struct ManifestAppInfo {
    pub name: String,
    pub app_uri: Option<String>,
    pub exec_path: Option<String>,
    pub manifest_path: PathBuf,
}

/// WSDG Manifest Opener - Opens apps via manifest files
pub struct WsdgManifestOpen {
    wsdg_open: WsdgOpen,
    env: WsdgEnv,
    manifest_cache: HashMap<String, ManifestAppInfo>,
    manifest_dirs: Vec<PathBuf>,
}

impl WsdgManifestOpen {
    pub fn new(wsdg_open: WsdgOpen, env: WsdgEnv) -> Self {
        let manifest_dirs = Self::get_manifest_dirs(&env);
        
        Self {
            wsdg_open,
            env,
            manifest_cache: HashMap::new(),
            manifest_dirs,
        }
    }
    
    /// Get manifest directories
    fn get_manifest_dirs(env: &WsdgEnv) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // System directories
        for dir in MANIFEST_DIRS {
            dirs.push(PathBuf::from(dir));
        }
        
        // User directories
        if let Ok(home) = env.home_dir() {
            for dir in USER_MANIFEST_DIRS {
                dirs.push(home.join(dir));
            }
        }
        
        dirs
    }
    
    /// Open application via custom app:// URI
    /// Format: app://app_name or app://app_name/action
    pub fn open_app_uri(&mut self, uri: &str) -> Result<Child, ManifestOpenError> {
        // Parse app:// URI
        let uri = uri.trim_start_matches("app://");
        let parts: Vec<&str> = uri.split('/').collect();
        
        if parts.is_empty() {
            return Err(ManifestOpenError::InvalidManifest(
                "Empty app URI".to_string()
            ));
        }
        
        let app_name = parts[0];
        let action = if parts.len() > 1 {
            Some(parts[1..].join("/"))
        } else {
            None
        };
        
        // Find manifest for app
        let manifest_info = self.find_manifest(app_name)?;
        
        // Launch app with manifest info
        self.launch_from_manifest(&manifest_info, action.as_deref())
    }
    
    /// Open application by manifest file path
    pub fn open_manifest(&mut self, manifest_path: &Path) -> Result<Child, ManifestOpenError> {
        if !manifest_path.exists() {
            return Err(ManifestOpenError::ManifestNotFound(
                manifest_path.display().to_string()
            ));
        }
        
        let manifest_info = self.parse_manifest_file(manifest_path)?;
        self.launch_from_manifest(&manifest_info, None)
    }
    
    /// Find manifest by app name
    fn find_manifest(&mut self, app_name: &str) -> Result<ManifestAppInfo, ManifestOpenError> {
        // Check cache first
        if let Some(cached) = self.manifest_cache.get(app_name) {
            return Ok(cached.clone());
        }
        
        // Search for manifest file
        let manifest_filename = if app_name.ends_with(".manifest") {
            app_name.to_string()
        } else {
            format!("{}.manifest", app_name)
        };
        
        for dir in &self.manifest_dirs {
            let manifest_path = dir.join(&manifest_filename);
            if manifest_path.exists() {
                let manifest_info = self.parse_manifest_file(&manifest_path)?;
                self.manifest_cache.insert(app_name.to_string(), manifest_info.clone());
                return Ok(manifest_info);
            }
        }
        
        Err(ManifestOpenError::ManifestNotFound(app_name.to_string()))
    }
    
    /// Parse manifest file (lightweight parsing for app_uri)
    fn parse_manifest_file(&self, path: &Path) -> Result<ManifestAppInfo, ManifestOpenError> {
        let content = fs::read_to_string(path)?;
        
        let mut name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let mut app_uri = None;
        let mut exec_path = None;
        
        // Simple line-by-line parsing
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with("*//") || line.starts_with("//") {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.split("*//").next().unwrap_or(value).trim();
                let value = value.trim_matches('"').trim_matches('\'');
                
                match key {
                    "name" => name = value.to_string(),
                    "app_uri" => app_uri = Some(value.to_string()),
                    "uri_app_source" => {
                        // Extract path from file:// URI
                        let exec = value.trim_start_matches("file://")
                            .split_whitespace()
                            .next()
                            .unwrap_or(value);
                        exec_path = Some(exec.to_string());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(ManifestAppInfo {
            name,
            app_uri,
            exec_path,
            manifest_path: path.to_path_buf(),
        })
    }
    
    /// Launch application from manifest info
    fn launch_from_manifest(
        &mut self,
        manifest_info: &ManifestAppInfo,
        action: Option<&str>,
    ) -> Result<Child, ManifestOpenError> {
        // If app_uri is defined, use it
        if let Some(app_uri) = &manifest_info.app_uri {
            return self.launch_via_app_uri(app_uri, action);
        }
        
        // Otherwise use exec_path
        if let Some(exec_path) = &manifest_info.exec_path {
            return self.launch_via_exec(exec_path, action);
        }
        
        Err(ManifestOpenError::NoAppUri)
    }
    
    /// Launch via custom app_uri protocol
    fn launch_via_app_uri(
        &mut self,
        app_uri: &str,
        action: Option<&str>,
    ) -> Result<Child, ManifestOpenError> {
        // app_uri format: protocol://handler or direct://path
        
        if app_uri.starts_with("file://") {
            // Direct file execution
            let path = app_uri.trim_start_matches("file://");
            self.launch_via_exec(path, action)
        } else if app_uri.starts_with("wsdg://") {
            // WSDG internal handler
            let handler = app_uri.trim_start_matches("wsdg://");
            self.launch_wsdg_handler(handler, action)
        } else if app_uri.contains("://") {
            // Custom protocol - delegate to system
            let full_uri = if let Some(act) = action {
                format!("{}/{}", app_uri, act)
            } else {
                app_uri.to_string()
            };
            
            self.wsdg_open.open(&full_uri)
                .map_err(ManifestOpenError::OpenError)
        } else {
            // Treat as executable name
            self.launch_via_exec(app_uri, action)
        }
    }
    
    /// Launch via executable path
    fn launch_via_exec(
        &self,
        exec_path: &str,
        action: Option<&str>,
    ) -> Result<Child, ManifestOpenError> {
        let mut cmd = Command::new(exec_path);
        
        if let Some(act) = action {
            cmd.arg(act);
        }
        
        // Set WSDG environment
        for (key, value) in self.env.all_vars() {
            cmd.env(key, value);
        }
        
        cmd.spawn()
            .map_err(|e| ManifestOpenError::LaunchFailed(
                format!("{}: {}", exec_path, e)
            ))
    }
    
    /// Launch WSDG internal handler
    fn launch_wsdg_handler(
        &mut self,
        handler: &str,
        action: Option<&str>,
    ) -> Result<Child, ManifestOpenError> {
        // WSDG handlers are special built-in handlers
        match handler {
            "settings" => {
                // Open WSDG settings
                self.wsdg_open.open_app("wsdg-settings", &[])
                    .map_err(ManifestOpenError::OpenError)
            }
            "terminal" => {
                // Open terminal
                let terminals = vec!["wsdg-terminal", "gnome-terminal", "konsole", "xterm"];
                for term in terminals {
                    if let Ok(child) = self.wsdg_open.open_app(term, &[]) {
                        return Ok(child);
                    }
                }
                Err(ManifestOpenError::LaunchFailed("No terminal found".to_string()))
            }
            _ => {
                // Try to find as application
                let args = action.map(|a| vec![a]).unwrap_or_default();
                self.wsdg_open.open_app(handler, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    .map_err(ManifestOpenError::OpenError)
            }
        }
    }
    
    /// List all available manifest applications
    pub fn list_manifest_apps(&self) -> Vec<ManifestAppInfo> {
        let mut apps = Vec::new();
        
        for dir in &self.manifest_dirs {
            if !dir.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("manifest") {
                        if let Ok(manifest_info) = self.parse_manifest_file(&path) {
                            apps.push(manifest_info);
                        }
                    }
                }
            }
        }
        
        apps
    }
    
    /// Check if an app URI is registered
    pub fn has_app_uri(&mut self, app_name: &str) -> bool {
        self.find_manifest(app_name).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wsdg_env::WsdgEnvBuilder;
    
    #[test]
    fn test_parse_app_uri() {
        let uri = "app://myapp/action/subaction";
        let uri = uri.trim_start_matches("app://");
        let parts: Vec<&str> = uri.split('/').collect();
        
        assert_eq!(parts[0], "myapp");
        assert_eq!(parts[1..].join("/"), "action/subaction");
    }
    
    #[test]
    fn test_manifest_info_parsing() {
        let content = r#"
name = TestApp
app_uri = file:///usr/bin/testapp
uri_app_source = file://usr/bin/testapp
        "#;
        
        // Create a temp file for testing
        let temp_dir = std::env::temp_dir();
        let manifest_path = temp_dir.join("test.manifest");
        std::fs::write(&manifest_path, content).unwrap();
        
        let env = WsdgEnvBuilder::new().build();
        let wsdg_open = WsdgOpen::new(env.clone());
        let opener = WsdgManifestOpen::new(wsdg_open, env);
        
        let info = opener.parse_manifest_file(&manifest_path).unwrap();
        assert_eq!(info.name, "TestApp");
        assert_eq!(info.app_uri, Some("file:///usr/bin/testapp".to_string()));
        
        // Cleanup
        let _ = std::fs::remove_file(manifest_path);
    }
}

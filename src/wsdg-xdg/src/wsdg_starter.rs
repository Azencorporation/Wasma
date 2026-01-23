// WSDG Starter - Application Startup System
// Provides basic application startup settings, configuration and protocol support
// Supports appstarter or any appid starter
// Application name must end with "starter" and be formatted as .config
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::process::{Command, Child};
use thiserror::Error;

use crate::wsdg_env::WsdgEnv;

#[derive(Debug, Error)]
pub enum StarterError {
    #[error("Starter config not found: {0}")]
    ConfigNotFound(String),
    
    #[error("Invalid starter config: {0}")]
    InvalidConfig(String),
    
    #[error("Application not found: {0}")]
    AppNotFound(String),
    
    #[error("Failed to start: {0}")]
    StartFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Startup configuration for an application
#[derive(Debug, Clone)]
pub struct StarterConfig {
    pub app_name: String,
    pub exec_path: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub autostart: bool,
    pub delay: Option<u32>,
    pub protocol: Option<String>,
    pub icon: Option<String>,
}

impl Default for StarterConfig {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            exec_path: String::new(),
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            autostart: false,
            delay: None,
            protocol: None,
            icon: None,
        }
    }
}

/// WSDG Starter - Application startup manager
pub struct WsdgStarter {
    env: WsdgEnv,
    config_dirs: Vec<PathBuf>,
    configs: HashMap<String, StarterConfig>,
}

impl WsdgStarter {
    pub fn new(env: WsdgEnv) -> Self {
        Self {
            config_dirs: Self::get_config_directories(&env),
            env,
            configs: HashMap::new(),
        }
    }
    
    /// Get starter configuration directories
    fn get_config_directories(env: &WsdgEnv) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // System config directories
        dirs.push(PathBuf::from("/etc/wsdg/appstarter"));
        dirs.push(PathBuf::from("/usr/share/wsdg/appstarter"));
        
        // User config directories
        if let Ok(config_dir) = env.config_dir() {
            dirs.push(config_dir.join("wsdg/appstarter"));
        }
        
        if let Ok(local_dir) = env.local_dir() {
            dirs.push(local_dir.join("share/wsdg/appstarter"));
        }
        
        dirs
    }
    
    /// Load starter configuration from file
    pub fn load_config(&mut self, app_name: &str) -> Result<StarterConfig, StarterError> {
        // Check cache first
        if let Some(cached) = self.configs.get(app_name) {
            return Ok(cached.clone());
        }
        
        // Search for config file
        let config_filename = if app_name.ends_with("starter.config") {
            app_name.to_string()
        } else if app_name.ends_with("starter") {
            format!("{}.config", app_name)
        } else {
            format!("{}starter.config", app_name)
        };
        
        for dir in &self.config_dirs {
            let config_path = dir.join(&config_filename);
            if config_path.exists() {
                let config = self.parse_config_file(&config_path, app_name)?;
                self.configs.insert(app_name.to_string(), config.clone());
                return Ok(config);
            }
        }
        
        Err(StarterError::ConfigNotFound(app_name.to_string()))
    }
    
    /// Parse starter configuration file
    fn parse_config_file(&self, path: &Path, app_name: &str) -> Result<StarterConfig, StarterError> {
        let content = fs::read_to_string(path)?;
        
        let mut config = StarterConfig {
            app_name: app_name.to_string(),
            ..Default::default()
        };
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("*//") || line.starts_with("#") {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.split("*//").next().unwrap_or(value).trim();
                let value = value.trim_matches('"').trim_matches('\'');
                
                match key {
                    "exec" | "exec_path" => config.exec_path = value.to_string(),
                    "args" => config.args = value.split_whitespace()
                        .map(|s| s.to_string())
                        .collect(),
                    "working_dir" | "workdir" => {
                        config.working_dir = Some(PathBuf::from(value));
                    }
                    "autostart" => config.autostart = value == "true" || value == "yes" || value == "1",
                    "delay" => config.delay = value.parse().ok(),
                    "protocol" => config.protocol = Some(value.to_string()),
                    "icon" => config.icon = Some(value.to_string()),
                    key if key.starts_with("env.") => {
                        let env_key = key.strip_prefix("env.").unwrap();
                        config.env_vars.insert(env_key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }
        
        if config.exec_path.is_empty() {
            return Err(StarterError::InvalidConfig(
                "Missing exec_path".to_string()
            ));
        }
        
        Ok(config)
    }
    
    /// Start application with configuration
    pub fn start(&mut self, app_name: &str) -> Result<Child, StarterError> {
        let config = self.load_config(app_name)?;
        self.start_with_config(&config)
    }
    
    /// Start application with specific configuration
    pub fn start_with_config(&self, config: &StarterConfig) -> Result<Child, StarterError> {
        // Apply delay if specified
        if let Some(delay) = config.delay {
            std::thread::sleep(std::time::Duration::from_millis(delay as u64));
        }
        
        // Build command
        let mut cmd = Command::new(&config.exec_path);
        
        // Add arguments
        if !config.args.is_empty() {
            cmd.args(&config.args);
        }
        
        // Set working directory
        if let Some(ref workdir) = config.working_dir {
            cmd.current_dir(workdir);
        }
        
        // Set environment variables from WSDG
        for (key, value) in self.env.all_vars() {
            cmd.env(key, value);
        }
        
        // Set custom environment variables from config
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }
        
        // Spawn process
        cmd.spawn()
            .map_err(|e| StarterError::StartFailed(
                format!("{}: {}", config.exec_path, e)
            ))
    }
    
    /// Start all autostart applications
    pub fn start_autostart_apps(&mut self) -> Vec<Result<Child, StarterError>> {
        let mut results = Vec::new();
        
        // Scan all config directories
        for dir in &self.config_dirs.clone() {
            if !dir.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    if path.extension().and_then(|e| e.to_str()) == Some("config") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            // Try to load and check autostart
                            if let Ok(config) = self.load_config(stem) {
                                if config.autostart {
                                    results.push(self.start_with_config(&config));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Create a new starter configuration
    pub fn create_config(
        &mut self,
        app_name: &str,
        exec_path: &str,
    ) -> Result<StarterConfig, StarterError> {
        let config = StarterConfig {
            app_name: app_name.to_string(),
            exec_path: exec_path.to_string(),
            ..Default::default()
        };
        
        self.configs.insert(app_name.to_string(), config.clone());
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_config(&self, config: &StarterConfig) -> Result<(), StarterError> {
        // Use first config directory (typically user config)
        let config_dir = self.config_dirs.first()
            .ok_or_else(|| StarterError::InvalidConfig("No config directory".to_string()))?;
        
        // Create directory if it doesn't exist
        fs::create_dir_all(config_dir)?;
        
        let filename = format!("{}starter.config", config.app_name);
        let config_path = config_dir.join(filename);
        
        let mut content = String::new();
        content.push_str(&format!("*// WSDG Starter configuration for {}\n\n", config.app_name));
        content.push_str(&format!("exec_path = \"{}\"\n", config.exec_path));
        
        if !config.args.is_empty() {
            content.push_str(&format!("args = \"{}\"\n", config.args.join(" ")));
        }
        
        if let Some(ref workdir) = config.working_dir {
            content.push_str(&format!("working_dir = \"{}\"\n", workdir.display()));
        }
        
        content.push_str(&format!("autostart = {}\n", config.autostart));
        
        if let Some(delay) = config.delay {
            content.push_str(&format!("delay = {}\n", delay));
        }
        
        if let Some(ref protocol) = config.protocol {
            content.push_str(&format!("protocol = \"{}\"\n", protocol));
        }
        
        if let Some(ref icon) = config.icon {
            content.push_str(&format!("icon = \"{}\"\n", icon));
        }
        
        for (key, value) in &config.env_vars {
            content.push_str(&format!("env.{} = \"{}\"\n", key, value));
        }
        
        fs::write(config_path, content)?;
        Ok(())
    }
    
    /// List all available starter configurations
    pub fn list_configs(&self) -> Vec<String> {
        let mut configs = Vec::new();
        
        for dir in &self.config_dirs {
            if !dir.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    if path.extension().and_then(|e| e.to_str()) == Some("config") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            configs.push(stem.to_string());
                        }
                    }
                }
            }
        }
        
        configs.sort();
        configs.dedup();
        configs
    }
    
    /// Check if app has protocol support
    pub fn supports_protocol(&mut self, app_name: &str, protocol: &str) -> bool {
        if let Ok(config) = self.load_config(app_name) {
            if let Some(ref proto) = config.protocol {
                return proto == protocol || proto.contains(protocol);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wsdg_env::WsdgEnvBuilder;
    
    #[test]
    fn test_starter_config_creation() {
        let env = WsdgEnvBuilder::new().build();
        let mut starter = WsdgStarter::new(env);
        
        let config = starter.create_config("testapp", "/usr/bin/testapp").unwrap();
        assert_eq!(config.app_name, "testapp");
        assert_eq!(config.exec_path, "/usr/bin/testapp");
    }
    
    #[test]
    fn test_config_filename_generation() {
        let test_cases = vec![
            ("app", "appstarter.config"),
            ("appstarter", "appstarter.config"),
            ("appstarter.config", "appstarter.config"),
        ];
        
        for (input, expected) in test_cases {
            let filename = if input.ends_with("starter.config") {
                input.to_string()
            } else if input.ends_with("starter") {
                format!("{}.config", input)
            } else {
                format!("{}starter.config", input)
            };
            
            assert_eq!(filename, expected);
        }
    }
}

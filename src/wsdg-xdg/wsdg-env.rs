// WSDG Environment Manager
// Interprets and manages WSDG environment
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Environment variable not found: {0}")]
    VarNotFound(String),
    
    #[error("Invalid environment value: {0}")]
    InvalidValue(String),
    
    #[error("Translation error: {0}")]
    TranslationError(String),
}

/// WSDG Environment Manager
pub struct WsdgEnv {
    /// Environment variables managed by WSDG
    vars: HashMap<String, String>,
    
    /// System environment variables
    system_vars: HashMap<String, String>,
    
    /// Whether to use system fallback
    use_system_fallback: bool,
}

impl WsdgEnv {
    /// Create new WSDG environment
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            system_vars: Self::load_system_vars(),
            use_system_fallback: true,
        }
    }
    
    /// Load system environment variables
    fn load_system_vars() -> HashMap<String, String> {
        env::vars().collect()
    }
    
    /// Set WSDG environment variable
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }
    
    /// Get WSDG environment variable
    pub fn get(&self, key: &str) -> Option<&String> {
        // Check WSDG vars first
        if let Some(value) = self.vars.get(key) {
            return Some(value);
        }
        
        // Fallback to system if enabled
        if self.use_system_fallback {
            self.system_vars.get(key)
        } else {
            None
        }
    }
    
    /// Get or error if not found
    pub fn get_required(&self, key: &str) -> Result<&String, EnvError> {
        self.get(key).ok_or_else(|| EnvError::VarNotFound(key.to_string()))
    }
    
    /// Set multiple variables from config
    pub fn set_from_config(&mut self, config: &HashMap<String, String>) {
        for (key, value) in config {
            self.set(key, value);
        }
    }
    
    /// Get HOME directory
    pub fn home_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(home) = self.get("HOME") {
            Ok(PathBuf::from(home))
        } else {
            dirs::home_dir().ok_or_else(|| EnvError::VarNotFound("HOME".to_string()))
        }
    }
    
    /// Get CONFIG directory
    pub fn config_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(config) = self.get("CONFIG") {
            Ok(PathBuf::from(config))
        } else if let Some(xdg_config) = self.get("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(xdg_config))
        } else {
            self.home_dir().map(|h| h.join(".config"))
        }
    }
    
    /// Get LOCAL directory
    pub fn local_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(local) = self.get("LOCAL") {
            Ok(PathBuf::from(local))
        } else {
            self.home_dir().map(|h| h.join(".local"))
        }
    }
    
    /// Get SHARE directory
    pub fn share_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(share) = self.get("SHARE") {
            Ok(PathBuf::from(share))
        } else if let Some(xdg_data) = self.get("XDG_DATA_HOME") {
            Ok(PathBuf::from(xdg_data))
        } else {
            self.local_dir().map(|l| l.join("share"))
        }
    }
    
    /// Get CACHE directory
    pub fn cache_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(cache) = self.get("CACHE") {
            Ok(PathBuf::from(cache))
        } else if let Some(xdg_cache) = self.get("XDG_CACHE_HOME") {
            Ok(PathBuf::from(xdg_cache))
        } else {
            self.home_dir().map(|h| h.join(".cache"))
        }
    }
    
    /// Get RUNTIME directory
    pub fn runtime_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(runtime) = self.get("RUNTIME") {
            Ok(PathBuf::from(runtime))
        } else if let Some(xdg_runtime) = self.get("XDG_RUNTIME_DIR") {
            Ok(PathBuf::from(xdg_runtime))
        } else {
            // Fallback to /run/user/$UID
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Ok(home) = self.home_dir() {
                    if let Ok(metadata) = std::fs::metadata(&home) {
                        let uid = metadata.uid();
                        return Ok(PathBuf::from(format!("/run/user/{}", uid)));
                    }
                }
            }
            Ok(PathBuf::from("/tmp"))
        }
    }
    
    /// Get STATE directory
    pub fn state_dir(&self) -> Result<PathBuf, EnvError> {
        if let Some(state) = self.get("STATE") {
            Ok(PathBuf::from(state))
        } else if let Some(xdg_state) = self.get("XDG_STATE_HOME") {
            Ok(PathBuf::from(xdg_state))
        } else {
            self.local_dir().map(|l| l.join("state"))
        }
    }
    
    /// Get current user
    pub fn user(&self) -> Result<String, EnvError> {
        if let Some(user) = self.get("USER").or_else(|| self.get("USE_STD")) {
            Ok(user.clone())
        } else {
            env::var("USER")
                .or_else(|_| env::var("USERNAME"))
                .map_err(|_| EnvError::VarNotFound("USER".to_string()))
        }
    }
    
    /// Get user ID (Unix only)
    #[cfg(unix)]
    pub fn uid(&self) -> Result<u32, EnvError> {
        if let Some(uid_str) = self.get("UID") {
            uid_str.parse().map_err(|_| EnvError::InvalidValue("UID".to_string()))
        } else {
            use std::os::unix::fs::MetadataExt;
            if let Ok(home) = self.home_dir() {
                if let Ok(metadata) = std::fs::metadata(&home) {
                    return Ok(metadata.uid());
                }
            }
            Ok(1000) // Default fallback
        }
    }
    
    #[cfg(not(unix))]
    pub fn uid(&self) -> Result<u32, EnvError> {
        Ok(1000) // Default on non-Unix
    }
    
    /// Enable/disable system fallback
    pub fn set_system_fallback(&mut self, enabled: bool) {
        self.use_system_fallback = enabled;
    }
    
    /// Get all WSDG variables
    pub fn all_vars(&self) -> &HashMap<String, String> {
        &self.vars
    }
    
    /// Export to system environment (Unix shells)
    pub fn export_to_shell(&self, shell: &str) -> Vec<String> {
        let mut exports = Vec::new();
        
        for (key, value) in &self.vars {
            let export_line = match shell {
                "fish" => format!("set -gx {} \"{}\"", key, value),
                _ => format!("export {}=\"{}\"", key, value), // bash/zsh
            };
            exports.push(export_line);
        }
        
        exports
    }
    
    /// Merge with XDG translator config
    pub fn merge_from_translator(&mut self, translator: &crate::xdg_wsdg_translate::XdgWsdgTranslator) {
        // This would integrate with the translator
        // Implementation depends on translator API
    }
}

impl Default for WsdgEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// WSDG Environment Builder
pub struct WsdgEnvBuilder {
    env: WsdgEnv,
}

impl WsdgEnvBuilder {
    pub fn new() -> Self {
        Self {
            env: WsdgEnv::new(),
        }
    }
    
    pub fn home(mut self, path: impl Into<String>) -> Self {
        self.env.set("HOME", path);
        self
    }
    
    pub fn config(mut self, path: impl Into<String>) -> Self {
        self.env.set("CONFIG", path);
        self
    }
    
    pub fn local(mut self, path: impl Into<String>) -> Self {
        self.env.set("LOCAL", path);
        self
    }
    
    pub fn share(mut self, path: impl Into<String>) -> Self {
        self.env.set("SHARE", path);
        self
    }
    
    pub fn cache(mut self, path: impl Into<String>) -> Self {
        self.env.set("CACHE", path);
        self
    }
    
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.env.set("USER", user);
        self
    }
    
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.set(key, value);
        self
    }
    
    pub fn system_fallback(mut self, enabled: bool) -> Self {
        self.env.set_system_fallback(enabled);
        self
    }
    
    pub fn build(self) -> WsdgEnv {
        self.env
    }
}

impl Default for WsdgEnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wsdg_env_basic() {
        let mut env = WsdgEnv::new();
        env.set("TEST_VAR", "test_value");
        
        assert_eq!(env.get("TEST_VAR"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_directory_resolution() {
        let env = WsdgEnvBuilder::new()
            .home("/home/testuser")
            .config("/home/testuser/.config")
            .build();
        
        assert_eq!(env.home_dir().unwrap(), PathBuf::from("/home/testuser"));
        assert_eq!(env.config_dir().unwrap(), PathBuf::from("/home/testuser/.config"));
    }
    
    #[test]
    fn test_fallback_directories() {
        let env = WsdgEnvBuilder::new()
            .home("/home/testuser")
            .build();
        
        // Should fallback to $HOME/.config
        assert_eq!(env.config_dir().unwrap(), PathBuf::from("/home/testuser/.config"));
        
        // Should fallback to $HOME/.local
        assert_eq!(env.local_dir().unwrap(), PathBuf::from("/home/testuser/.local"));
    }
    
    #[test]
    fn test_shell_export() {
        let env = WsdgEnvBuilder::new()
            .home("/home/testuser")
            .var("TEST", "value")
            .build();
        
        let bash_exports = env.export_to_shell("bash");
        assert!(bash_exports.iter().any(|e| e.contains("export TEST=")));
        
        let fish_exports = env.export_to_shell("fish");
        assert!(fish_exports.iter().any(|e| e.contains("set -gx TEST")));
    }
}

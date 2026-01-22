// XDG-WSDG Translation Layer
// Translates XDG environment calls to WSDG interface logic
// Fed directly from env.path configuration
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranslateError {
    #[error("Failed to read env.path config: {0}")]
    ConfigReadError(String),
    
    #[error("Parse error at line {line}: {reason}")]
    ParseError { line: usize, reason: String },
    
    #[error("Unknown XDG path: {0}")]
    UnknownXdgPath(String),
    
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    
    #[error("Invalid shell standard: {0}")]
    InvalidShellStd(String),
    
    #[error("Path resolution failed: {0}")]
    ResolutionFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellStandard {
    Bash,
    Zsh,
    Fish,
    Sh,
}

impl ShellStandard {
    pub fn from_str(s: &str) -> Result<Self, TranslateError> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "sh" => Ok(Self::Sh),
            "bash/zsh" => Ok(Self::Bash), // Default to bash
            _ => Err(TranslateError::InvalidShellStd(s)),
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::Sh => "sh",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub shell_std: ShellStandard,
    pub clamp_use: u8,
    pub std_exports: HashMap<String, String>,
    pub xdg_paths: HashMap<String, String>,
    pub overrides: HashMap<String, String>,
    pub shell_variables: Vec<String>,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            shell_std: ShellStandard::Bash,
            clamp_use: 2,
            std_exports: HashMap::new(),
            xdg_paths: HashMap::new(),
            overrides: HashMap::new(),
            shell_variables: Vec::new(),
        }
    }
}

pub struct EnvPathParser {
    config_path: PathBuf,
}

impl EnvPathParser {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
    
    pub fn from_default() -> Result<Self, TranslateError> {
        // Try to find env.path in standard locations
        let mut possible_paths = vec![
            PathBuf::from("/etc/wsdg/env.path"),
            PathBuf::from("/usr/share/wsdg/env.path"),
        ];
        
        if let Some(config_dir) = dirs::config_dir() {
            possible_paths.push(config_dir.join("wsdg/env.path"));
        }
        
        for path in possible_paths {
            if path.exists() {
                return Ok(Self::new(path));
            }
        }
        
        Err(TranslateError::ConfigReadError(
            "env.path not found in standard locations".to_string()
        ))
    }
    
    pub fn load(&self) -> Result<EnvConfig, TranslateError> {
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| TranslateError::ConfigReadError(e.to_string()))?;
        
        self.parse(&content)
    }
    
    pub fn parse(&self, content: &str) -> Result<EnvConfig, TranslateError> {
        let mut config = EnvConfig::default();
        let mut in_env_block = false;
        
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("*//") || line.starts_with("#") {
                continue;
            }
            
            // Check for env block
            if line.starts_with("env_to_$SHELL") {
                in_env_block = true;
                continue;
            }
            
            if line == "}" && in_env_block {
                in_env_block = false;
                continue;
            }
            
            // Parse shell standard
            if line.starts_with("use_shell_std:") {
                let value = line.split(':').nth(1)
                    .ok_or(TranslateError::ParseError {
                        line: line_num + 1,
                        reason: "Missing shell standard value".to_string(),
                    })?;
                config.shell_std = ShellStandard::from_str(value)?;
            }
            
            // Parse clamp setting
            else if line.starts_with("clamp_use_to") {
                let value = line.split(':').nth(1)
                    .and_then(|s| s.trim().parse::<u8>().ok())
                    .ok_or(TranslateError::ParseError {
                        line: line_num + 1,
                        reason: "Invalid clamp_use_to value".to_string(),
                    })?;
                config.clamp_use = value;
            }
            
            // Parse use_std export
            else if line.starts_with("use_std export:") {
                let (var, value) = self.parse_export(line, line_num)?;
                
                // Check for override clause
                if line.contains("over_clay_dg") {
                    let override_part = line.split("over_clay_dg").nth(1)
                        .ok_or(TranslateError::ParseError {
                            line: line_num + 1,
                            reason: "Invalid override syntax".to_string(),
                        })?;
                    
                    if let Some((xdg_var, wsdg_var)) = self.parse_override_clause(override_part) {
                        config.overrides.insert(xdg_var, wsdg_var);
                    }
                }
                
                config.std_exports.insert(var, value);
            }
            
            // Parse direct XDG variables
            else if line.starts_with("XDG_") && in_env_block {
                let (var, value) = self.parse_xdg_direct(line, line_num)?;
                config.xdg_paths.insert(var, value);
            }
            
            // Parse shell variables export
            else if line.contains("setenv $SHELL_VARIABLES") {
                if let Some(vars) = line.split("setenv").nth(1) {
                    config.shell_variables.push(vars.trim().to_string());
                }
            }
        }
        
        Ok(config)
    }
    
    fn parse_export(&self, line: &str, line_num: usize) -> Result<(String, String), TranslateError> {
        // Parse: use_std export:$HOME = /home/array
        let after_export = line.split("export:").nth(1)
            .ok_or(TranslateError::ParseError {
                line: line_num + 1,
                reason: "Invalid export syntax".to_string(),
            })?;
        
        // Remove override part if exists
        let main_part = after_export.split("*//").next()
            .and_then(|s| s.split("->").next())
            .unwrap_or(after_export);
        
        let parts: Vec<&str> = main_part.split('=').collect();
        if parts.len() >= 2 {
            let var = parts[0].trim().trim_start_matches('$');
            let value = parts[1].trim().trim_matches('"');
            Ok((var.to_string(), value.to_string()))
        } else {
            Err(TranslateError::ParseError {
                line: line_num + 1,
                reason: "Invalid export format".to_string(),
            })
        }
    }
    
    fn parse_xdg_direct(&self, line: &str, line_num: usize) -> Result<(String, String), TranslateError> {
        // Parse: XDG_CONFIG_HOME="home/.config"
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() >= 2 {
            let var = parts[0].trim();
            let value = parts[1].trim()
                .trim_matches('"')
                .split("*//").next()
                .unwrap_or("")
                .trim();
            Ok((var.to_string(), value.to_string()))
        } else {
            Err(TranslateError::ParseError {
                line: line_num + 1,
                reason: "Invalid XDG variable format".to_string(),
            })
        }
    }
    
    fn parse_override_clause(&self, clause: &str) -> Option<(String, String)> {
        // Parse: *XDG = XDG_CONFIG_HOME
        let parts: Vec<&str> = clause.split('=').collect();
        if parts.len() >= 2 {
            let xdg_var = parts[1].trim().to_string();
            let wsdg_var = parts[0].trim().trim_start_matches('*').to_string();
            Some((xdg_var, wsdg_var))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct XdgWsdgTranslator {
    config: EnvConfig,
    cache: HashMap<String, PathBuf>,
}

impl XdgWsdgTranslator {
    pub fn new(config: EnvConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }
    
    pub fn from_env_path(config_path: PathBuf) -> Result<Self, TranslateError> {
        let parser = EnvPathParser::new(config_path);
        let config = parser.load()?;
        Ok(Self::new(config))
    }
    
    pub fn from_default() -> Result<Self, TranslateError> {
        let parser = EnvPathParser::from_default()?;
        let config = parser.load()?;
        Ok(Self::new(config))
    }
    
    /// Translate XDG path to WSDG resolved path
    pub fn translate_xdg(&mut self, xdg_var: &str) -> Result<PathBuf, TranslateError> {
        // Check cache first
        if let Some(cached) = self.cache.get(xdg_var) {
            return Ok(cached.clone());
        }
        
        // 1. Check for overrides first
        if let Some(override_var) = self.config.overrides.get(xdg_var) {
            let resolved = self.resolve_wsdg_var(override_var)?;
            self.cache.insert(xdg_var.to_string(), resolved.clone());
            return Ok(resolved);
        }
        
        // 2. Check if direct XDG path is defined
        if let Some(xdg_path) = self.config.xdg_paths.get(xdg_var) {
            let resolved = self.expand_path(xdg_path)?;
            self.cache.insert(xdg_var.to_string(), resolved.clone());
            return Ok(resolved);
        }
        
        // 3. Use standard XDG → WSDG mapping
        let wsdg_var = self.xdg_to_wsdg_standard(xdg_var)?;
        let resolved = self.resolve_wsdg_var(&wsdg_var)?;
        self.cache.insert(xdg_var.to_string(), resolved.clone());
        Ok(resolved)
    }
    
    /// Standard XDG to WSDG variable mapping
    fn xdg_to_wsdg_standard(&self, xdg_var: &str) -> Result<String, TranslateError> {
        let wsdg = match xdg_var {
            "XDG_CONFIG_HOME" => "$CONFIG",
            "XDG_DATA_HOME" => "$SHARE",
            "XDG_CACHE_HOME" => "$CACHE",
            "XDG_RUNTIME_DIR" => "$RUNTIME",
            "XDG_STATE_HOME" => "$STATE",
            _ => return Err(TranslateError::UnknownXdgPath(xdg_var.to_string())),
        };
        Ok(wsdg.to_string())
    }
    
    /// Resolve WSDG variable like $HOME, $CONFIG, etc.
    pub fn resolve_wsdg_var(&self, var: &str) -> Result<PathBuf, TranslateError> {
        let var_name = var.trim_start_matches('$');
        
        // Check in std_exports first
        if let Some(path) = self.config.std_exports.get(var_name) {
            return self.expand_path(path);
        }
        
        // Fallback to system environment
        match var_name {
            "HOME" => {
                env::var("HOME")
                    .or_else(|_| dirs::home_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .ok_or(env::VarError::NotPresent))
                    .map(PathBuf::from)
                    .map_err(|_| TranslateError::VariableNotFound(var.to_string()))
            }
            "USER" | "USE_STD" => {
                env::var("USER")
                    .or_else(|_| env::var("USERNAME"))
                    .map(PathBuf::from)
                    .map_err(|_| TranslateError::VariableNotFound(var.to_string()))
            }
            _ => Err(TranslateError::VariableNotFound(var.to_string())),
        }
    }
    
    /// Expand path with variable substitution
    pub fn expand_path(&self, path: &str) -> Result<PathBuf, TranslateError> {
        let mut result = String::new();
        let mut current_var = String::new();
        let mut in_var = false;
        
        for ch in path.chars() {
            if ch == '$' {
                if in_var && !current_var.is_empty() {
                    // Resolve previous variable
                    let resolved = self.resolve_single_var(&current_var)?;
                    result.push_str(&resolved);
                    current_var.clear();
                }
                in_var = true;
            } else if (ch == '/' || ch == '\\') && in_var {
                // End of variable
                if !current_var.is_empty() {
                    let resolved = self.resolve_single_var(&current_var)?;
                    result.push_str(&resolved);
                    current_var.clear();
                }
                result.push(ch);
                in_var = false;
            } else if in_var {
                current_var.push(ch);
            } else {
                result.push(ch);
            }
        }
        
        // Handle trailing variable
        if !current_var.is_empty() {
            let resolved = self.resolve_single_var(&current_var)?;
            result.push_str(&resolved);
        }
        
        // Clean path
        let cleaned = result.trim_start_matches('/').to_string();
        Ok(PathBuf::from(if result.starts_with('/') {
            format!("/{}", cleaned)
        } else {
            result
        }))
    }
    
    fn resolve_single_var(&self, var: &str) -> Result<String, TranslateError> {
        // Check std_exports
        if let Some(value) = self.config.std_exports.get(var) {
            return Ok(value.clone());
        }
        
        // System variables
        match var {
            "HOME" => {
                env::var("HOME")
                    .or_else(|_| dirs::home_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .ok_or(env::VarError::NotPresent))
                    .map_err(|_| TranslateError::VariableNotFound(var.to_string()))
            }
            "USER" | "USE_STD" => {
                env::var("USER")
                    .or_else(|_| env::var("USERNAME"))
                    .map_err(|_| TranslateError::VariableNotFound(var.to_string()))
            }
            "UID" => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if let Some(home) = dirs::home_dir() {
                        if let Ok(metadata) = fs::metadata(&home) {
                            return Ok(metadata.uid().to_string());
                        }
                    }
                    Ok("1000".to_string())
                }
                #[cfg(not(unix))]
                {
                    Ok("1000".to_string())
                }
            }
            _ => Err(TranslateError::VariableNotFound(var.to_string())),
        }
    }
    
    /// Get shell standard
    pub fn shell_standard(&self) -> ShellStandard {
        self.config.shell_std
    }
    
    /// Get clamp use setting
    pub fn clamp_use(&self) -> u8 {
        self.config.clamp_use
    }
    
    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_env_config() {
        let content = r#"
use_shell_std: bash/zsh
clamp_use_to : 2

env_to_$SHELL {
    use_std export:$HOME = /home/testuser
    use_std export:$CONFIG = /home/.config
    
    XDG_CONFIG_HOME="/home/.config"
    XDG_DATA_HOME=/home/.local/share
}
        "#;
        
        let parser = EnvPathParser::new(PathBuf::from("test"));
        let config = parser.parse(content).unwrap();
        
        assert_eq!(config.shell_std, ShellStandard::Bash);
        assert_eq!(config.clamp_use, 2);
        assert_eq!(config.std_exports.get("HOME"), Some(&"/home/testuser".to_string()));
        assert_eq!(config.xdg_paths.get("XDG_CONFIG_HOME"), Some(&"/home/.config".to_string()));
    }
    
    #[test]
    fn test_variable_expansion() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("HOME".to_string(), "/home/user".to_string());
        config.std_exports.insert("CONFIG".to_string(), ".config".to_string());
        
        let translator = XdgWsdgTranslator::new(config);
        
        let expanded = translator.expand_path("$HOME/$CONFIG").unwrap();
        assert_eq!(expanded, PathBuf::from("/home/user/.config"));
    }
    
    #[test]
    fn test_xdg_translation() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("CONFIG".to_string(), "/home/user/.config".to_string());
        
        let mut translator = XdgWsdgTranslator::new(config);
        
        let path = translator.translate_xdg("XDG_CONFIG_HOME").unwrap();
        assert!(path.to_string_lossy().contains("config"));
    }
    
    #[test]
    fn test_override_mechanism() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("XDG".to_string(), "/custom/xdg".to_string());
        config.overrides.insert("XDG_CONFIG_HOME".to_string(), "XDG".to_string());
        
        let mut translator = XdgWsdgTranslator::new(config);
        
        let path = translator.translate_xdg("XDG_CONFIG_HOME").unwrap();
        assert_eq!(path, PathBuf::from("/custom/xdg"));
    }
}

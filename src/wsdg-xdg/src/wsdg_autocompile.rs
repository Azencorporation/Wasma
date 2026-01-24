// WSDG AutoCompile - Translation Layer Auto Compiler
// Provides autocompiled support for XDG-WSDG translation layer
// Used in buffer before XDG-WSDG translation or needed for application environments/interfaces
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use thiserror::Error;

use crate::xdg_wsdg_translate::{XdgWsdgTranslator, EnvConfig, ShellStandard};

#[derive(Debug, Error)]
pub enum AutoCompileError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
    
    #[error("Invalid buffer state: {0}")]
    InvalidBuffer(String),
    
    #[error("Translation error: {0}")]
    TranslationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Compilation buffer for XDG-WSDG translation
#[derive(Debug, Clone)]
pub struct CompilationBuffer {
    pub xdg_paths: HashMap<String, String>,
    pub wsdg_paths: HashMap<String, String>,
    pub env_mappings: HashMap<String, String>,
    pub shell_exports: Vec<String>,
}

impl Default for CompilationBuffer {
    fn default() -> Self {
        Self {
            xdg_paths: HashMap::new(),
            wsdg_paths: HashMap::new(),
            env_mappings: HashMap::new(),
            shell_exports: Vec::new(),
        }
    }
}

/// Auto compilation mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompileMode {
    /// Just-in-time: compile when needed
    JIT,
    /// Ahead-of-time: pre-compile all translations
    AOT,
    /// Hybrid: compile common paths ahead, others on-demand
    Hybrid,
}

/// WSDG Auto Compiler - Compiles XDG to WSDG translations
pub struct WsdgAutoCompiler {
    translator: XdgWsdgTranslator,
    buffer: CompilationBuffer,
    mode: CompileMode,
    cache_dir: PathBuf,
    compiled: bool,
}

impl WsdgAutoCompiler {
    pub fn new(translator: XdgWsdgTranslator, mode: CompileMode) -> Self {
        let cache_dir = Self::get_cache_dir();
        
        Self {
            translator,
            buffer: CompilationBuffer::default(),
            mode,
            cache_dir,
            compiled: false,
        }
    }
    
    /// Get compilation cache directory
    fn get_cache_dir() -> PathBuf {
        if let Some(cache) = dirs::cache_dir() {
            cache.join("wsdg/autocompile")
        } else {
            PathBuf::from("/tmp/wsdg/autocompile")
        }
    }
    
    /// Compile XDG environment to WSDG buffer
    pub fn compile(&mut self) -> Result<(), AutoCompileError> {
        match self.mode {
            CompileMode::JIT => {
                // Lazy compilation - just mark as ready
                self.compiled = true;
                Ok(())
            }
            CompileMode::AOT => {
                // Pre-compile all standard paths
                self.compile_all_standard_paths()?;
                self.compiled = true;
                Ok(())
            }
            CompileMode::Hybrid => {
                // Compile common paths
                self.compile_common_paths()?;
                self.compiled = true;
                Ok(())
            }
        }
    }
    
    /// Compile all standard XDG paths
    fn compile_all_standard_paths(&mut self) -> Result<(), AutoCompileError> {
        let standard_xdg = vec![
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_RUNTIME_DIR",
            "XDG_STATE_HOME",
        ];
        
        for xdg_var in standard_xdg {
            if let Ok(wsdg_path) = self.translator.translate_xdg(xdg_var) {
                self.buffer.xdg_paths.insert(
                    xdg_var.to_string(),
                    wsdg_path.to_string_lossy().to_string()
                );
            }
        }
        
        // Compile shell exports
        self.compile_shell_exports()?;
        
        Ok(())
    }
    
    /// Compile common paths (for hybrid mode)
    fn compile_common_paths(&mut self) -> Result<(), AutoCompileError> {
        let common = vec![
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
        ];
        
        for xdg_var in common {
            if let Ok(wsdg_path) = self.translator.translate_xdg(xdg_var) {
                self.buffer.xdg_paths.insert(
                    xdg_var.to_string(),
                    wsdg_path.to_string_lossy().to_string()
                );
            }
        }
        
        Ok(())
    }
    
    /// Compile shell export statements
    fn compile_shell_exports(&mut self) -> Result<(), AutoCompileError> {
        let shell_std = self.translator.shell_standard();
        
        for (xdg_var, wsdg_path) in &self.buffer.xdg_paths {
            let export = match shell_std {
                ShellStandard::Fish => {
                    format!("set -gx {} \"{}\"", xdg_var, wsdg_path)
                }
                _ => {
                    format!("export {}=\"{}\"", xdg_var, wsdg_path)
                }
            };
            
            self.buffer.shell_exports.push(export);
        }
        
        Ok(())
    }
    
    /// Get compiled path from buffer
    pub fn get_compiled_path(&mut self, xdg_var: &str) -> Result<PathBuf, AutoCompileError> {
        // Check buffer first
        if let Some(path) = self.buffer.xdg_paths.get(xdg_var) {
            return Ok(PathBuf::from(path));
        }
        
        // If JIT mode, compile on demand
        if self.mode == CompileMode::JIT || self.mode == CompileMode::Hybrid {
            let wsdg_path = self.translator.translate_xdg(xdg_var)
                .map_err(|e| AutoCompileError::TranslationError(e.to_string()))?;
            
            self.buffer.xdg_paths.insert(
                xdg_var.to_string(),
                wsdg_path.to_string_lossy().to_string()
            );
            
            Ok(wsdg_path)
        } else {
            Err(AutoCompileError::InvalidBuffer(
                format!("Path not compiled: {}", xdg_var)
            ))
        }
    }
    
    /// Save compiled buffer to cache
    pub fn save_cache(&self) -> Result<(), AutoCompileError> {
        if !self.compiled {
            return Err(AutoCompileError::InvalidBuffer(
                "Nothing compiled yet".to_string()
            ));
        }
        
        // Create cache directory
        fs::create_dir_all(&self.cache_dir)?;
        
        let cache_file = self.cache_dir.join("compiled.cache");
        
        // Serialize buffer
        let mut content = String::new();
        content.push_str("*// WSDG AutoCompile Cache\n\n");
        
        content.push_str("[xdg_paths]\n");
        for (xdg, wsdg) in &self.buffer.xdg_paths {
            content.push_str(&format!("{} = \"{}\"\n", xdg, wsdg));
        }
        
        content.push_str("\n[shell_exports]\n");
        for export in &self.buffer.shell_exports {
            content.push_str(&format!("{}\n", export));
        }
        
        fs::write(cache_file, content)?;
        Ok(())
    }
    
    /// Load compiled buffer from cache
    pub fn load_cache(&mut self) -> Result<(), AutoCompileError> {
        let cache_file = self.cache_dir.join("compiled.cache");
        
        if !cache_file.exists() {
            return Err(AutoCompileError::InvalidBuffer(
                "No cache file found".to_string()
            ));
        }
        
        let content = fs::read_to_string(cache_file)?;
        
        let mut current_section = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with("*//") {
                continue;
            }
            
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line.trim_matches(|c| c == '[' || c == ']').to_string();
                continue;
            }
            
            match current_section.as_str() {
                "xdg_paths" => {
                    if let Some((key, value)) = line.split_once('=') {
                        let value = value.trim().trim_matches('"');
                        self.buffer.xdg_paths.insert(
                            key.trim().to_string(),
                            value.to_string()
                        );
                    }
                }
                "shell_exports" => {
                    self.buffer.shell_exports.push(line.to_string());
                }
                _ => {}
            }
        }
        
        self.compiled = true;
        Ok(())
    }
    
    /// Generate environment setup script
    pub fn generate_env_script(&self, shell: ShellStandard) -> String {
        let mut script = String::new();
        
        match shell {
            ShellStandard::Bash | ShellStandard::Zsh | ShellStandard::Sh => {
                script.push_str("#!/bin/bash\n");
                script.push_str("# WSDG Environment Setup (Auto-generated)\n\n");
                
                for (xdg, wsdg) in &self.buffer.xdg_paths {
                    script.push_str(&format!("export {}=\"{}\"\n", xdg, wsdg));
                }
            }
            ShellStandard::Fish => {
                script.push_str("#!/usr/bin/env fish\n");
                script.push_str("# WSDG Environment Setup (Auto-generated)\n\n");
                
                for (xdg, wsdg) in &self.buffer.xdg_paths {
                    script.push_str(&format!("set -gx {} \"{}\"\n", xdg, wsdg));
                }
            }
        }
        
        script
    }
    
    /// Clear compilation buffer
    pub fn clear_buffer(&mut self) {
        self.buffer = CompilationBuffer::default();
        self.compiled = false;
    }
    
    /// Check if compiled
    pub fn is_compiled(&self) -> bool {
        self.compiled
    }
    
    /// Get buffer reference
    pub fn buffer(&self) -> &CompilationBuffer {
        &self.buffer
    }
    
    /// Get translator reference
    pub fn translator(&self) -> &XdgWsdgTranslator {
        &self.translator
    }
    
    /// Get mutable translator reference
    pub fn translator_mut(&mut self) -> &mut XdgWsdgTranslator {
        &mut self.translator
    }
}

/// Auto-compile helper for easy integration
pub struct AutoCompileHelper;

impl AutoCompileHelper {
    /// Quick compile with default settings
    pub fn quick_compile(env_config: EnvConfig) -> Result<WsdgAutoCompiler, AutoCompileError> {
        let translator = XdgWsdgTranslator::new(env_config);
        let mut compiler = WsdgAutoCompiler::new(translator, CompileMode::Hybrid);
        
        compiler.compile()?;
        Ok(compiler)
    }
    
    /// Try to load from cache, compile if needed
    pub fn load_or_compile(translator: XdgWsdgTranslator) -> Result<WsdgAutoCompiler, AutoCompileError> {
        let mut compiler = WsdgAutoCompiler::new(translator, CompileMode::AOT);
        
        if compiler.load_cache().is_err() {
            compiler.compile()?;
            let _ = compiler.save_cache();
        }
        
        Ok(compiler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xdg_wsdg_translate::EnvConfig;
    
    #[test]
    fn test_autocompiler_creation() {
        let config = EnvConfig::default();
        let translator = XdgWsdgTranslator::new(config);
        let compiler = WsdgAutoCompiler::new(translator, CompileMode::JIT);
        
        assert!(!compiler.is_compiled());
    }
    
    #[test]
    fn test_jit_compilation() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("CONFIG".to_string(), "/home/test/.config".to_string());
        
        let translator = XdgWsdgTranslator::new(config);
        let mut compiler = WsdgAutoCompiler::new(translator, CompileMode::JIT);
        
        compiler.compile().unwrap();
        assert!(compiler.is_compiled());
    }
    
    #[test]
    fn test_aot_compilation() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("CONFIG".to_string(), "/home/test/.config".to_string());
        config.std_exports.insert("SHARE".to_string(), "/home/test/.local/share".to_string());
        
        let translator = XdgWsdgTranslator::new(config);
        let mut compiler = WsdgAutoCompiler::new(translator, CompileMode::AOT);
        
        compiler.compile().unwrap();
        assert!(compiler.is_compiled());
        assert!(!compiler.buffer().xdg_paths.is_empty());
    }
    
    #[test]
    fn test_shell_script_generation() {
        let mut config = EnvConfig::default();
        config.std_exports.insert("HOME".to_string(), "/home/test".to_string());
        
        let translator = XdgWsdgTranslator::new(config);
        let mut compiler = WsdgAutoCompiler::new(translator, CompileMode::Hybrid);
        
        compiler.compile().unwrap();
        
        let bash_script = compiler.generate_env_script(ShellStandard::Bash);
        assert!(bash_script.contains("#!/bin/bash"));
        
        let fish_script = compiler.generate_env_script(ShellStandard::Fish);
        assert!(fish_script.contains("#!/usr/bin/env fish"));
    }
}

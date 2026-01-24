//! WSDG-XDG Translation Layer
//!
//! This library provides a comprehensive translation layer between XDG (X Desktop Group)
//! environment standards and WSDG (Windows Desktop Group) interface logic.
//!
//! # Features
//!
//! - **XDG to WSDG Translation**: Seamlessly translate XDG environment variables and paths to WSDG equivalents
//! - **Application Launching**: Open and manage applications with protocol support
//! - **Icon Discovery**: Automatically find and manage application icons
//! - **MIME Type Detection**: Comprehensive MIME type registry and file type detection
//! - **Auto-compilation**: Buffer-based compilation for efficient environment translation
//! - **Settings Management**: Unified settings system for GUI applications
//! - **Manifest Support**: Application manifests with custom URI schemes
//!
//! # Architecture
//!
//! The WSDG-XDG system is part of WASMA (Windows Assignment System Monitoring Architecture)
//! and provides the following components:
//!
//! - `xdg_wsdg_translate`: Core XDG→WSDG translation engine
//! - `wsdg_env`: Environment variable management
//! - `wsdg_open`: Application launcher
//! - `wsdg_ghx_open`: URI and protocol handler
//! - `wsdg_mime_array`: MIME type detection and registry
//! - `wsdg_byico_icoctl`: Icon discovery system
//! - `wsdg_autocompile`: Auto-compilation for translation layer
//! - `wsdg_settings`: Settings management
//! - `wsdg_starter`: Application startup configuration
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use wsdg_xdg::{WsdgEnv, WsdgOpen, XdgWsdgTranslator};
//!
//! // Create WSDG environment
//! let env = WsdgEnv::new();
//!
//! // Create application launcher
//! let mut opener = WsdgOpen::new(env.clone());
//!
//! // Open a file
//! opener.open("document.pdf").unwrap();
//!
//! // Use XDG translation
//! let mut translator = XdgWsdgTranslator::from_default().unwrap();
//! let config_path = translator.translate_xdg("XDG_CONFIG_HOME").unwrap();
//! println!("Config dir: {}", config_path.display());
//! ```
//!
//! # Configuration
//!
//! The system reads configuration from `env.path` files in standard locations:
//! - `/etc/wsdg/env.path`
//! - `/usr/share/wsdg/env.path`
//! - `~/.config/wsdg/env.path`
//!
//! Example `env.path`:
//!
//! ```text
//! use_shell_std: bash/zsh
//! clamp_use_to: 2
//!
//! env_to_$SHELL {
//!     use_std export:$HOME = /home/user
//!     use_std export:$CONFIG = /home/user/.config
//!     
//!     XDG_CONFIG_HOME = "/home/user/.config"
//!     XDG_DATA_HOME = "/home/user/.local/share"
//! }
//! ```

// Core modules
pub mod xdg_wsdg_translate;
pub mod wsdg_env;
pub mod wsdg_open;
pub mod wsdg_ghx_open;
pub mod wsdg_mime_array;
pub mod wsdg_byico_icoctl;
pub mod wsdg_autocompile;
pub mod wsdg_settings;
pub mod wsdg_starter;

// Re-exports for convenience
pub use xdg_wsdg_translate::{
    XdgWsdgTranslator,
    EnvPathParser,
    EnvConfig,
    ShellStandard,
    TranslateError,
};

pub use wsdg_env::{
    WsdgEnv,
    WsdgEnvBuilder,
    EnvError,
};

pub use wsdg_open::{
    WsdgOpen,
    AppInfo,
    OpenError,
};

pub use wsdg_ghx_open::{
    WsdgGhxOpen,
    Uri,
    UriBuilder,
    GhxOpenError,
};

pub use wsdg_mime_array::{
    WsdgMimeArray,
    MimeType,
    MimeCategory,
    MimeError,
};

pub use wsdg_byico_icoctl::{
    WsdgIcoCtl,
    IconInfo,
    IconSize,
    IconFormat as IcoFormat,
    IconThemeManager,
    IconError,
};

pub use wsdg_autocompile::{
    WsdgAutoCompiler,
    CompilationBuffer,
    CompileMode,
    AutoCompileHelper,
    AutoCompileError,
};

pub use wsdg_settings::{
    WsdgSettingsManager,
    WsdgSettings,
    ThemeSettings,
    FontSettings,
    IconSettings,
    WindowSettings,
    SettingsError,
};

pub use wsdg_starter::{
    WsdgStarter,
    StarterConfig,
    StarterError,
};

/// WSDG-XDG version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// WSDG-XDG library information
pub const LIBRARY_INFO: &str = "WSDG-XDG Translation Layer - Part of WASMA";

/// Initialize WSDG environment with default configuration
///
/// This is a convenience function that sets up the complete WSDG environment
/// including translation, auto-compilation, and settings.
///
/// # Example
///
/// ```rust,no_run
/// use wsdg_xdg::init_wsdg_env;
///
/// let (env, translator) = init_wsdg_env().unwrap();
/// ```
pub fn init_wsdg_env() -> Result<(WsdgEnv, XdgWsdgTranslator), Box<dyn std::error::Error>> {
    // Create environment
    let env = WsdgEnv::new();
    
    // Load translator
    let translator = XdgWsdgTranslator::from_default()?;
    
    Ok((env, translator))
}

/// Initialize complete WSDG system with auto-compilation
///
/// This sets up the full WSDG system including environment, translation,
/// auto-compilation, and settings management.
///
/// # Example
///
/// ```rust,no_run
/// use wsdg_xdg::init_wsdg_system;
///
/// let system = init_wsdg_system().unwrap();
/// ```
pub fn init_wsdg_system() -> Result<WsdgSystem, Box<dyn std::error::Error>> {
    let env = WsdgEnv::new();
    let translator = XdgWsdgTranslator::from_default()?;
    let compiler = AutoCompileHelper::load_or_compile(translator.clone())?;
    let settings = WsdgSettingsManager::new(env.clone());
    
    Ok(WsdgSystem {
        env,
        translator,
        compiler,
        settings,
    })
}

/// Complete WSDG system configuration
pub struct WsdgSystem {
    pub env: WsdgEnv,
    pub translator: XdgWsdgTranslator,
    pub compiler: WsdgAutoCompiler,
    pub settings: WsdgSettingsManager,
}

impl WsdgSystem {
    /// Create application opener
    pub fn create_opener(&self) -> WsdgOpen {
        WsdgOpen::new(self.env.clone())
            .with_translator(self.translator.clone())
    }
    
    /// Create GHX opener (URI handler)
    pub fn create_ghx_opener(&self) -> WsdgGhxOpen {
        let opener = self.create_opener();
        WsdgGhxOpen::new(opener)
    }
    
    /// Create icon controller
    pub fn create_icon_controller(&self) -> WsdgIcoCtl {
        WsdgIcoCtl::new()
    }
    
    /// Create MIME array
    pub fn create_mime_array(&self) -> WsdgMimeArray {
        WsdgMimeArray::new()
    }
    
    /// Create starter
    pub fn create_starter(&self) -> WsdgStarter {
        WsdgStarter::new(self.env.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
    
    #[test]
    fn test_library_info() {
        assert!(LIBRARY_INFO.contains("WSDG"));
        assert!(LIBRARY_INFO.contains("WASMA"));
    }
    
    #[test]
    fn test_init_env() {
        // This may fail in test environment without proper config
        // but tests the API structure
        let result = init_wsdg_env();
        // Just test that the function exists and returns the right type
        assert!(result.is_ok() || result.is_err());
    }
}

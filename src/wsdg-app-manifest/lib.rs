// WASMA - Windows Assignment System Monitoring Architecture
// Core Library Module
// January 15, 2026

pub mod parser;
pub mod manifest;
pub mod source_parser;
pub mod window_handling;
pub mod window_client;
pub mod window_multitary;
pub mod window_singularity;
pub mod protocols;
pub mod uclient;
pub mod wgclient;
pub mod window_resourcer_engineering;

// Re-export commonly used types
pub use parser::{ConfigParser, ParserError, Protocol, ProtocolConfig, WasmaConfig};
pub use manifest::{ManifestParser, ManifestError, WasmaManifest, ResourceConfig as ManifestResourceConfig};
pub use source_parser::{SourceParser, SourceError, PermissionSource};
pub use window_handling::{
    Window, WindowHandler, WindowGeometry, WindowState, WindowType,
    ResourceLimits, PermissionScope, BackendType, ResourceUsage,
    WasmaWindowManager, launch_window_manager, Message,
};
pub use window_client::WindowClient;
pub use window_multitary::{WindowMultitary, Viewport};
pub use window_singularity::{WindowSingularity, SINGULARITY_LOCK};

// WBackend integration
pub use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};

use std::sync::Arc;

/// WASMA Core - Main entry point for the architecture
pub struct WasmaCore {
    pub config: Arc<WasmaConfig>,
    pub window_handler: Arc<WindowHandler>,
    pub resource_mode: ResourceMode,
}

impl WasmaCore {
    /// Create new WASMA Core instance from config file
    pub fn new(config_path: Option<String>) -> Result<Self, ParserError> {
        let parser = ConfigParser::new(config_path);
        let config = parser.load()?;
        
        let resource_mode = if config.resource_limits.scope_level > 0 {
            ResourceMode::Auto
        } else {
            ResourceMode::Manual
        };
        
        let window_handler = Arc::new(WindowHandler::new(resource_mode));
        
        Ok(Self {
            config: Arc::new(config),
            window_handler,
            resource_mode,
        })
    }

    /// Create from existing config
    pub fn from_config(config: WasmaConfig, resource_mode: ResourceMode) -> Self {
        let window_handler = Arc::new(WindowHandler::new(resource_mode));
        
        Self {
            config: Arc::new(config),
            window_handler,
            resource_mode,
        }
    }

    /// Create a new window using config defaults
    pub fn create_window(
        &self,
        title: String,
        app_id: String,
        width: u32,
        height: u32,
    ) -> Result<u64, String> {
        let geometry = WindowGeometry {
            x: 100,
            y: 100,
            width,
            height,
        };
        
        let manifest_path = if !self.config.uri_handling.window_app_spec.is_empty() {
            Some(self.config.uri_handling.window_app_spec.clone())
        } else {
            None
        };
        
        self.window_handler.create_window(
            title,
            app_id,
            geometry,
            manifest_path,
            self.resource_mode,
        )
    }

    /// Create window with manifest file
    pub fn create_window_with_manifest(
        &self,
        manifest_path: String,
    ) -> Result<u64, String> {
        // Parse manifest
        let manifest_parser = ManifestParser::new(manifest_path.clone());
        let manifest = manifest_parser.load()
            .map_err(|e| format!("Failed to load manifest: {}", e))?;
        
        // Create window from manifest
        let geometry = WindowGeometry {
            x: manifest.window.width.map(|w| w as i32).unwrap_or(100),
            y: manifest.window.height.map(|h| h as i32).unwrap_or(100),
            width: manifest.window.width.unwrap_or(800),
            height: manifest.window.height.unwrap_or(600),
        };
        
        self.window_handler.create_window(
            manifest.app.name.clone(),
            format!("wasma.{}", manifest.app.name.to_lowercase()),
            geometry,
            Some(manifest_path),
            self.resource_mode,
        )
    }

    /// Load permissions from manifest
    pub fn load_permissions(&self, manifest_path: &str) -> Result<PermissionSource, String> {
        let source_parser = SourceParser::new(None);
        
        // Try to load embedded source from manifest first
        let manifest_content = std::fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        
        if let Some(embedded) = source_parser.load_embedded(&manifest_content)
            .map_err(|e| format!("Failed to parse embedded source: {}", e))? {
            return Ok(embedded);
        }
        
        // Otherwise, resolve external source file
        let manifest_parser = ManifestParser::new(manifest_path.to_string());
        let manifest = manifest_parser.load()
            .map_err(|e| format!("Failed to load manifest: {}", e))?;
        
        let permission_type = match manifest.permissions.permission_check {
            manifest::PermissionCheckType::PermissionDevel => "permission_devel",
            manifest::PermissionCheckType::PermissionSys => "permission_sys",
            manifest::PermissionCheckType::PermissionPreset => "permission_preset",
            manifest::PermissionCheckType::PermissionPinning => "permission_pinning",
            manifest::PermissionCheckType::PermissionPurning => "permission_purning",
        };
        
        let source_path = source_parser.resolve_source_path(permission_type);
        
        source_parser.load(source_path.to_str().unwrap())
            .map_err(|e| format!("Failed to load source: {}", e))
    }

    /// Create window with custom resource limits
    pub fn create_window_with_limits(
        &self,
        title: String,
        app_id: String,
        width: u32,
        height: u32,
        limits: ResourceLimits,
    ) -> Result<u64, String> {
        let geometry = WindowGeometry {
            x: 100,
            y: 100,
            width,
            height,
        };
        
        let window_id = self.window_handler.create_window(
            title,
            app_id,
            geometry,
            None,
            ResourceMode::Manual,
        )?;
        
        self.window_handler.adjust_window_resources(window_id, limits)?;
        Ok(window_id)
    }

    /// Get window resource usage
    pub fn get_window_resources(&self, window_id: u64) -> Result<ResourceUsage, String> {
        self.window_handler.get_window_resource_usage(window_id)
    }

    /// Run resource management cycle
    pub fn update(&self) {
        self.window_handler.run_resource_cycle();
    }

    /// Close window
    pub fn close_window(&self, window_id: u64) -> Result<(), String> {
        self.window_handler.close_window(window_id)
    }

    /// Get all windows
    pub fn list_windows(&self) -> Vec<Window> {
        self.window_handler.list_windows()
    }

    /// Focus window
    pub fn focus_window(&self, window_id: u64) -> Result<(), String> {
        self.window_handler.focus_window(window_id)
    }

    /// Set window state
    pub fn set_window_state(&self, window_id: u64, state: WindowState) -> Result<(), String> {
        self.window_handler.set_window_state(window_id, state)
    }

    /// Check if multi-instance mode is enabled
    pub fn is_multi_instance(&self) -> bool {
        self.config.uri_handling.multi_instances
    }

    /// Check if singularity mode is enabled
    pub fn is_singularity(&self) -> bool {
        self.config.uri_handling.singularity_instances
    }

    /// Get configured protocols
    pub fn get_protocols(&self) -> &Vec<ProtocolConfig> {
        &self.config.uri_handling.protocols
    }

    /// Launch GUI window manager
    pub fn launch_gui(self) -> iced::Result {
        launch_window_manager(self.resource_mode)
    }
}

/// Builder pattern for WASMA Core
pub struct WasmaCoreBuilder {
    config_path: Option<String>,
    resource_mode: Option<ResourceMode>,
}

impl WasmaCoreBuilder {
    pub fn new() -> Self {
        Self {
            config_path: None,
            resource_mode: None,
        }
    }

    pub fn with_config_path(mut self, path: String) -> Self {
        self.config_path = Some(path);
        self
    }

    pub fn with_resource_mode(mut self, mode: ResourceMode) -> Self {
        self.resource_mode = Some(mode);
        self
    }

    pub fn build(self) -> Result<WasmaCore, ParserError> {
        let parser = ConfigParser::new(self.config_path);
        let config = parser.load()?;
        
        let resource_mode = self.resource_mode.unwrap_or_else(|| {
            if config.resource_limits.scope_level > 0 {
                ResourceMode::Auto
            } else {
                ResourceMode::Manual
            }
        });
        
        Ok(WasmaCore::from_config(config, resource_mode))
    }
}

impl Default for WasmaCoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions
pub mod utils {
    use super::*;

    /// Create default config file if it doesn't exist
    pub fn init_config(path: Option<String>) -> Result<String, ParserError> {
        let parser = ConfigParser::new(path.clone());
        let config_path = parser.config_path.clone();
        
        if std::path::Path::new(&config_path).exists() {
            return Ok(config_path);
        }

        let default_config = parser.generate_default_config();
        std::fs::create_dir_all(
            std::path::Path::new(&config_path)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )?;
        
        std::fs::write(&config_path, default_config)?;
        Ok(config_path)
    }

    /// Validate config file
    pub fn validate_config(path: Option<String>) -> Result<(), ParserError> {
        let parser = ConfigParser::new(path);
        let _config = parser.load()?;
        println!("✅ Configuration is valid");
        Ok(())
    }

    /// Validate manifest file
    pub fn validate_manifest(path: String) -> Result<(), String> {
        let parser = ManifestParser::new(path);
        let _manifest = parser.load()
            .map_err(|e| format!("Manifest validation failed: {}", e))?;
        println!("✅ Manifest is valid");
        Ok(())
    }

    /// Print config information
    pub fn print_config_info(path: Option<String>) -> Result<(), ParserError> {
        let parser = ConfigParser::new(path);
        let config = parser.load()?;
        
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              WASMA Configuration Info                      ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        
        println!("🔧 Mode Configuration:");
        println!("  Multi-Instance: {}", config.uri_handling.multi_instances);
        println!("  Singularity: {}", config.uri_handling.singularity_instances);
        
        println!("\n📡 Protocols:");
        for (i, proto) in config.uri_handling.protocols.iter().enumerate() {
            println!("  [{}] {:?} - {}:{}", 
                i + 1, 
                proto.protocol, 
                proto.ip, 
                proto.port
            );
            if let Some(ref domain) = proto.domain {
                println!("      Domain: {}", domain);
            }
        }
        
        println!("\n💾 Resource Limits:");
        println!("  Scope Level: {}", config.resource_limits.scope_level);
        println!("  Renderer: {}", config.resource_limits.renderer);
        println!("  Execution Mode: {:?}", config.resource_limits.execution_mode);
        if let Some(mem) = config.resource_limits.max_memory_mb {
            println!("  Max Memory: {} MiB", mem);
        }
        if let Some(vram) = config.resource_limits.max_vram_mb {
            println!("  Max VRAM: {} MiB", vram);
        }
        if !config.resource_limits.cpu_cores.is_empty() {
            println!("  CPU Cores: {:?}", config.resource_limits.cpu_cores);
        }
        
        println!("\n👤 User Configuration:");
        println!("  User: {}", config.user_config.user_withed);
        println!("  Groups: {:?}", config.user_config.groups_withed);
        
        if let Some(ref comp) = config.uri_handling.compilation_server {
            println!("\n🔨 Compilation Server:");
            println!("  URI: {}:{}", comp.uri, comp.port);
        }
        
        println!();
        Ok(())
    }

    /// Print manifest information
    pub fn print_manifest_info(path: String) -> Result<(), String> {
        let parser = ManifestParser::new(path);
        let manifest = parser.load()
            .map_err(|e| format!("Failed to load manifest: {}", e))?;
        
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              WASMA Manifest Info                           ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        
        println!("📦 Application:");
        println!("  Name: {}", manifest.app.name);
        if let Some(ref img) = manifest.app.uri_appimg {
            println!("  Icon: {}", img);
        }
        
        println!("\n💾 Resources:");
        println!("  CPU Perception: {}", manifest.resources.cpu_perception);
        println!("  GPU Size: {} MB", manifest.resources.gpu_perp.default_size);
        println!("  RAM: {} {} MB", 
            manifest.resources.ram_using.ram_type,
            manifest.resources.ram_using.size
        );
        
        println!("\n🔐 Permissions:");
        println!("  Type: {:?}", manifest.permissions.permission_check);
        
        println!();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasma_core_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let core = WasmaCore::from_config(config, ResourceMode::Auto);
        assert!(core.is_singularity());
        assert!(!core.is_multi_instance());
    }

    #[test]
    fn test_manifest_integration() {
        // This test would require a valid manifest file
        // Placeholder for integration testing
    }
}

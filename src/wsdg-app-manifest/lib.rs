// WSDG App Manifest - Manifest and Permission Parser Library
// Part of WASMA (Windows Assignment System Monitoring Architecture)
// January 15, 2026

#![warn(missing_docs, rust_2018_idioms)]
#![doc = "WSDG App Manifest Parser Library - Handles .manifest files and permission sources"]

/// Manifest parser module for parsing .manifest files.
pub mod manifest_parser;
/// Source parser module for parsing permission source files.
pub mod source_parser;

// Re-export main types
pub use manifest_parser::{
    ManifestParser, ManifestError, WasmaManifest,
    AppMetadata, ResourceConfig, 
    CpuAffinityConfig, CpuCoreServe,
    GpuConfig, GpuAllocationType, GpuSizeMode, GpuUsing,
    RamConfig, CacheMode, RamBitwidth,
    PermissionReference, PermissionCheckType,
    WindowConfig,
};

pub use source_parser::{
    SourceParser, SourceError, PermissionSource,
    NetworkPermissions, WebResolving,
    FilesystemPermissions, FileException,
    UsbPermissions, UsbConnectional, PlugAndPlay,
    MediaPermissions, WebcamPermission, AudioPermission,
    SystemPermissions,
};

// Re-export wbackend types for convenience
pub use wbackend::ExecutionMode;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse a manifest file
///
/// # Arguments
/// * `path` - Path to .manifest file
///
/// # Returns
/// Parsed WasmaManifest on success
///
/// # Examples
/// ```no_run
/// use wsdg_app_manifest::parse_manifest;
/// 
/// let manifest = parse_manifest("app.manifest").unwrap();
/// println!("App: {}", manifest.app.name);
/// ```
pub fn parse_manifest(path: impl Into<String>) -> Result<WasmaManifest, ManifestError> {
    let parser = ManifestParser::new(path.into());
    parser.load()
}

/// Parse a permission source file
///
/// # Arguments
/// * `path` - Path to permission source file
///
/// # Returns
/// Parsed PermissionSource on success
///
/// # Examples
/// ```no_run
/// use wsdg_app_manifest::parse_source;
/// 
/// let source = parse_source("source").unwrap();
/// println!("Ethernet: {}", source.network.ethernet);
/// ```
pub fn parse_source(path: impl AsRef<str>) -> Result<PermissionSource, SourceError> {
    let parser = SourceParser::new(None);
    parser.load(path.as_ref())
}

/// Parse embedded permission source from manifest content
///
/// # Arguments
/// * `manifest_content` - Content of manifest file
///
/// # Returns
/// Optional PermissionSource if embedded source is found
pub fn parse_embedded_source(manifest_content: &str) -> Result<Option<PermissionSource>, SourceError> {
    let parser = SourceParser::new(None);
    parser.load_embedded(manifest_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_manifest_parsing() {
        let content = r#"
name = TestApp
cpu_perception = 2
ram_using = "DDR5" "2048MB" "*cache_resolved:swaponline"
        "#;

        let parser = ManifestParser::new("test.manifest".to_string());
        let manifest = parser.parse(content).unwrap();
        
        assert_eq!(manifest.app.name, "TestApp");
        assert_eq!(manifest.resources.cpu_perception, 2);
        assert_eq!(manifest.resources.ram_using.size, 2048);
    }

    #[test]
    fn test_source_parsing() {
        let content = r#"
USE_ETHERNET_CONNECTION = 1
USE_WIFI_CONNECTION = 0
USE_FILE_EXCEPTION = *file://Documents
        "#;

        let parser = SourceParser::new(None);
        let source = parser.parse(content).unwrap();
        
        assert!(source.network.ethernet);
        assert!(!source.network.wifi);
        
        match source.filesystem.file_exception {
            FileException::Specific(ref paths) => {
                assert_eq!(paths[0], "Documents");
            }
            _ => panic!("Expected Specific file exception"),
        }
    }
}

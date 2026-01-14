// WASMA Permission Source Parser
// Parses permission source files for application permissions
// January 15, 2026

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("Source file not found: {0}")]
    FileNotFound(String),
    
    #[error("Failed to read source: {0}")]
    ReadError(String),
    
    #[error("Failed to parse source line {line}: {reason}")]
    ParseError { line: usize, reason: String },
    
    #[error("Invalid permission value: {0}")]
    InvalidValue(String),
}

/// Permission Source Structure
#[derive(Debug, Clone)]
pub struct PermissionSource {
    /// Network permissions
    pub network: NetworkPermissions,
    
    /// File system permissions
    pub filesystem: FilesystemPermissions,
    
    /// USB permissions
    pub usb: UsbPermissions,
    
    /// Media permissions
    pub media: MediaPermissions,
    
    /// System permissions
    pub system: SystemPermissions,
    
    /// Custom permissions (key-value pairs)
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkPermissions {
    pub ethernet: bool,
    pub wifi: bool,
    pub web_resolving: WebResolving,
}

#[derive(Debug, Clone)]
pub enum WebResolving {
    AllDefault,
    Specific(Vec<String>), // Specific domains
}

impl Default for WebResolving {
    fn default() -> Self {
        WebResolving::AllDefault
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilesystemPermissions {
    pub chattr_disabler: bool,
    pub file_exception: FileException,
    pub file_width: Option<u64>, // in GB
}

#[derive(Debug, Clone)]
pub enum FileException {
    None,
    Specific(Vec<String>), // file://Documents
    QueryAll,              // *&ALL
    NoQuery,               // file://*
}

impl Default for FileException {
    fn default() -> Self {
        FileException::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct UsbPermissions {
    pub connectional: UsbConnectional,
    pub net: bool,
    pub plug_and_play: PlugAndPlay,
}

#[derive(Debug, Clone)]
pub enum UsbConnectional {
    None,
    AllDriver,
    DiskIot,
    EthernetUsbPort,
    UsbMedia,
    PlugAndPlayOnly,
}

impl Default for UsbConnectional {
    fn default() -> Self {
        UsbConnectional::None
    }
}

#[derive(Debug, Clone)]
pub enum PlugAndPlay {
    None,
    All,
    Microphone,
    Camera,
    Mouse,
    Midi,
    Specific(Vec<String>),
}

impl Default for PlugAndPlay {
    fn default() -> Self {
        PlugAndPlay::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct MediaPermissions {
    pub webcam: WebcamPermission,
    pub microphone: AudioPermission,
    pub audio: AudioPermission,
}

#[derive(Debug, Clone)]
pub enum WebcamPermission {
    No,
    StepByDevice, // &step_by_device = :use_webcam
    UseWebcam,    // use_webcam
    All,
}

impl Default for WebcamPermission {
    fn default() -> Self {
        WebcamPermission::No
    }
}

#[derive(Debug, Clone)]
pub enum AudioPermission {
    No,
    Justing, // One-time access
    OpenedAll,
    All,
}

impl Default for AudioPermission {
    fn default() -> Self {
        AudioPermission::No
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemPermissions {
    pub custom_fields: HashMap<String, String>,
}

/// Source Parser
pub struct SourceParser {
    base_path: Option<PathBuf>,
}

impl SourceParser {
    pub fn new(base_path: Option<PathBuf>) -> Self {
        Self { base_path }
    }

    /// Load permission source from file or embedded [source] section
    pub fn load(&self, path: &str) -> Result<PermissionSource, SourceError> {
        if !Path::new(path).exists() {
            return Err(SourceError::FileNotFound(path.to_string()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| SourceError::ReadError(e.to_string()))?;

        self.parse(&content)
    }

    /// Load from manifest embedded source
    pub fn load_embedded(&self, manifest_content: &str) -> Result<Option<PermissionSource>, SourceError> {
        if let Some(source_content) = self.extract_source_section(manifest_content) {
            Ok(Some(self.parse(&source_content)?))
        } else {
            Ok(None)
        }
    }

    /// Resolve source path based on permission type
    pub fn resolve_source_path(&self, permission_type: &str) -> PathBuf {
        let base = self.base_path.clone().unwrap_or_else(|| PathBuf::from("/"));
        
        match permission_type {
            "permission_sys" => base.join("$ROOT/lib/share/permission/source"),
            "permission_pinning" => base.join("$ROOT/lib/share/permissioner/source_pinning"),
            "permission_purning" => base.join("$ROOT/lib/share/permissionpurnering/source_purnering"),
            "permission_preset" => base.join("$ROOT/lib/share/permission/source_preset"),
            "permission_devel" => {
                // $HOME/$USE_CONFIG/permission_app/source
                base.join("$HOME/$USE_CONFIG/permission_app/source")
            }
            _ => base.join("source"),
        }
    }

    fn extract_source_section(&self, content: &str) -> Option<String> {
        if let Some(start_idx) = content.find("[source]") {
            let after_marker = &content[start_idx + 8..];
            return Some(after_marker.to_string());
        }
        None
    }

    fn parse(&self, content: &str) -> Result<PermissionSource, SourceError> {
        let mut network = NetworkPermissions::default();
        let mut filesystem = FilesystemPermissions::default();
        let mut usb = UsbPermissions::default();
        let mut media = MediaPermissions::default();
        let mut system = SystemPermissions::default();
        let mut custom = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("*//") || line.starts_with("//") {
                continue;
            }

            // Parse key = value pairs
            if let Some((key, value)) = self.split_key_value(line) {
                match key {
                    "USE_ETHERNET_CONNECTION" => {
                        network.ethernet = self.parse_bool(value);
                    }
                    "USE_WIFI_CONNECTION" => {
                        network.wifi = self.parse_bool(value);
                    }
                    "USE_WEB_RESOLVEING" | "USE_WEB_RESOLVING" => {
                        network.web_resolving = self.parse_web_resolving(value);
                    }
                    "USE_CHATTR_DISABLER" => {
                        filesystem.chattr_disabler = self.parse_bool(value);
                    }
                    "USE_FILE_EXCEPTION" => {
                        filesystem.file_exception = self.parse_file_exception(value);
                    }
                    "USE_FILE_WITDH" | "USE_FILE_WIDTH" => {
                        filesystem.file_width = self.parse_file_width(value);
                    }
                    "USE_USB_CONNECTIONAL" => {
                        usb.connectional = self.parse_usb_connectional(value);
                    }
                    "USE_USB_NET" => {
                        usb.net = self.parse_bool(value);
                    }
                    "USE_USB_PLUG_AND_PLAY" => {
                        usb.plug_and_play = self.parse_plug_and_play(value);
                    }
                    "USE_WEBCAM_OPENED" => {
                        media.webcam = self.parse_webcam_permission(value);
                    }
                    "USE_MICROPHONE" => {
                        media.microphone = self.parse_audio_permission(value);
                    }
                    "USE_AUDIO" => {
                        media.audio = self.parse_audio_permission(value);
                    }
                    _ => {
                        // Store as custom permission
                        custom.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Ok(PermissionSource {
            network,
            filesystem,
            usb,
            media,
            system,
            custom,
        })
    }

    fn split_key_value<'a>(&self, line: &'a str) -> Option<(&'a str, &'a str)> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            Some((parts[0].trim(), parts[1].trim()))
        } else {
            None
        }
    }

    fn parse_bool(&self, value: &str) -> bool {
        let value = value.trim().to_lowercase();
        value == "1" || value == "true" || value == "yes"
    }

    fn parse_web_resolving(&self, value: &str) -> WebResolving {
        let value = value.trim();
        
        if value.to_uppercase().contains("ALL_DEFAULT") {
            WebResolving::AllDefault
        } else if value.starts_with("http://") || value.starts_with("https://") {
            WebResolving::Specific(vec![value.to_string()])
        } else {
            WebResolving::AllDefault
        }
    }

    fn parse_file_exception(&self, value: &str) -> FileException {
        let value = value.trim();
        
        if value.contains("*&ALL") {
            FileException::QueryAll
        } else if value.contains("file://*") {
            FileException::NoQuery
        } else if value.starts_with("*file://") || value.starts_with("file://") {
            let path = value.trim_start_matches('*').trim_start_matches("file://");
            FileException::Specific(vec![path.to_string()])
        } else {
            FileException::None
        }
    }

    fn parse_file_width(&self, value: &str) -> Option<u64> {
        let value = value.trim().to_uppercase();
        
        if let Some(num_str) = value.split_whitespace().next() {
            if let Ok(num) = num_str.parse::<u64>() {
                // Assume GB if no unit specified
                return Some(num);
            }
        }
        
        None
    }

    fn parse_usb_connectional(&self, value: &str) -> UsbConnectional {
        let value = value.trim().to_lowercase();
        
        if value.contains("all_driver") {
            UsbConnectional::AllDriver
        } else if value.contains("driver_disk_iot") {
            UsbConnectional::DiskIot
        } else if value.contains("driver_ethernet_usb_port") {
            UsbConnectional::EthernetUsbPort
        } else if value.contains("driver_usb_media") {
            UsbConnectional::UsbMedia
        } else if value.contains("driver_usb_plug_and_play") {
            UsbConnectional::PlugAndPlayOnly
        } else {
            UsbConnectional::None
        }
    }

    fn parse_plug_and_play(&self, value: &str) -> PlugAndPlay {
        let value = value.trim().to_uppercase();
        
        if value == "ALL" {
            PlugAndPlay::All
        } else if value.contains("MICROPHONE") {
            PlugAndPlay::Microphone
        } else if value.contains("CAMERA") {
            PlugAndPlay::Camera
        } else if value.contains("MOUSE") {
            PlugAndPlay::Mouse
        } else if value.contains("MIDI") {
            PlugAndPlay::Midi
        } else {
            PlugAndPlay::None
        }
    }

    fn parse_webcam_permission(&self, value: &str) -> WebcamPermission {
        let value = value.trim().to_lowercase();
        
        if value.contains("all") && !value.contains("step") {
            WebcamPermission::All
        } else if value.contains("step_by_device") {
            WebcamPermission::StepByDevice
        } else if value.contains("use_webcam") {
            WebcamPermission::UseWebcam
        } else {
            WebcamPermission::No
        }
    }

    fn parse_audio_permission(&self, value: &str) -> AudioPermission {
        let value = value.trim().to_uppercase();
        
        if value == "ALL" {
            AudioPermission::All
        } else if value == "JUSTING" {
            AudioPermission::Justing
        } else if value.contains("OPENED_ALL") {
            AudioPermission::OpenedAll
        } else if value == "NO" {
            AudioPermission::No
        } else {
            AudioPermission::No
        }
    }
}

impl Default for PermissionSource {
    fn default() -> Self {
        Self {
            network: NetworkPermissions::default(),
            filesystem: FilesystemPermissions::default(),
            usb: UsbPermissions::default(),
            media: MediaPermissions::default(),
            system: SystemPermissions::default(),
            custom: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_parsing() {
        let content = r#"
USE_ETHERNET_CONNECTION = 1
USE_WIFI_CONNECTION = 0
USE_FILE_EXCEPTION = *file://Documents
USE_USB_CONNECTIONAL = driver_disk_iot
        "#;

        let parser = SourceParser::new(None);
        let source = parser.parse(content).unwrap();
        
        assert!(source.network.ethernet);
        assert!(!source.network.wifi);
        
        match source.filesystem.file_exception {
            FileException::Specific(paths) => {
                assert_eq!(paths[0], "Documents");
            }
            _ => panic!("Expected Specific file exception"),
        }
    }

    #[test]
    fn test_audio_permissions() {
        let content = r#"
USE_MICROPHONE = ALL
USE_AUDIO = JUSTING
        "#;

        let parser = SourceParser::new(None);
        let source = parser.parse(content).unwrap();
        
        assert!(matches!(source.media.microphone, AudioPermission::All));
        assert!(matches!(source.media.audio, AudioPermission::Justing));
    }
}

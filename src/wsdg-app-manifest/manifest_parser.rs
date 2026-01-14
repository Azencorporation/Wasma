// WASMA Manifest Parser
// Parses .manifest files for application resource configuration
// January 15, 2026

use std::fs;
use std::path::Path;
use thiserror::Error;
use wbackend::ExecutionMode;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Manifest file not found: {0}")]
    FileNotFound(String),
    
    #[error("Failed to read manifest: {0}")]
    ReadError(String),
    
    #[error("Failed to parse manifest line {line}: {reason}")]
    ParseError { line: usize, reason: String },
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
1    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
}

/// WASMA Application Manifest Structure
#[derive(Debug, Clone)]
pub struct WasmaManifest {
    /// Application metadata
    pub app: AppMetadata,
    
    /// Resource configuration
    pub resources: ResourceConfig,
    
    /// Permission reference
    pub permissions: PermissionReference,
    
    /// Window configuration
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Default)]
pub struct AppMetadata {
    pub name: String,
    pub uri_appimg: Option<String>,
    pub uri_shortcut: Option<String>,
    pub uri_app_source: Option<String>,
    pub uri_app_resource: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    // CPU Configuration
    pub cpu_perception: u32,
    pub cpu_affinity: CpuAffinityConfig,
    pub cpu_core_serve: CpuCoreServe,
    
    // GPU Configuration
    pub gpu_perp: GpuConfig,
    pub gpu_using: GpuUsing,
    
    // RAM Configuration
    pub ram_using: RamConfig,
    pub ram_used_bitwidth: RamBitwidth,
}

#[derive(Debug, Clone)]
pub struct CpuAffinityConfig {
    pub resource_max: u32,
    pub bitmax: u32,
}

#[derive(Debug, Clone)]
pub enum CpuCoreServe {
    Static(u32),
    Dynamic,
    AffinityDefault,
}

#[derive(Debug, Clone)]
pub struct GpuConfig {
    pub allocation_type: GpuAllocationType,
    pub size_mode: GpuSizeMode,
    pub default_size: u64, // in MB
}

#[derive(Debug, Clone)]
pub enum GpuAllocationType {
    Allocation,
    Location(String),
}

#[derive(Debug, Clone)]
pub enum GpuSizeMode {
    ByDefault,
    ByCustom,
    BySection,
    ByProp,
}

#[derive(Debug, Clone)]
pub struct GpuUsing {
    pub size: u64, // in MB
    pub resource_max: u32,
    pub bitwidth: u32,
}

#[derive(Debug, Clone)]
pub struct RamConfig {
    pub ram_type: String, // DDR4, DDR5, etc.
    pub size: u64, // in MB
    pub cache_mode: CacheMode,
}

#[derive(Debug, Clone)]
pub enum CacheMode {
    SwapOnline,
    SwapOffline,
    Resolved,
}

#[derive(Debug, Clone)]
pub struct RamBitwidth {
    pub size: u64, // in MB
    pub bit_width: u32,
    pub cache_resourceing: f32, // percentage
}

#[derive(Debug, Clone)]
pub struct PermissionReference {
    pub permission_check: PermissionCheckType,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PermissionCheckType {
    PermissionDevel,
    PermissionSys,
    PermissionPreset,
    PermissionPinning,
    PermissionPurning,
}

#[derive(Debug, Clone, Default)]
pub struct WindowConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: bool,
}

/// Manifest Parser
pub struct ManifestParser {
    path: String,
}

impl ManifestParser {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<WasmaManifest, ManifestError> {
        if !Path::new(&self.path).exists() {
            return Err(ManifestError::FileNotFound(self.path.clone()));
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| ManifestError::ReadError(e.to_string()))?;

        self.parse(&content)
    }

    fn parse(&self, content: &str) -> Result<WasmaManifest, ManifestError> {
        let mut app = AppMetadata::default();
        let mut cpu_perception = 1;
        let mut cpu_affinity = CpuAffinityConfig { resource_max: 10, bitmax: 20 };
        let mut cpu_core_serve = CpuCoreServe::Static(1);
        let mut gpu_perp = GpuConfig {
            allocation_type: GpuAllocationType::Allocation,
            size_mode: GpuSizeMode::ByDefault,
            default_size: 1024,
        };
        let mut gpu_using = GpuUsing {
            size: 1024,
            resource_max: 15,
            bitwidth: 25,
        };
        let mut ram_using = RamConfig {
            ram_type: "DDR5".to_string(),
            size: 1024,
            cache_mode: CacheMode::SwapOnline,
        };
        let mut ram_bitwidth = RamBitwidth {
            size: 1024,
            bit_width: 15,
            cache_resourceing: 20.0,
        };
        let mut permission_ref = PermissionReference {
            permission_check: PermissionCheckType::PermissionDevel,
            source_path: None,
        };
        let mut window = WindowConfig::default();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("*//") || line.starts_with("//") {
                continue;
            }

            // Parse key = value pairs
            if let Some((key, value)) = self.split_key_value(line) {
                match key {
                    "name" => {
                        app.name = self.extract_value(value);
                    }
                    "uri_appimg" => {
                        app.uri_appimg = Some(self.extract_uri(value));
                    }
                    "uri_shortcut" => {
                        app.uri_shortcut = Some(self.extract_uri(value));
                    }
                    "uri_app_source" => {
                        app.uri_app_source = Some(self.extract_uri(value));
                    }
                    "uri_app_resource" => {
                        app.uri_app_resource = self.extract_uri_list(value);
                    }
                    "cpu_perception" => {
                        cpu_perception = self.parse_u32(value, line_num, "cpu_perception")?;
                    }
                    "cpu_affinity" => {
                        cpu_affinity = self.parse_cpu_affinity(value, line_num)?;
                    }
                    "cpu_core_serve" => {
                        cpu_core_serve = self.parse_cpu_core_serve(value, line_num)?;
                    }
                    "gpu_perp" => {
                        gpu_perp = self.parse_gpu_perp(value, line_num)?;
                    }
                    "gpu_using" => {
                        gpu_using = self.parse_gpu_using(value, line_num)?;
                    }
                    "ram_using" => {
                        ram_using = self.parse_ram_using(value, line_num)?;
                    }
                    "ram_used_bitwitdh" | "ram_used_bitwidth" => {
                        ram_bitwidth = self.parse_ram_bitwidth(value, line_num)?;
                    }
                    "permission_check" => {
                        permission_ref = self.parse_permission_check(value, line_num)?;
                    }
                    _ => {
                        // Unknown key, skip
                    }
                }
            }
        }

        Ok(WasmaManifest {
            app,
            resources: ResourceConfig {
                cpu_perception,
                cpu_affinity,
                cpu_core_serve,
                gpu_perp,
                gpu_using,
                ram_using,
                ram_used_bitwidth: ram_bitwidth,
            },
            permissions: permission_ref,
            window,
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

    fn extract_value(&self, value: &str) -> String {
        // Remove comments
        let value = value.split("*//").next().unwrap_or(value).trim();
        value.to_string()
    }

    fn extract_uri(&self, value: &str) -> String {
        let value = self.extract_value(value);
        // Remove quotes if present
        value.trim_matches('"').trim_matches('\'').to_string()
    }

    fn extract_uri_list(&self, value: &str) -> Vec<String> {
        let value = self.extract_value(value);
        value.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn parse_u32(&self, value: &str, line_num: usize, field: &str) -> Result<u32, ManifestError> {
        let value = self.extract_value(value);
        value.parse().map_err(|_| ManifestError::ParseError {
            line: line_num + 1,
            reason: format!("Invalid {} value: {}", field, value),
        })
    }

    fn parse_cpu_affinity(&self, value: &str, line_num: usize) -> Result<CpuAffinityConfig, ManifestError> {
        // Parse: perception { 100 resource_max : 10 } bitmax *"20"
        let mut resource_max = 10;
        let mut bitmax = 20;

        if let Some(brace_content) = self.extract_braces(value) {
            if let Some(rm) = self.extract_field(&brace_content, "resource_max") {
                resource_max = rm.parse().unwrap_or(10);
            }
        }

        if let Some(bm) = self.extract_quoted_field(value, "bitmax") {
            bitmax = bm.parse().unwrap_or(20);
        }

        Ok(CpuAffinityConfig { resource_max, bitmax })
    }

    fn parse_cpu_core_serve(&self, value: &str, line_num: usize) -> Result<CpuCoreServe, ManifestError> {
        let value = self.extract_value(value);
        
        if value.contains("dynamic") || value.contains("\"dynamic\"") {
            Ok(CpuCoreServe::Dynamic)
        } else if value.contains("affinity_default") {
            Ok(CpuCoreServe::AffinityDefault)
        } else {
            // Try to extract numeric value
            let num_str = value.trim_matches('"').trim();
            if let Ok(num) = num_str.parse::<u32>() {
                Ok(CpuCoreServe::Static(num))
            } else {
                Ok(CpuCoreServe::AffinityDefault)
            }
        }
    }

    fn parse_gpu_perp(&self, value: &str, line_num: usize) -> Result<GpuConfig, ManifestError> {
        // Parse: "VRAM:allocation:size_bydefault = 1024"
        let value = self.extract_value(value).trim_matches('"').to_string();
        
        let mut allocation_type = GpuAllocationType::Allocation;
        let mut size_mode = GpuSizeMode::ByDefault;
        let mut default_size = 1024;

        if value.contains("location") {
            allocation_type = GpuAllocationType::Location(value.clone());
        }

        if value.contains("size_bycustom") {
            size_mode = GpuSizeMode::ByCustom;
        } else if value.contains("size_byinsection") {
            size_mode = GpuSizeMode::BySection;
        } else if value.contains("size_byprop") {
            size_mode = GpuSizeMode::ByProp;
        }

        // Extract size
        if let Some(size_str) = value.split('=').nth(1) {
            if let Ok(size) = size_str.trim().parse::<u64>() {
                default_size = size;
            }
        }

        Ok(GpuConfig {
            allocation_type,
            size_mode,
            default_size,
        })
    }

    fn parse_gpu_using(&self, value: &str, line_num: usize) -> Result<GpuUsing, ManifestError> {
        // Parse: "1024" { 100 resource_max : 15 } bitwidthed *"25"
        let mut size = 1024;
        let mut resource_max = 15;
        let mut bitwidth = 25;

        // Extract size
        if let Some(size_str) = value.split_whitespace().next() {
            if let Ok(s) = size_str.trim_matches('"').parse::<u64>() {
                size = s;
            }
        }

        // Extract resource_max from braces
        if let Some(brace_content) = self.extract_braces(value) {
            if let Some(rm) = self.extract_field(&brace_content, "resource_max") {
                resource_max = rm.parse().unwrap_or(15);
            }
        }

        // Extract bitwidth
        if let Some(bw) = self.extract_quoted_field(value, "bitwidthed") {
            bitwidth = bw.parse().unwrap_or(25);
        }

        Ok(GpuUsing { size, resource_max, bitwidth })
    }

    fn parse_ram_using(&self, value: &str, line_num: usize) -> Result<RamConfig, ManifestError> {
        // Parse: "DDR5" "1024MB" "*cache_resolved:swaponline"
        let parts: Vec<&str> = value.split_whitespace().collect();
        
        let mut ram_type = "DDR5".to_string();
        let mut size = 1024;
        let mut cache_mode = CacheMode::SwapOnline;

        if !parts.is_empty() {
            ram_type = parts[0].trim_matches('"').to_string();
        }

        if parts.len() > 1 {
            let size_str = parts[1].trim_matches('"').replace("MB", "");
            size = size_str.parse().unwrap_or(1024);
        }

        if parts.len() > 2 {
            let cache_str = parts[2].to_lowercase();
            if cache_str.contains("swapoffline") {
                cache_mode = CacheMode::SwapOffline;
            } else if cache_str.contains("swaponline") {
                cache_mode = CacheMode::SwapOnline;
            } else if cache_str.contains("resolved") {
                cache_mode = CacheMode::Resolved;
            }
        }

        Ok(RamConfig { ram_type, size, cache_mode })
    }

    fn parse_ram_bitwidth(&self, value: &str, line_num: usize) -> Result<RamBitwidth, ManifestError> {
        // Parse: "1024MB" "bit_width : 15" *cache_resourceing : "20%"
        let mut size = 1024;
        let mut bit_width = 15;
        let mut cache_resourceing = 20.0;

        // Extract size
        if let Some(size_str) = value.split_whitespace().next() {
            let s = size_str.trim_matches('"').replace("MB", "");
            size = s.parse().unwrap_or(1024);
        }

        // Extract bit_width
        if let Some(bw) = self.extract_field(value, "bit_width") {
            bit_width = bw.parse().unwrap_or(15);
        }

        // Extract cache_resourceing
        if let Some(cr) = self.extract_field(value, "cache_resourceing") {
            let percent = cr.trim_matches('"').replace("%", "");
            cache_resourceing = percent.parse().unwrap_or(20.0);
        }

        Ok(RamBitwidth { size, bit_width, cache_resourceing })
    }

    fn parse_permission_check(&self, value: &str, line_num: usize) -> Result<PermissionReference, ManifestError> {
        // Parse: URI:PERMISSION_DEVEL://string : permission_devel *USER
        let value_lower = value.to_lowercase();
        
        let permission_type = if value_lower.contains("permission_sys") {
            PermissionCheckType::PermissionSys
        } else if value_lower.contains("permission_preset") {
            PermissionCheckType::PermissionPreset
        } else if value_lower.contains("permission_pinning") {
            PermissionCheckType::PermissionPinning
        } else if value_lower.contains("permission_purning") {
            PermissionCheckType::PermissionPurning
        } else {
            PermissionCheckType::PermissionDevel
        };

        Ok(PermissionReference {
            permission_check: permission_type,
            source_path: None, // Will be resolved later
        })
    }

    // Helper functions
    fn extract_braces(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find('{') {
            if let Some(end) = text.find('}') {
                return Some(text[start+1..end].to_string());
            }
        }
        None
    }

    fn extract_field(&self, text: &str, field: &str) -> Option<String> {
        if let Some(idx) = text.find(field) {
            let after = &text[idx..];
            if let Some(colon_idx) = after.find(':') {
                let value_part = &after[colon_idx+1..];
                let value = value_part.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches('"');
                return Some(value.to_string());
            }
        }
        None
    }

    fn extract_quoted_field(&self, text: &str, marker: &str) -> Option<String> {
        if let Some(idx) = text.find(marker) {
            let after = &text[idx..];
            if let Some(quote_start) = after.find('"') {
                let after_quote = &after[quote_start+1..];
                if let Some(quote_end) = after_quote.find('"') {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

// WASMA - Configuration Parser
// Sadece wasma.in.conf dosyasını okur ve parse eder

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use thiserror::Error;
use wbackend::ExecutionMode;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Grpc,
    Http,
    Https,
    Tor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol: Protocol,
    pub ip: IpAddr,
    pub port: u16,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UriHandlingConfig {
    pub multi_instances: bool,
    pub singularity_instances: bool,
    pub protocols: Vec<ProtocolConfig>,
    pub window_app_spec: String,
    pub compilation_server: Option<CompilationServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationServer {
    pub uri: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub user_withed: String,
    pub groups_withed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub ip_scope: String,
    pub scope_level: u32,
    pub renderer: String,
    // ✅ WASMA-specific extended fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vram_mb: Option<u64>,
    #[serde(default)]
    pub cpu_cores: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmaConfig {
    pub uri_handling: UriHandlingConfig,
    pub user_config: UserConfig,
    pub resource_limits: ResourceLimits,
}

/// Config Parser - Sadece dosya okuma ve parsing
pub struct ConfigParser {
    pub config_path: String,
}

impl ConfigParser {
    pub fn new(config_path: Option<String>) -> Self {
        let path = config_path.unwrap_or_else(|| "/etc/wasma/wasma.in.conf".to_string());
        Self { config_path: path }
    }

    /// Config dosyasını yükle
    pub fn load(&self) -> Result<WasmaConfig, ParserError> {
        if !Path::new(&self.config_path).exists() {
            return Err(ParserError::ConfigNotFound(self.config_path.clone()));
        }

        let content = fs::read_to_string(&self.config_path)?;
        self.parse(&content)
    }

    /// Config içeriğini parse et
    pub fn parse(&self, content: &str) -> Result<WasmaConfig, ParserError> {
        let mut multi_instances = false;
        let mut singularity_instances = false;
        let mut protocols = Vec::new();
        let mut window_app_spec = String::new();
        let mut compilation_server = None;
        let mut user_withed = "sysuser".to_string();
        let mut groups_withed = Vec::new();
        let mut ip_scope = "ip_base10".to_string();
        let mut scope_level = 50;
        let mut renderer = "glx_renderer".to_string();
        
        // ✅ Extended fields
        let mut execution_mode = None;
        let mut max_memory_mb = None;
        let mut max_vram_mb = None;
        let mut cpu_cores = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("*//") || line.starts_with("#") || line.is_empty() {
                continue;
            }

            if line.contains("multi_instances") {
                multi_instances = line.contains("true");
            }

            if line.contains("singularity_instances") {
                singularity_instances = line.contains("true");
            }

            if line.contains("protocol_def") {
                if let Some(proto_str) = self.extract_value(line) {
                    let parts: Vec<&str> = proto_str.split("://").collect();
                    if parts.len() == 2 {
                        let protocol = match parts[0] {
                            "tor" => Protocol::Tor,
                            "https" => Protocol::Https,
                            "http" => Protocol::Http,
                            "grpc" => Protocol::Grpc,
                            _ => Protocol::Http,
                        };
                        
                        let addr_parts: Vec<&str> = parts[1].split(':').collect();
                        if addr_parts.len() == 2 {
                            let ip = addr_parts[0].parse()
                                .map_err(|e| ParserError::ParseError(format!("Invalid IP: {}", e)))?;
                            let port = addr_parts[1].parse()
                                .map_err(|e| ParserError::ParseError(format!("Invalid port: {}", e)))?;
                            
                            protocols.push(ProtocolConfig {
                                protocol,
                                ip,
                                port,
                                domain: None,
                            });
                        }
                    }
                }
            }

            if line.contains("domain_def") {
                if let Some(d) = self.extract_value(line) {
                    if let Some(last_proto) = protocols.last_mut() {
                        last_proto.domain = Some(d.to_string());
                    }
                }
            }

            if line.contains("uri_handling_window_appspef") {
                if let Some(spec) = self.extract_value(line) {
                    window_app_spec = spec.to_string();
                }
            }

            if line.contains("uri_compilation_define") {
                if let Some(comp_uri) = self.extract_value(line) {
                    let parts: Vec<&str> = comp_uri.split("://").collect();
                    if parts.len() == 2 {
                        let addr_parts: Vec<&str> = parts[1].split(':').collect();
                        if addr_parts.len() == 2 {
                            compilation_server = Some(CompilationServer {
                                uri: addr_parts[0].to_string(),
                                port: addr_parts[1].parse().unwrap_or(90),
                            });
                        }
                    }
                }
            }

            if line.contains("user_withed") {
                if let Some(user) = self.extract_value_from_parens(line) {
                    user_withed = user.to_string();
                }
            }

            if line.contains("groups_ewithed") {
                if let Some(groups) = self.extract_value_from_parens(line) {
                    groups_withed = groups.split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }

            if line.contains("in_limited_scope") {
                if let Some(scope) = line.split("in_limited_scope:").nth(1) {
                    ip_scope = scope.split_whitespace().next()
                        .unwrap_or("ip_base10").to_string();
                }
            }

            if line.contains("in_scoped_bylevel") {
                if let Some(level) = line.split("in_scoped_bylevel:").nth(1) {
                    scope_level = level.split_whitespace().next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(50);
                }
            }

            if line.contains("in_request_withed") {
                if let Some(rend) = line.split("in_request_withed:").nth(1) {
                    renderer = rend.split_whitespace().next()
                        .unwrap_or("glx_renderer").to_string();
                }
            }

            // ✅ Extended parsing
            if line.contains("execution_mode") {
                if let Some(mode_str) = self.extract_value(line) {
                    execution_mode = match mode_str.to_lowercase().as_str() {
                        "cpu" | "cpu_only" => Some(ExecutionMode::CpuOnly),
                        "gpu" | "gpu_only" => Some(ExecutionMode::GpuOnly),
                        "gpu_preferred" | "gpu_pref" => Some(ExecutionMode::GpuPreferred),
                        "hybrid" => Some(ExecutionMode::Hybrid),
                        _ => Some(ExecutionMode::GpuPreferred),
                    };
                }
            }

            if line.contains("max_memory_mb") {
                if let Some(mem_str) = self.extract_value(line) {
                    max_memory_mb = mem_str.parse().ok();
                }
            }

            if line.contains("max_vram_mb") {
                if let Some(vram_str) = self.extract_value(line) {
                    max_vram_mb = vram_str.parse().ok();
                }
            }

            if line.contains("cpu_cores") {
                if let Some(cores_str) = self.extract_value(line) {
                    cpu_cores = cores_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                }
            }
        }

        Ok(WasmaConfig {
            uri_handling: UriHandlingConfig {
                multi_instances,
                singularity_instances,
                protocols,
                window_app_spec,
                compilation_server,
            },
            user_config: UserConfig {
                user_withed,
                groups_withed,
            },
            resource_limits: ResourceLimits {
                ip_scope,
                scope_level,
                renderer,
                execution_mode,
                max_memory_mb,
                max_vram_mb,
                cpu_cores,
            },
        })
    }

    /// Default config üret
    pub fn generate_default_config(&self) -> String {
        r#"uri_handling_op {
multi_instances = false;
singularity_instances = true;
protocol_def : http://127.0.0.1:8080
uri_handling_window_appspef : file://server_request/request.manifest
#*_END_BLOCK_DEFINE
uO:?? user_withed(*sysuser)
rg0:?? groups_ewithed(*groups_insys)
r0:?? in_limited_scope:ip_base10 in_scoped_bylevel:50 in_request_withed:glx_renderer
execution_mode : gpu_preferred
max_memory_mb : 512
max_vram_mb : 256
}"#.to_string()
    }

    /// Validation
    pub fn validate(&self, config: &WasmaConfig) -> Result<(), ParserError> {
        if config.uri_handling.multi_instances && config.uri_handling.singularity_instances {
            return Err(ParserError::InvalidConfig(
                "Cannot enable both multi_instances and singularity_instances".to_string()
            ));
        }

        if config.uri_handling.singularity_instances && config.uri_handling.protocols.len() > 1 {
            return Err(ParserError::InvalidConfig(
                "Singularity mode only allows one protocol".to_string()
            ));
        }

        if config.uri_handling.protocols.is_empty() {
            return Err(ParserError::InvalidConfig(
                "At least one protocol must be configured".to_string()
            ));
        }

        for proto in &config.uri_handling.protocols {
            if proto.port == 0 {
                return Err(ParserError::InvalidConfig(
                    format!("Port must be between 1-65535, got {}", proto.port)
                ));
            }
        }

        Ok(())
    }

// Satır 341-345: Some() ile sar
fn extract_value<'a>(&self, line: &'a str) -> Option<&'a str> {
    Some(line.split(':')
        .nth(1)?
        .split("*//")
        .next()?
        .trim())
}

// Satır 349-353: Some() ile sar
fn extract_value_from_parens<'a>(&self, line: &'a str) -> Option<&'a str> {
    Some(line.split('(')
        .nth(1)?
        .split(')')
        .next()?
        .trim_start_matches('*'))
     }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = ConfigParser::new(None);
        assert_eq!(parser.config_path, "/etc/wasma/wasma.in.conf");
    }

    #[test]
    fn test_config_parsing() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let result = parser.parse(&config_content);
        
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.uri_handling.singularity_instances);
        assert!(!config.uri_handling.multi_instances);
        assert_eq!(config.uri_handling.protocols.len(), 1);
        assert_eq!(config.resource_limits.max_memory_mb, Some(512));
        assert_eq!(config.resource_limits.max_vram_mb, Some(256));
    }

    #[test]
    fn test_validation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        assert!(parser.validate(&config).is_ok());
    }

    #[test]
    fn test_execution_mode_parsing() {
        let parser = ConfigParser::new(None);
        let config_content = "execution_mode : hybrid\n";
        let config = parser.parse(config_content).unwrap();
        
        assert_eq!(config.resource_limits.execution_mode, Some(ExecutionMode::Hybrid));
    }
}

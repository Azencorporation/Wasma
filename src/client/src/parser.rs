use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),
    #[error("Invalid protocol configuration: {0}")]
    InvalidConfig(String),
    #[error("Protocol parsing error: {0}")]
    ParseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Grpc,
    Http,
    Https,
    Tor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UriHandlingConfig {
    pub multi_instances: bool,        // true: multiple protocols allowed (tor+http+https+grpc)
    pub singularity_instances: bool,  // true: only single protocol allowed
    pub protocols: Vec<ProtocolConfig>, // List of protocol configurations
    pub window_app_spec: String,      // User-defined: manifest file path
    pub compilation_server: Option<CompilationServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol: Protocol,           // User choice: grpc, http, https, tor
    pub ip: IpAddr,                   // User-defined: any valid IP address
    pub port: u16,                    // User-defined: any port 1-65535
    pub domain: Option<String>,       // User-defined: any domain (not restricted to .onion)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationServer {
    pub uri: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub user_withed: String,         // User-defined: any system-level user variable
    pub groups_withed: Vec<String>,  // User-defined: any system groups
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub ip_scope: String,        // User-defined: any IP request scope identifier
    pub scope_level: u32,        // User-defined: IP request limit (e.g., 20, 30, 50)
    pub renderer: String,        // User-defined: any renderer (glx_renderer, vulkan, cpu, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmaConfig {
    pub uri_handling: UriHandlingConfig,
    pub user_config: UserConfig,
    pub resource_limits: ResourceLimits,
}

pub struct ProtocolManager {
    config_path: String,
    config: Option<WasmaConfig>,
}

impl ProtocolManager {
    pub fn new(config_path: Option<String>) -> Self {
        let path = config_path.unwrap_or_else(|| "/etc/wasma/wasma.in.conf".to_string());
        Self {
            config_path: path,
            config: None,
        }
    }

    /// Load and parse configuration from wasma.in.conf
    pub fn load_config(&mut self) -> Result<(), ProtocolError> {
        if !Path::new(&self.config_path).exists() {
            return Err(ProtocolError::ConfigNotFound(self.config_path.clone()));
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| ProtocolError::ConfigNotFound(e.to_string()))?;

        self.config = Some(self.parse_config(&content)?);
        Ok(())
    }

    /// Parse wasma.in.conf format
    fn parse_config(&self, content: &str) -> Result<WasmaConfig, ProtocolError> {
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

        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.starts_with("*//") || line.starts_with("#") || line.is_empty() {
                continue;
            }

            // Parse multi_instances
            if line.contains("multi_instances") {
                multi_instances = line.contains("true");
            }

            // Parse singularity_instances
            if line.contains("singularity_instances") {
                singularity_instances = line.contains("true");
            }

            // Parse protocol_def
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
                                .map_err(|e| ProtocolError::ParseError(format!("Invalid IP: {}", e)))?;
                            let port = addr_parts[1].parse()
                                .map_err(|e| ProtocolError::ParseError(format!("Invalid port: {}", e)))?;
                            
                            // Check for domain in next lines
                            let mut domain = None;
                            
                            protocols.push(ProtocolConfig {
                                protocol,
                                ip,
                                port,
                                domain: None, // Will be filled by domain_def
                            });
                        }
                    }
                }
            }

            // Parse domain_def - apply to last added protocol
            if line.contains("domain_def") {
                if let Some(d) = self.extract_value(line) {
                    if let Some(last_proto) = protocols.last_mut() {
                        last_proto.domain = Some(d.to_string());
                    }
                }
            }

            // Parse uri_handling_window_appspef
            if line.contains("uri_handling_window_appspef") {
                if let Some(spec) = self.extract_value(line) {
                    window_app_spec = spec.to_string();
                }
            }

            // Parse uri_compilation_define
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

            // Parse user_withed
            if line.contains("user_withed") {
                if let Some(user) = self.extract_value_from_parens(line) {
                    user_withed = user.to_string();
                }
            }

            // Parse groups_ewithed
            if line.contains("groups_ewithed") {
                if let Some(groups) = self.extract_value_from_parens(line) {
                    groups_withed = groups.split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }

            // Parse resource limits
            if line.contains("in_limited_scope") {
                if let Some(scope) = line.split("in_limited_scope:").nth(1) {
                    // Extract ip_baseXX where XX can be any number
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
                    // Any renderer: glx_renderer, vulkan, cpu, software, etc.
                    renderer = rend.split_whitespace().next()
                        .unwrap_or("glx_renderer").to_string();
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
            },
        })
    }

    fn extract_value(&self, line: &str) -> Option<&str> {
        line.split(':')
            .nth(1)?
            .split("*//")
            .next()?
            .trim()
    }

    fn extract_value_from_parens(&self, line: &str) -> Option<&str> {
        line.split('(')
            .nth(1)?
            .split(')')
            .next()?
            .trim_start_matches('*')
    }

    pub fn get_config(&self) -> Option<&WasmaConfig> {
        self.config.as_ref()
    }

    /// Get protocol endpoint URL(s)
    pub fn get_endpoint_urls(&self) -> Result<Vec<String>, ProtocolError> {
        let config = self.config.as_ref()
            .ok_or_else(|| ProtocolError::InvalidConfig("Config not loaded".to_string()))?;

        let mut urls = Vec::new();

        for proto_config in &config.uri_handling.protocols {
            let protocol_str = match proto_config.protocol {
                Protocol::Grpc => "grpc",
                Protocol::Http => "http",
                Protocol::Https => "https",
                Protocol::Tor => "tor",
            };

            let url = if let Some(ref domain) = proto_config.domain {
                format!("{}://{}", protocol_str, domain)
            } else {
                format!("{}://{}:{}", 
                    protocol_str, 
                    proto_config.ip, 
                    proto_config.port
                )
            };
            urls.push(url);
        }

        Ok(urls)
    }

    /// Get primary endpoint URL (first protocol)
    pub fn get_primary_endpoint(&self) -> Result<String, ProtocolError> {
        let urls = self.get_endpoint_urls()?;
        urls.first()
            .cloned()
            .ok_or_else(|| ProtocolError::InvalidConfig("No protocols configured".to_string()))
    }

    /// Check if multi-instance mode is enabled
    pub fn is_multi_instance(&self) -> bool {
        self.config.as_ref()
            .map(|c| c.uri_handling.multi_instances)
            .unwrap_or(false)
    }

    /// Check if singularity mode is enabled
    pub fn is_singularity(&self) -> bool {
        self.config.as_ref()
            .map(|c| c.uri_handling.singularity_instances)
            .unwrap_or(false)
    }

    /// Get all configured protocols
    pub fn get_protocols(&self) -> Option<&Vec<ProtocolConfig>> {
        self.config.as_ref()
            .map(|c| &c.uri_handling.protocols)
    }

    /// Get manifest file path
    pub fn get_manifest_path(&self) -> Option<String> {
        self.config.as_ref()
            .map(|c| c.uri_handling.window_app_spec.clone())
    }

    /// Get compilation server info
    pub fn get_compilation_server(&self) -> Option<&CompilationServer> {
        self.config.as_ref()
            .and_then(|c| c.uri_handling.compilation_server.as_ref())
    }

    /// Get user configuration
    pub fn get_user_config(&self) -> Option<&UserConfig> {
        self.config.as_ref()
            .map(|c| &c.user_config)
    }

    /// Get resource limits
    pub fn get_resource_limits(&self) -> Option<&ResourceLimits> {
        self.config.as_ref()
            .map(|c| &c.resource_limits)
    }

    /// Validate configuration - only basic checks, user has full control
    pub fn validate(&self) -> Result<(), ProtocolError> {
        let config = self.config.as_ref()
            .ok_or_else(|| ProtocolError::InvalidConfig("Config not loaded".to_string()))?;

        // Check mode consistency
        if config.uri_handling.multi_instances && config.uri_handling.singularity_instances {
            return Err(ProtocolError::InvalidConfig(
                "Cannot enable both multi_instances and singularity_instances".to_string()
            ));
        }

        // Check protocol count based on mode
        if config.uri_handling.singularity_instances && config.uri_handling.protocols.len() > 1 {
            return Err(ProtocolError::InvalidConfig(
                "Singularity mode only allows one protocol".to_string()
            ));
        }

        if config.uri_handling.protocols.is_empty() {
            return Err(ProtocolError::InvalidConfig(
                "At least one protocol must be configured".to_string()
            ));
        }

        // Validate each protocol
        for proto in &config.uri_handling.protocols {
            if proto.port == 0 || proto.port > 65535 {
                return Err(ProtocolError::InvalidConfig(
                    format!("Port must be between 1-65535, got {}", proto.port)
                ));
            }
        }

        // Validate manifest path exists
        if config.uri_handling.window_app_spec.is_empty() {
            return Err(ProtocolError::InvalidConfig(
                "Window app spec path cannot be empty".to_string()
            ));
        }

        // Validate user exists in system (basic check)
        if config.user_config.user_withed.is_empty() {
            return Err(ProtocolError::InvalidConfig(
                "User withed cannot be empty - must specify system user".to_string()
            ));
        }

        // Validate groups format: must be groups_username
        for group in &config.user_config.groups_withed {
            if !group.starts_with("groups_") {
                return Err(ProtocolError::InvalidConfig(
                    format!("Group '{}' must start with 'groups_' prefix", group)
                ));
            }
        }

        // Validate ip_scope format: must start with ip_base
        if !config.resource_limits.ip_scope.starts_with("ip_base") {
            return Err(ProtocolError::InvalidConfig(
                format!("IP scope '{}' must start with 'ip_base' followed by a number", 
                    config.resource_limits.ip_scope)
            ));
        }

        // Note: System-level validation (user exists, groups have WASMA permissions) 
        // should be done by system integration layer

        Ok(())
    }

    /// Check if user groups have required WASMA permissions
    pub fn check_wasma_permissions(&self, system_groups: &[String]) -> Vec<WasmaPermission> {
        let mut found_permissions = Vec::new();
        
        for group in system_groups {
            if let Some(perm) = WasmaPermission::from_str(group) {
                found_permissions.push(perm);
            }
        }
        
        found_permissions
    }

    /// Check if user has specific WASMA permission
    pub fn has_permission(&self, system_groups: &[String], permission: WasmaPermission) -> bool {
        system_groups.iter()
            .any(|g| WasmaPermission::from_str(g) == Some(permission.clone()))
    }

    /// Get network control level based on permissions
    pub fn get_network_control_level(&self, system_groups: &[String]) -> NetworkControlLevel {
        if self.has_permission(system_groups, WasmaPermission::NetWide) {
            NetworkControlLevel::Full  // Direct network control
        } else if self.has_permission(system_groups, WasmaPermission::WasmaNetWideSys) {
            NetworkControlLevel::Limited  // Limited network control
        } else {
            NetworkControlLevel::None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkControlLevel {
    Full,    // netwide - direct network control
    Limited, // wasmanetwidthsys - limited network control
    None,    // no network permissions
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_manager_creation() {
        let manager = ProtocolManager::new(None);
        assert_eq!(manager.config_path, "/etc/wasma/wasma.in.conf");
    }

    #[test]
    fn test_config_parsing() {
        let manager = ProtocolManager::new(None);
        let config_content = r#"
uri_handling_op {
multi_instances = true;
singularity_instances = false;
protocol_def : tor://127.0.0.1:80
domain_def : azccriminal.onion
protocol_def : https://192.168.1.100:8443
domain_def : myserver.local
uri_handling_window_appspef : file://server_request/request.manifest
uri_compilation_define : uri://compilation_server:90
#*_END_BLOCK_DEFINE
uO:?? user_withed(*sysuser)
rg0:?? groups_ewithed(*groups_insys)
r0:?? in_limited_scope:ip_base10 in_scoped_bylevel:50 in_request_withed:glx_renderer
}
        "#;
        
        let result = manager.parse_config(config_content);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert!(config.uri_handling.multi_instances);
        assert!(!config.uri_handling.singularity_instances);
        assert_eq!(config.uri_handling.protocols.len(), 2);
        assert!(matches!(config.uri_handling.protocols[0].protocol, Protocol::Tor));
        assert_eq!(config.uri_handling.protocols[0].port, 80);
    }

    #[test]
    fn test_singularity_mode() {
        let manager = ProtocolManager::new(None);
        let config_content = r#"
uri_handling_op {
multi_instances = false;
singularity_instances = true;
protocol_def : grpc://0.0.0.0:50051
domain_def : api.example.com
uri_handling_window_appspef : file://server_request/request.manifest
}
        "#;
        
        let result = manager.parse_config(config_content);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert!(!config.uri_handling.multi_instances);
        assert!(config.uri_handling.singularity_instances);
        assert_eq!(config.uri_handling.protocols.len(), 1);
    }
}
use std::net::TcpStream;
use std::io::{Read, ErrorKind};
use crate::protocols::WasmaConfig;

// Gerçek donanım ve paralel işlem kütüphaneleri
use gl; // OpenGL için
use opencl3::command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE};
use opencl3::context::Context;
use opencl3::device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU};
use opencl3::memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_COPY_HOST_PTR};
use rayon::prelude::*; // Intel UHD simülasyonu için

/// WASMA Section Memory: Belleği matematiksel parçalara bölen veya saf bırakan yapı.
pub struct SectionMemory {
    pub raw_storage: Vec<u8>,
    pub cell_count: usize,
    pub cell_size: usize,
}

impl SectionMemory {
    pub fn new(level: u32) -> Self {
        let cell_count = if level == 0 { 1 } else { level as usize };
        let cell_size = 1024 * 1024; // 1MB Sabit hücre boyutu
        let total_size = cell_count * cell_size;

        Self {
            raw_storage: vec![0u8; total_size],
            cell_count,
            cell_size,
        }
    }

    #[inline]
    pub fn get_cell_mut(&mut self, index: usize) -> &mut [u8] {
        let start = index * self.cell_size;
        &mut self.raw_storage[start..start + self.cell_size]
    }

    #[inline]
    pub fn get_cell(&self, index: usize) -> &[u8] {
        let start = index * self.cell_size;
        &self.raw_storage[start..start + self.cell_size]
    }
}

pub struct UClient {
    config: WasmaConfig,
    memory: SectionMemory,
}

impl UClient {
    pub fn new(config: WasmaConfig) -> Self {
        let level = config.resource_limits.scope_level;
        Self {
            memory: SectionMemory::new(level),
            config,
        }
    }

    pub fn start_engine(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let proto = &self.config.uri_handling.protocols[0];
        let addr = format!("{}:{}", proto.ip, proto.port);
        let mut stream = TcpStream::connect(addr)?;
        
        let level = self.config.resource_limits.scope_level;

        println!("🟢 WASMA UClient: Engine Started.");
        println!("📡 Mode: {}", if level == 0 { "NULL_EXCEPTION (Bypass/Raw)" } else { "Partitioned" });

        if level == 0 {
            // --- NULL_EXCEPTION: Saf Ham Akış Modu ---
            // Bellek bölünmez, gelen veri 4KB pencerelerle anında renderer'a paslanır.
            let mut raw_buffer = [0u8; 4096]; 
            loop {
                match stream.read(&mut raw_buffer) {
                    Ok(0) => break,
                    Ok(n) => self.execute_raw_stream(&raw_buffer[..n]),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(Box::new(e)),
                }
            }
        } else {
            // --- Matematiksel Bölümleme Modu ---
            // Veri hücrelere (cell) dolana kadar bekler ve senkronize işler.
            loop {
                for i in 0..self.memory.cell_count {
                    let cell = self.memory.get_cell_mut(i);
                    match stream.read_exact(cell) {
                        Ok(_) => self.execute_renderer(i),
                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                        Err(e) => return Err(Box::new(e)),
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_renderer(&self, cell_index: usize) {
        let cell_data = self.memory.get_cell(cell_index);
        self.dispatch_to_hardware(cell_data);
    }

    fn execute_raw_stream(&self, raw_data: &[u8]) {
        self.dispatch_to_hardware(raw_data);
    }

    fn dispatch_to_hardware(&self, data: &[u8]) {
        match self.config.resource_limits.renderer.as_str() {
            "glx_renderer" => self.run_glx(data),
            "renderer_iuhd" => self.run_iuhd(data),
            "renderer_opencl" => self.run_opencl(data),
            _ => self.run_glx(data),
        }
    }

    // --- Renderer Implementasyonları ---

    fn run_glx(&self, data: &[u8]) {
        unsafe {
            // X11/Wayland bypass: Doğrudan VRAM Texture güncellemesi
            gl::TexSubImage2D(
                gl::TEXTURE_2D, 0, 0, 0,
                1024, 1, // Ham veri genişliği bazlı basım
                gl::RGBA, gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _
            );
        }
    }

    fn run_iuhd(&self, data: &[u8]) {
        // Intel UHD: Rayon ile paralel 4x4 matris işleme
        data.par_chunks(16).for_each(|block| {
            let _avg: u32 = block.iter().map(|&x| x as u32).sum::<u32>() / 16;
            // Matematiksel keskinleştirme algoritmaları burada çalışır
        });
    }

    fn run_opencl(&self, data: &[u8]) {
        // GPGPU: Zero-copy host pointer mapping
        let devices = get_all_devices(CL_DEVICE_TYPE_GPU).unwrap();
        let device = Device::new(devices[0]);
        let context = Context::from_device(&device).unwrap();
        let _buffer = unsafe {
            Buffer::<u8>::create(
                &context,
                CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
                data.len(),
                data.as_ptr() as *mut _,
            ).unwrap()
        };
    }
}
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use x11rb::connection::Connection as XConnection;
use x11rb::protocol::xproto::{self, ConnectionExt};
use wayland_client::Connection as WlConnection;
use crate::protocols::{WasmaConfig, ProtocolType, ProtocolManager};

pub struct WGClient {
    config: Arc<WasmaConfig>,
    x11_ctx: Option<(Arc<x11rb::rust_connection::RustConnection>, xproto::Window)>,
    wl_ctx: Option<WlConnection>,
    is_wasma_native: bool,
}

impl WGClient {
    pub fn new(config: WasmaConfig) -> Self {
        let is_native = config.resource_limits.scope_level > 0;
        let mut x11_ctx = None;
        let mut wl_ctx = None;
        if !is_native {
            if let Ok(conn) = WlConnection::connect_to_env() {
                wl_ctx = Some(conn);
            } else if let Ok((conn, screen_num)) = x11rb::connect(None) {
                let win = conn.generate_id().unwrap();
                let screen = &conn.setup().roots[screen_num];
                conn.create_window(
                    x11rb::COPY_DEPTH_FROM_PARENT, win, screen.root,
                    0, 0, 1280, 720, 0,
                    xproto::WindowClass::INPUT_OUTPUT, 0,
                    &xproto::CreateWindowAux::new().background_pixel(screen.white_pixel)
                ).ok();
                conn.map_window(win).ok();
                conn.flush().ok();
                x11_ctx = Some((Arc::new(conn), win));
            }
        }
        Self {
            config: Arc::new(config),
            x11_ctx,
            wl_ctx,
            is_wasma_native: is_native,
        }
    }

    pub async fn run_engine(&self, mut manager: ProtocolManager) {
        let is_multi = self.config.uri_handling.multi_instances;
        let is_singularity = self.config.uri_handling.singularity_instances;
        let mut stream_count = 0;

        for mut stream in manager.active_streams {
            if is_singularity && stream_count >= 1 { break; }
            let proto_type = stream.get_type();
            
            tokio::spawn(async move {
                match proto_type {
                    ProtocolType::Tor => {
                        let mut buf = [0u8; 65536];
                        while let Ok(n) = stream.read(&mut buf).await {
                            if n == 0 { break; }
                            // Tor: Saf TCP Stream, doğrudan render
                            WGClient::route_to_display(&buf[..n], 0);
                        }
                    },
                    ProtocolType::Grpc => {
                        // gRPC: Protobuf mesajını çöz ve içindeki pikselleri çek
                        while let Ok(Some(frame)) = stream.next_message().await {
                            // gRPC mesaj yapısı: frame.payload (raw image data)
                            WGClient::route_to_display(&frame.payload, 1);
                        }
                    },
                    ProtocolType::Https | ProtocolType::Http => {
                        // HTTP: Chunked veriyi decode et (gzip/br vb. handle edilmiş varsayılır)
                        while let Ok(chunk) = stream.next_chunk().await {
                            WGClient::route_to_display(&chunk, 2);
                        }
                    }
                }
            });
            stream_count += 1;
            if !is_multi { break; }
        }
    }

    fn route_to_display(data: &[u8], stream_id: u8) {
        // Wasma Core modunda mı yoksa Standart OS modunda mıyız?
        if unsafe { WASMA_CORE_ACTIVE } {
            Self::write_raw_vram(data, stream_id);
        } else {
            // Fallback: X11 veya Wayland üzerinden çizim
            // Bu fonksiyonu statik çağırmak için global context veya lazy_static gerekebilir
            // Simülasyon gereği doğrudan render lojiğine paslıyoruz
            Self::execute_fallback_render(data, stream_id);
        }
    }

    fn write_raw_vram(data: &[u8], stream_id: u8) {
        unsafe {
            // Her kanal için 1MB izole bellek alanı (KIP felsefesi)
            let offset = stream_id as usize * (1024 * 1024);
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                (WASMA_VRAM_ADDR + offset) as *mut u8,
                data.len()
            );
        }
    }

fn execute_fallback_render(&self, data: &[u8], stream_id: u8) {
    let width = 1280;
    let height = 240; 
    let y_offset = (stream_id as i16) * (height as i16);

    if let Some((conn, win)) = &self.x11_ctx {
        let gc = conn.generate_id().unwrap();
        conn.create_gc(gc, *win, &xproto::CreateGCAux::new()).ok();
        
        conn.put_image(
            xproto::ImageFormat::Z_PIXMAP,
            *win,
            gc,
            width as u16,
            height as u16,
            0,
            y_offset,
            0,
            24,
            data,
        ).ok();

        conn.free_gc(gc).ok();
        conn.flush().ok();
    } else if let Some(wl_conn) = &self.wl_ctx {
        self.render_wayland_shm(data, stream_id, width, height);
    }
}

fn render_wayland_shm(&self, data: &[u8], stream_id: u8, width: i32, height: i32) {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    use wayland_client::protocol::wl_shm;

    let size = (width * height * 4) as usize;
    let tmp_file = File::create("/dev/shm/wasma_buffer").expect("SHM error");
    tmp_file.set_len(size as u64).ok();
    
    let mmap = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            tmp_file.as_raw_fd(),
            0,
        )
    };

    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), mmap as *mut u8, data.len().min(size));
    }

    if let Some(wl_conn) = &self.wl_ctx {
        // Wayland Surface Attachment ve Commit işlemleri burada wl_conn üzerinden yürütülür
        // wl_surface.attach(buffer, 0, (stream_id as i32) * height);
        // wl_surface.damage(0, 0, width, height);
        // wl_surface.commit();
    }

    unsafe { libc::munmap(mmap, size); }
}
    pub fn write_x11_frame(&self, data: &[u8], stream_id: u8) {
        if let Some((conn, win)) = &self.x11_ctx {
            let gc = conn.generate_id().unwrap();
            conn.create_gc(gc, *win, &xproto::CreateGCAux::new()).ok();
            
            // Multi-instance için dikey tiling (her stream 200px aşağıda başlar)
            let y_pos = (stream_id as i16) * 200;
            
            conn.put_image(
                xproto::ImageFormat::Z_PIXMAP,
                *win,
                gc,
                1280, 200, // Stream başına yükseklik
                0, y_pos, 0, 24, data
            ).ok();
            
            conn.free_gc(gc).ok();
            conn.flush().ok();
        }
    }
}
use std::sync::Arc;
use crate::protocols::{WasmaConfig, ProtocolType};
use crate::window_multitary::WindowMultitary;
use crate::window_singularity::{WindowSingularity, SINGULARITY_LOCK};
use std::sync::atomic::Ordering;

pub struct WindowClient {
    config: Arc<WasmaConfig>,
    multitary: WindowMultitary,
    singularity: WindowSingularity,
    width: u32,
    height: u32,
}

impl WindowClient {
    pub fn new(config: WasmaConfig, width: u32, height: u32) -> Self {
        let config_arc = Arc::new(config);
        Self {
            config: config_arc.clone(),
            multitary: WindowMultitary::new((*config_arc).clone(), width, height),
            singularity: WindowSingularity::new((*config_arc).clone(), width, height),
            width,
            height,
        }
    }

    pub fn render_frame(&self, stream_id: u8, data: &[u8]) {
        let is_singularity = SINGULARITY_LOCK.load(Ordering::SeqCst);
        
        if is_singularity {
            let bounds = self.singularity.get_exclusive_bounds();
            self.dispatch_to_hardware(data, bounds, stream_id);
        } else {
            if let Some(viewport) = self.multitary.get_viewport_for_stream(stream_id) {
                if viewport.active {
                    let bounds = (viewport.x, viewport.y, viewport.width, viewport.height);
                    self.dispatch_to_hardware(data, bounds, stream_id);
                }
            }
        }
    }

    fn dispatch_to_hardware(&self, data: &[u8], bounds: (i32, i32, u32, u32), stream_id: u8) {
        if self.config.resource_limits.scope_level > 0 {
            self.blit_native_vram(data, bounds, stream_id);
        } else {
            self.blit_os_fallback(data, bounds, stream_id);
        }
    }

    fn blit_native_vram(&self, data: &[u8], bounds: (i32, i32, u32, u32), stream_id: u8) {
        unsafe {
            let (x, y, w, h) = bounds;
            let offset = (y as usize * self.width as usize + x as usize) * 4;
            let target_ptr = (crate::wgclient::WASMA_VRAM_ADDR as *mut u8).add(offset);
            std::ptr::copy_nonoverlapping(data.as_ptr(), target_ptr, data.len());
        }
    }

    fn blit_os_fallback(&self, data: &[u8], bounds: (i32, i32, u32, u32), _stream_id: u8) {
        let (x, y, w, h) = bounds;
        // X11 PutImage or Wayland Subsurface commit using calculated x, y, w, h
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
        self.multitary.update_resolution(new_width, new_height);
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};
use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, row, text, scrollable, Space},
    alignment, executor, window, Length, Color, Background,
};
use iced::window::{Id as WindowId, Position};

/// Window state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

/// Window type classification
#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    Normal,
    Dialog,
    Utility,
    Splash,
    Menu,
    Dropdown,
    Popup,
    Tooltip,
    Notification,
}

/// Resource limits per window - WBackend integration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f32>,
    pub max_gpu_memory_mb: Option<u64>,
    pub pixel_load_limit: Option<u32>,
    pub content_size_limit: Option<u64>,
    pub renderer: String,
    pub execution_mode: ExecutionMode,
    pub cpu_cores: Vec<usize>,
    pub lease_duration_secs: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(512),
            max_cpu_percent: Some(50.0),
            max_gpu_memory_mb: Some(256),
            pixel_load_limit: Some(50),
            content_size_limit: Some(1024 * 1024 * 10),
            renderer: "cpu_renderer".to_string(),
            execution_mode: ExecutionMode::GpuPreferred,
            cpu_cores: Vec::new(),
            lease_duration_secs: Some(30),
        }
    }
}

/// Window geometry and positioning
#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Permission scope for window operations
#[derive(Debug, Clone)]
pub struct PermissionScope {
    pub can_access_network: bool,
    pub can_access_filesystem: bool,
    pub can_spawn_children: bool,
    pub can_use_gpu: bool,
    pub allowed_protocols: Vec<String>,
    pub sandbox_level: u8,
}

impl Default for PermissionScope {
    fn default() -> Self {
        Self {
            can_access_network: false,
            can_access_filesystem: false,
            can_spawn_children: false,
            can_use_gpu: true,
            allowed_protocols: vec!["http".to_string(), "https".to_string()],
            sandbox_level: 5,
        }
    }
}

/// Core window structure with WBackend and Iced integration
#[derive(Debug, Clone)]
pub struct Window {
    pub id: u64,
    pub iced_window_id: Option<WindowId>,
    pub title: String,
    pub app_id: String,
    pub state: WindowState,
    pub window_type: WindowType,
    pub geometry: WindowGeometry,
    pub parent_id: Option<u64>,
    pub children_ids: Vec<u64>,
    pub visible: bool,
    pub focused: bool,
    pub resource_limits: ResourceLimits,
    pub permissions: PermissionScope,
    pub manifest_path: Option<String>,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub backend_type: BackendType,
    pub assignment_id: Option<u32>,
    pub resource_mode: ResourceMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    Native,
    Wayland,
    X11,
    Remote(String),
}

/// Window handling operations with WBackend and Iced rendering
pub struct WindowHandler {
    windows: Arc<Mutex<HashMap<u64, Window>>>,
    next_id: Arc<Mutex<u64>>,
    focused_window: Arc<Mutex<Option<u64>>>,
    wbackend: Arc<WBackend>,
    assignment_to_window: Arc<Mutex<HashMap<u32, u64>>>,
}

impl WindowHandler {
    pub fn new(resource_mode: ResourceMode) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            focused_window: Arc::new(Mutex::new(None)),
            wbackend: Arc::new(WBackend::new(resource_mode)),
            assignment_to_window: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_window(
        &self,
        title: String,
        app_id: String,
        geometry: WindowGeometry,
        manifest_path: Option<String>,
        resource_mode: ResourceMode,
    ) -> Result<u64, String> {
        let mut next_id = self.next_id.lock().unwrap();
        let window_id = *next_id;
        *next_id += 1;

        let (mut resource_limits, permissions) = if let Some(ref path) = manifest_path {
            self.load_manifest_config(path)?
        } else {
            (ResourceLimits::default(), PermissionScope::default())
        };

        let assignment_id = window_id as u32;
        let mut assignment = Assignment::new(assignment_id);
        
        assignment.execution_mode = resource_limits.execution_mode;
        assignment.ram_limit = (resource_limits.max_memory_mb.unwrap_or(512) * 1024 * 1024) as usize;
        assignment.vram_limit = (resource_limits.max_gpu_memory_mb.unwrap_or(256) * 1024 * 1024) as usize;
        
        if !resource_limits.cpu_cores.is_empty() {
            assignment.cpu_cores = resource_limits.cpu_cores.clone();
        }
        
        if let Some(lease_secs) = resource_limits.lease_duration_secs {
            assignment.start_lease(Duration::from_secs(lease_secs));
        }

        self.wbackend.add_assignment(assignment);

        let mut mapping = self.assignment_to_window.lock().unwrap();
        mapping.insert(assignment_id, window_id);

        let window = Window {
            id: window_id,
            iced_window_id: None, // Will be set when Iced window is spawned
            title,
            app_id,
            state: WindowState::Normal,
            window_type: WindowType::Normal,
            geometry,
            parent_id: None,
            children_ids: Vec::new(),
            visible: true,
            focused: false,
            resource_limits,
            permissions,
            manifest_path,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            backend_type: BackendType::Native,
            assignment_id: Some(assignment_id),
            resource_mode,
        };

        let mut windows = self.windows.lock().unwrap();
        windows.insert(window_id, window);

        println!(
            "🪟 Window {} created | Assignment {} | Mode: {:?}",
            window_id, assignment_id, resource_mode
        );

        Ok(window_id)
    }

    pub fn run_resource_cycle(&self) {
        self.wbackend.run_cycle();
    }

    pub fn adjust_window_resources(
        &self,
        window_id: u64,
        new_limits: ResourceLimits,
    ) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        let window = windows.get_mut(&window_id)
            .ok_or_else(|| format!("Window {} not found", window_id))?;

        if window.resource_mode != ResourceMode::Manual {
            return Err(format!(
                "Window {} is in Auto mode, cannot manually adjust resources",
                window_id
            ));
        }

        window.resource_limits = new_limits.clone();

        if let Some(assignment_id) = window.assignment_id {
            if let Some(mut assignment) = self.wbackend.get_assignment(assignment_id) {
                assignment.execution_mode = new_limits.execution_mode;
                assignment.ram_limit = (new_limits.max_memory_mb.unwrap_or(512) * 1024 * 1024) as usize;
                assignment.vram_limit = (new_limits.max_gpu_memory_mb.unwrap_or(256) * 1024 * 1024) as usize;
                
                if !new_limits.cpu_cores.is_empty() {
                    assignment.cpu_cores = new_limits.cpu_cores.clone();
                    assignment.bind_cpu();
                }

                if assignment.should_bind_gpu() {
                    assignment.bind_gpu();
                }

                println!(
                    "🔧 Window {} resources adjusted | RAM: {} MiB | VRAM: {} MiB",
                    window_id,
                    new_limits.max_memory_mb.unwrap_or(0),
                    new_limits.max_gpu_memory_mb.unwrap_or(0)
                );
            }
        }

        Ok(())
    }

    pub fn get_window_resource_usage(&self, window_id: u64) -> Result<ResourceUsage, String> {
        let windows = self.windows.lock().unwrap();
        let window = windows.get(&window_id)
            .ok_or_else(|| format!("Window {} not found", window_id))?;

        if let Some(assignment_id) = window.assignment_id {
            if let Some(assignment) = self.wbackend.get_assignment(assignment_id) {
                let gpu_active = assignment.gpu_device.is_some();
                let task_running = assignment.task_handle.is_some() 
                    && *assignment.task_active.lock().unwrap();

                let remaining_lease = assignment.lease_duration
                    .and_then(|d| assignment.lease_start.map(|s| {
                        d.as_secs().saturating_sub(s.elapsed().as_secs())
                    }))
                    .unwrap_or(0);

                return Ok(ResourceUsage {
                    window_id,
                    assignment_id,
                    ram_allocated_mb: (assignment.ram_limit / (1024 * 1024)) as u64,
                    vram_allocated_mb: (assignment.vram_limit / (1024 * 1024)) as u64,
                    cpu_cores: assignment.cpu_cores.clone(),
                    gpu_device: assignment.gpu_device.clone(),
                    task_active: task_running,
                    gpu_active,
                    remaining_lease_secs: remaining_lease,
                    execution_mode: assignment.execution_mode,
                });
            }
        }

        Err(format!("No assignment found for window {}", window_id))
    }

    pub fn set_window_state(&self, id: u64, state: WindowState) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.get_mut(&id) {
            window.state = state;
            window.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(format!("Window {} not found", id))
        }
    }

    pub fn set_geometry(&self, id: u64, geometry: WindowGeometry) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.get_mut(&id) {
            window.geometry = geometry;
            window.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(format!("Window {} not found", id))
        }
    }

    pub fn focus_window(&self, id: u64) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        
        for window in windows.values_mut() {
            window.focused = false;
        }

        if let Some(window) = windows.get_mut(&id) {
            window.focused = true;
            window.last_activity = SystemTime::now();
            drop(windows);
            
            let mut focused = self.focused_window.lock().unwrap();
            *focused = Some(id);
            Ok(())
        } else {
            Err(format!("Window {} not found", id))
        }
    }

    pub fn close_window(&self, id: u64) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        
        if let Some(window) = windows.get(&id) {
            let children = window.children_ids.clone();
            let assignment_id = window.assignment_id;
            
            for child_id in children {
                if let Some(child) = windows.get(&child_id) {
                    if let Some(child_assignment_id) = child.assignment_id {
                        if let Some(mut assignment) = self.wbackend.get_assignment(child_assignment_id) {
                            assignment.stop_task();
                        }
                    }
                }
                windows.remove(&child_id);
            }
            
            if let Some(parent_id) = window.parent_id {
                if let Some(parent) = windows.get_mut(&parent_id) {
                    parent.children_ids.retain(|&cid| cid != id);
                }
            }
            
            if let Some(aid) = assignment_id {
                if let Some(mut assignment) = self.wbackend.get_assignment(aid) {
                    assignment.stop_task();
                    println!("🛑 Window {} assignment {} stopped", id, aid);
                }
                
                let mut mapping = self.assignment_to_window.lock().unwrap();
                mapping.remove(&aid);
            }
            
            windows.remove(&id);
            println!("🗑️  Window {} closed", id);
            Ok(())
        } else {
            Err(format!("Window {} not found", id))
        }
    }

    pub fn set_parent(&self, child_id: u64, parent_id: u64) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        
        if !windows.contains_key(&parent_id) {
            return Err(format!("Parent window {} not found", parent_id));
        }
        
        if let Some(child) = windows.get_mut(&child_id) {
            child.parent_id = Some(parent_id);
        } else {
            return Err(format!("Child window {} not found", child_id));
        }
        
        if let Some(parent) = windows.get_mut(&parent_id) {
            if !parent.children_ids.contains(&child_id) {
                parent.children_ids.push(child_id);
            }
        }
        
        Ok(())
    }

    pub fn get_window(&self, id: u64) -> Option<Window> {
        let windows = self.windows.lock().unwrap();
        windows.get(&id).cloned()
    }

    pub fn list_windows(&self) -> Vec<Window> {
        let windows = self.windows.lock().unwrap();
        windows.values().cloned().collect()
    }

    pub fn get_focused_window(&self) -> Option<u64> {
        let focused = self.focused_window.lock().unwrap();
        *focused
    }

    fn load_manifest_config(&self, path: &str) -> Result<(ResourceLimits, PermissionScope), String> {
        Ok((ResourceLimits::default(), PermissionScope::default()))
    }
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub window_id: u64,
    pub assignment_id: u32,
    pub ram_allocated_mb: u64,
    pub vram_allocated_mb: u64,
    pub cpu_cores: Vec<usize>,
    pub gpu_device: Option<String>,
    pub task_active: bool,
    pub gpu_active: bool,
    pub remaining_lease_secs: u64,
    pub execution_mode: ExecutionMode,
}

// ============================================================================
// ICED INTEGRATION - WASMA Window Manager UI
// ============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    CreateWindow,
    CloseWindow(u64),
    FocusWindow(u64),
    MinimizeWindow(u64),
    MaximizeWindow(u64),
    UpdateResourceCycle,
    AdjustResources(u64),
    ChangeExecutionMode(u64, ExecutionMode),
}

pub struct WasmaWindowManager {
    handler: Arc<WindowHandler>,
    selected_window: Option<u64>,
}

impl Application for WasmaWindowManager {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ResourceMode;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let handler = Arc::new(WindowHandler::new(flags));
        
        (
            WasmaWindowManager {
                handler,
                selected_window: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("WASMA - Window Assignment System Monitoring Architecture")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CreateWindow => {
                let geometry = WindowGeometry {
                    x: 100,
                    y: 100,
                    width: 800,
                    height: 600,
                };
                
                match self.handler.create_window(
                    format!("WASMA Window {}", self.handler.list_windows().len() + 1),
                    "wasma.window".to_string(),
                    geometry,
                    None,
                    ResourceMode::Auto,
                ) {
                    Ok(id) => {
                        println!("✅ Window {} created", id);
                        Command::none()
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create window: {}", e);
                        Command::none()
                    }
                }
            }
            
            Message::CloseWindow(id) => {
                if let Err(e) = self.handler.close_window(id) {
                    eprintln!("❌ Failed to close window {}: {}", id, e);
                }
                if self.selected_window == Some(id) {
                    self.selected_window = None;
                }
                Command::none()
            }
            
            Message::FocusWindow(id) => {
                if let Err(e) = self.handler.focus_window(id) {
                    eprintln!("❌ Failed to focus window {}: {}", id, e);
                }
                self.selected_window = Some(id);
                Command::none()
            }
            
            Message::MinimizeWindow(id) => {
                if let Err(e) = self.handler.set_window_state(id, WindowState::Minimized) {
                    eprintln!("❌ Failed to minimize window {}: {}", id, e);
                }
                Command::none()
            }
            
            Message::MaximizeWindow(id) => {
                if let Err(e) = self.handler.set_window_state(id, WindowState::Maximized) {
                    eprintln!("❌ Failed to maximize window {}: {}", id, e);
                }
                Command::none()
            }
            
            Message::UpdateResourceCycle => {
                self.handler.run_resource_cycle();
                Command::none()
            }
            
            Message::AdjustResources(id) => {
                // Implement resource adjustment dialog
                println!("🔧 Adjust resources for window {}", id);
                Command::none()
            }
            
            Message::ChangeExecutionMode(id, mode) => {
                if let Some(window) = self.handler.get_window(id) {
                    let mut new_limits = window.resource_limits.clone();
                    new_limits.execution_mode = mode;
                    if let Err(e) = self.handler.adjust_window_resources(id, new_limits) {
                        eprintln!("❌ Failed to change execution mode: {}", e);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let windows = self.handler.list_windows();
        
        let header = row![
            text("WASMA Window Manager")
                .size(24)
                .style(Color::from_rgb(0.2, 0.6, 1.0)),
            Space::with_width(Length::Fill),
            button("+ New Window").on_press(Message::CreateWindow),
            Space::with_width(10),
            button("⟳ Update Resources").on_press(Message::UpdateResourceCycle),
        ]
        .padding(20)
        .spacing(10);

        let mut window_list = column![].spacing(10).padding(20);

        if windows.is_empty() {
            window_list = window_list.push(
                text("No active windows. Click 'New Window' to create one.")
                    .size(16)
                    .style(Color::from_rgb(0.5, 0.5, 0.5))
            );
        } else {
            for window in windows {
                let is_selected = self.selected_window == Some(window.id);
                let window_card = self.create_window_card(&window, is_selected);
                window_list = window_list.push(window_card);
            }
        }

        let content = column![
            header,
            scrollable(window_list)
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl WasmaWindowManager {
    fn create_window_card(&self, window: &Window, is_selected: bool) -> Element<Message> {
        let state_icon = match window.state {
            WindowState::Normal => "🟢",
            WindowState::Minimized => "🟡",
            WindowState::Maximized => "🔵",
            WindowState::Fullscreen => "⚡",
            WindowState::Hidden => "⚫",
        };

        let focus_indicator = if window.focused { "👁️ " } else { "" };

        let title_row = row![
            text(format!("{}{} {}", focus_indicator, state_icon, window.title))
                .size(18),
            Space::with_width(Length::Fill),
            button("Focus").on_press(Message::FocusWindow(window.id)),
            Space::with_width(5),
            button("Minimize").on_press(Message::MinimizeWindow(window.id)),
            Space::with_width(5),
            button("Maximize").on_press(Message::MaximizeWindow(window.id)),
            Space::with_width(5),
            button("✕").on_press(Message::CloseWindow(window.id)),
        ]
        .spacing(5);

        let info = if let Ok(usage) = self.handler.get_window_resource_usage(window.id) {
            let mode_str = match usage.execution_mode {
                ExecutionMode::CpuOnly => "🔵 CPU-Only",
                ExecutionMode::GpuPreferred => "🟢 GPU Preferred",
                ExecutionMode::GpuOnly => "🟡 GPU-Only",
                ExecutionMode::Hybrid => "⚡ Hybrid",
            };

            column![
                text(format!("ID: {} | Assignment: {}", window.id, usage.assignment_id))
                    .size(14),
                text(format!(
                    "{} | Status: {}",
                    mode_str,
                    if usage.task_active { "RUNNING" } else { "STOPPED" }
                ))
                .size(14),
                text(format!(
                    "RAM: {} MiB | VRAM: {} MiB | Cores: {:?}",
                    usage.ram_allocated_mb, usage.vram_allocated_mb, usage.cpu_cores
                ))
                .size(14),
                text(format!(
                    "GPU: {} | Lease: {}s",
                    usage.gpu_device.unwrap_or_else(|| "None".to_string()),
                    usage.remaining_lease_secs
                ))
                .size(14),
                text(format!("Renderer: {} | {}x{}", 
                    window.resource_limits.renderer,
                    window.geometry.width,
                    window.geometry.height
                ))
                .size(14),
            ]
            .spacing(5)
        } else {
            column![text("Resource info unavailable").size(14)]
        };

        let card_content = column![title_row, info].spacing(10).padding(15);

        let card_background = if is_selected {
            Background::Color(Color::from_rgb(0.2, 0.3, 0.4))
        } else {
            Background::Color(Color::from_rgb(0.15, 0.15, 0.15))
        };

        container(card_content)
            .width(Length::Fill)
            .style(move |_theme: &Theme| {
                container::Appearance {
                    background: Some(card_background),
                    border: iced::Border {
                        color: if is_selected {
                            Color::from_rgb(0.3, 0.6, 1.0)
                        } else {
                            Color::from_rgb(0.3, 0.3, 0.3)
                        },
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

/// Launch WASMA Window Manager with Iced
pub fn launch_window_manager(resource_mode: ResourceMode) -> iced::Result {
    WasmaWindowManager::run(Settings {
        window: window::Settings {
            size: (1200, 800).into(),
            position: Position::Centered,
            ..Default::default()
        },
        flags: resource_mode,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let handler = WindowHandler::new(ResourceMode::Auto);
        let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
        let id = handler.create_window(
            "Test".to_string(),
            "test.app".to_string(),
            geometry,
            None,
            ResourceMode::Auto,
        ).unwrap();
        
        assert!(handler.get_window(id).is_some());
    }
}
use std::collections::HashMap;
use crate::protocols::{WasmaConfig, ProtocolType};

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_index: u8,
    pub active: bool,
}

pub struct WindowMultitary {
    config: WasmaConfig,
    pub viewports: HashMap<u8, Viewport>, // stream_id -> Viewport
    screen_width: u32,
    screen_height: u32,
}

impl WindowMultitary {
    pub fn new(config: WasmaConfig, screen_width: u32, screen_height: u32) -> Self {
        let mut multitary = Self {
            config,
            viewports: HashMap::new(),
            screen_width,
            screen_height,
        };
        multitary.calculate_layouts();
        multitary
    }

    pub fn calculate_layouts(&mut self) {
        let proto_count = self.config.uri_handling.protocols.len();
        let is_singularity = self.config.uri_handling.singularity_instances;
        let is_multi = self.config.uri_handling.multi_instances;

        if is_singularity && !is_multi {
            // Singularity: Tek pencere, tam ekran
            self.viewports.insert(0, Viewport {
                x: 0, y: 0,
                width: self.screen_width,
                height: self.screen_height,
                z_index: 1,
                active: true,
            });
        } else if is_multi {
            // Multi-Instance: Ekranı protokol sayısına göre dikey böl (Tiling)
            let section_height = self.screen_height / (proto_count as u32);
            for i in 0..proto_count {
                self.viewports.insert(i as u8, Viewport {
                    x: 0,
                    y: (i as i32) * (section_height as i32),
                    width: self.screen_width,
                    height: section_height,
                    z_index: 1,
                    active: true,
                });
            }
        }
    }

    pub fn get_viewport_for_stream(&self, stream_id: u8) -> Option<&Viewport> {
        self.viewports.get(&stream_id)
    }

    pub fn update_resolution(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.calculate_layouts();
    }

    pub fn handle_input_focus(&self, x: i32, y: i32) -> Option<u8> {
        // Fare koordinatına göre hangi protokolün (stream_id) aktif olduğunu bulur
        for (id, vp) in &self.viewports {
            if x >= vp.x && x <= (vp.x + vp.width as i32) &&
               y >= vp.y && y <= (vp.y + vp.height as i32) {
                return Some(*id);
            }
        }
        None
    }
}
use core_affinity::{get_core_ids, set_for_current};
use libc::{sched_param, pthread_setschedparam, SCHED_FIFO};
use crate::protocols::WasmaConfig;

pub struct ResourcerEngineering {
    config: WasmaConfig,
}

impl ResourcerEngineering {
    pub fn new(config: WasmaConfig) -> Self {
        Self { config }
    }

    pub fn apply_hardware_affinity(&self) {
        let is_nonprof = false;
        if is_nonprof {
            self.apply_dynamic_allocation();
        } else {
            self.apply_static_allocation();
        }
        self.set_realtime_priority();
    }

    fn apply_static_allocation(&self) {
        let core_ids = get_core_ids().expect("Cores missing");
        let static_core_count = 5;
        for i in 0..static_core_count {
            if i < core_ids.len() {
                set_for_current(core_ids[i]);
            }
        }
        if self.config.resource_limits.scope_level > 0 {
            self.init_gpu_context(2048);
        }
    }

    fn apply_dynamic_allocation(&self) {
        let core_ids = get_core_ids().expect("Cores missing");
        if let Some(core) = core_ids.first() {
            set_for_current(*core);
        }
    }

    fn init_gpu_context(&self, vram_mb: u64) {
        unsafe {
            let level = self.config.resource_limits.scope_level;
            self.enable_glx_pipeline(level, vram_mb);
        }
    }

    fn enable_glx_pipeline(&self, level: u32, vram_mb: u64) {
        let _allocation_size = vram_mb * 1024 * 1024;
        let _scale = level as f32 / 100.0;
    }

    fn set_realtime_priority(&self) {
        unsafe {
            let param = sched_param { sched_priority: 99 };
            let thread_id = libc::pthread_self();
            pthread_setschedparam(thread_id, SCHED_FIFO, &param);
        }
    }

    pub fn isolate_task_by_config(&self) {
        let core_ids = get_core_ids().unwrap();
        if core_ids.len() > 2 {
            set_for_current(core_ids[2]);
        }
    }
}
use crate::protocols::{WasmaConfig, ProtocolType};
use std::sync::atomic::{AtomicBool, Ordering};

pub static SINGULARITY_LOCK: AtomicBool = AtomicBool::new(false);

pub struct WindowSingularity {
    config: WasmaConfig,
    target_id: u8,
    screen_width: u32,
    screen_height: u32,
}

impl WindowSingularity {
    pub fn new(config: WasmaConfig, screen_width: u32, screen_height: u32) -> Self {
        Self {
            config,
            target_id: 0,
            screen_width,
            screen_height,
        }
    }

    pub fn enter_singularity_mode(&mut self, stream_id: u8) {
        SINGULARITY_LOCK.store(true, Ordering::SeqCst);
        self.target_id = stream_id;
        self.enforce_exclusive_resource();
    }

    fn enforce_exclusive_resource(&self) {
        if self.config.resource_limits.scope_level > 0 {
            unsafe {
                let vram_ptr = crate::wgclient::WASMA_VRAM_ADDR as *mut u8;
                std::ptr::write_bytes(vram_ptr, 0, (self.screen_width * self.screen_height * 4) as usize);
            }
        }
    }

    pub fn get_exclusive_bounds(&self) -> (i32, i32, u32, u32) {
        (0, 0, self.screen_width, self.screen_height)
    }

    pub fn validate_protocol_focus(&self, proto: ProtocolType) -> bool {
        let active_proto = &self.config.uri_handling.protocols[self.target_id as usize].protocol;
        active_proto == &proto
    }

    pub fn exit_singularity_mode(&mut self) {
        SINGULARITY_LOCK.store(false, Ordering::SeqCst);
    }

    pub fn apply_hardware_acceleration(&self) {
        if let Some(proto) = self.config.uri_handling.protocols.get(self.target_id as usize) {
            match proto.protocol {
                ProtocolType::Grpc => {
                    // gRPC/HTTP2 HPACK compression optimization
                },
                ProtocolType::Tor => {
                    // Tor cell-buffer prioritizing
                },
                _ => {}
            }
        }
    }
}

// Re-export parser types for backward compatibility
pub use crate::parser::{
    Protocol, ProtocolConfig, WasmaConfig, UriHandlingConfig,
    CompilationServer, UserConfig, ResourceLimits, ParserError,
    ConfigParser,
};

use std::net::TcpStream;
use std::sync::Arc;

/// Protocol manager for handling active connections
pub struct ProtocolManager {
    config: Arc<WasmaConfig>,
    active_streams: Vec<Box<dyn ProtocolStream>>,
}

impl ProtocolManager {
    pub fn new(config: WasmaConfig) -> Self {
        Self {
            config: Arc::new(config),
            active_streams: Vec::new(),
        }
    }

    pub fn from_config(config: Arc<WasmaConfig>) -> Self {
        Self {
            config,
            active_streams: Vec::new(),
        }
    }

    /// Connect to all configured protocols
    pub fn connect_all(&mut self) -> Result<(), String> {
        for proto_config in &self.config.uri_handling.protocols {
            match self.connect_protocol(proto_config) {
                Ok(stream) => {
                    self.active_streams.push(stream);
                    println!("✅ Connected to {:?} at {}:{}", 
                        proto_config.protocol,
                        proto_config.ip,
                        proto_config.port
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect to {:?}: {}", 
                        proto_config.protocol, e
                    );
                }
            }
        }
        Ok(())
    }

    fn connect_protocol(&self, config: &ProtocolConfig) -> Result<Box<dyn ProtocolStream>, String> {
        let addr = format!("{}:{}", config.ip, config.port);
        
        match config.protocol {
            Protocol::Http | Protocol::Https => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("TCP connection failed: {}", e))?;
                Ok(Box::new(HttpStream::new(stream, config.protocol == Protocol::Https)))
            }
            Protocol::Grpc => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("gRPC connection failed: {}", e))?;
                Ok(Box::new(GrpcStream::new(stream)))
            }
            Protocol::Tor => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("Tor connection failed: {}", e))?;
                Ok(Box::new(TorStream::new(stream)))
            }
        }
    }

    pub fn get_config(&self) -> &WasmaConfig {
        &self.config
    }

    pub fn is_multi_instance(&self) -> bool {
        self.config.uri_handling.multi_instances
    }

    pub fn is_singularity(&self) -> bool {
        self.config.uri_handling.singularity_instances
    }

    pub fn get_active_stream_count(&self) -> usize {
        self.active_streams.len()
    }
}

/// Protocol stream trait for unified handling
pub trait ProtocolStream: Send {
    fn get_type(&self) -> Protocol;
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

struct HttpStream {
    stream: TcpStream,
    is_https: bool,
}

impl HttpStream {
    fn new(stream: TcpStream, is_https: bool) -> Self {
        Self { stream, is_https }
    }
}

impl ProtocolStream for HttpStream {
    fn get_type(&self) -> Protocol {
        if self.is_https {
            Protocol::Https
        } else {
            Protocol::Http
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

struct GrpcStream {
    stream: TcpStream,
}

impl GrpcStream {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl ProtocolStream for GrpcStream {
    fn get_type(&self) -> Protocol {
        Protocol::Grpc
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

struct TorStream {
    stream: TcpStream,
}

impl TorStream {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl ProtocolStream for TorStream {
    fn get_type(&self) -> Protocol {
        Protocol::Tor
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_manager_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let manager = ProtocolManager::new(config);
        assert!(manager.is_singularity());
        assert!(!manager.is_multi_instance());
    }
}// Re-export parser types for backward compatibility
pub use crate::parser::{
    Protocol, ProtocolConfig, WasmaConfig, UriHandlingConfig,
    CompilationServer, UserConfig, ResourceLimits, ParserError,
    ConfigParser,
};

use std::net::TcpStream;
use std::sync::Arc;

/// Protocol manager for handling active connections
pub struct ProtocolManager {
    config: Arc<WasmaConfig>,
    active_streams: Vec<Box<dyn ProtocolStream>>,
}

impl ProtocolManager {
    pub fn new(config: WasmaConfig) -> Self {
        Self {
            config: Arc::new(config),
            active_streams: Vec::new(),
        }
    }

    pub fn from_config(config: Arc<WasmaConfig>) -> Self {
        Self {
            config,
            active_streams: Vec::new(),
        }
    }

    /// Connect to all configured protocols
    pub fn connect_all(&mut self) -> Result<(), String> {
        for proto_config in &self.config.uri_handling.protocols {
            match self.connect_protocol(proto_config) {
                Ok(stream) => {
                    self.active_streams.push(stream);
                    println!("✅ Connected to {:?} at {}:{}", 
                        proto_config.protocol,
                        proto_config.ip,
                        proto_config.port
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect to {:?}: {}", 
                        proto_config.protocol, e
                    );
                }
            }
        }
        Ok(())
    }

    fn connect_protocol(&self, config: &ProtocolConfig) -> Result<Box<dyn ProtocolStream>, String> {
        let addr = format!("{}:{}", config.ip, config.port);
        
        match config.protocol {
            Protocol::Http | Protocol::Https => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("TCP connection failed: {}", e))?;
                Ok(Box::new(HttpStream::new(stream, config.protocol == Protocol::Https)))
            }
            Protocol::Grpc => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("gRPC connection failed: {}", e))?;
                Ok(Box::new(GrpcStream::new(stream)))
            }
            Protocol::Tor => {
                let stream = TcpStream::connect(&addr)
                    .map_err(|e| format!("Tor connection failed: {}", e))?;
                Ok(Box::new(TorStream::new(stream)))
            }
        }
    }

    pub fn get_config(&self) -> &WasmaConfig {
        &self.config
    }

    pub fn is_multi_instance(&self) -> bool {
        self.config.uri_handling.multi_instances
    }

    pub fn is_singularity(&self) -> bool {
        self.config.uri_handling.singularity_instances
    }

    pub fn get_active_stream_count(&self) -> usize {
        self.active_streams.len()
    }
}

/// Protocol stream trait for unified handling
pub trait ProtocolStream: Send {
    fn get_type(&self) -> Protocol;
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

struct HttpStream {
    stream: TcpStream,
    is_https: bool,
}

impl HttpStream {
    fn new(stream: TcpStream, is_https: bool) -> Self {
        Self { stream, is_https }
    }
}

impl ProtocolStream for HttpStream {
    fn get_type(&self) -> Protocol {
        if self.is_https {
            Protocol::Https
        } else {
            Protocol::Http
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

struct GrpcStream {
    stream: TcpStream,
}

impl GrpcStream {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl ProtocolStream for GrpcStream {
    fn get_type(&self) -> Protocol {
        Protocol::Grpc
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

struct TorStream {
    stream: TcpStream,
}

impl TorStream {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl ProtocolStream for TorStream {
    fn get_type(&self) -> Protocol {
        Protocol::Tor
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::Write;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_manager_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let manager = ProtocolManager::new(config);
        assert!(manager.is_singularity());
        assert!(!manager.is_multi_instance());
    }
}

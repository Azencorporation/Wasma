// protocols.rs
use crate::parser::{ConfigParser, ParserError, Protocol, ProtocolConfig, WasmaConfig};
use std::sync::Arc;
use std::net::TcpStream;
use std::io::{Read, Write};

/// Protocol stream trait - async trait olarak düzeltildi
#[async_trait::async_trait]
pub trait ProtocolStream: Send {
    fn get_type(&self) -> Protocol;
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    async fn flush(&mut self) -> std::io::Result<()>;
    
    async fn next_message(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        let mut buf = vec![0u8; 65536];
        match self.read(&mut buf).await {
            Ok(0) => Ok(None),
            Ok(n) => Ok(Some(buf[..n].to_vec())),
            Err(e) => Err(e),
        }
    }
    
    async fn next_chunk(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; 8192];
        let n = self.read(&mut buf).await?;
        buf.truncate(n);
        Ok(buf)
    }
}

/// Protocol Manager - Network bağlantı yönetimi
pub struct ProtocolManager {
    config: Arc<WasmaConfig>,
    pub active_streams: Vec<Box<dyn ProtocolStream>>,
}

impl ProtocolManager {
    pub fn new(config_path: Option<String>) -> Result<Self, ParserError> {
        let parser = ConfigParser::new(config_path);
        let config = parser.load()?;
        
        Ok(Self {
            config: Arc::new(config),
            active_streams: Vec::new(),
        })
    }

    pub fn from_config(config: Arc<WasmaConfig>) -> Self {
        Self {
            config,
            active_streams: Vec::new(),
        }
    }

    pub fn load_config(&mut self) -> Result<(), ParserError> {
        // Config zaten new()'de yüklendi, sadece validate et
        self.validate()
    }

    pub fn validate(&self) -> Result<(), ParserError> {
        let parser = ConfigParser::new(None);
        parser.validate(&self.config)
    }

    pub fn get_config(&self) -> Option<&WasmaConfig> {
        Some(&self.config)
    }

    /// Tüm protokollere bağlan
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

    pub fn is_multi_instance(&self) -> bool {
        self.config.uri_handling.multi_instances
    }

    pub fn is_singularity(&self) -> bool {
        self.config.uri_handling.singularity_instances
    }

    pub fn get_active_stream_count(&self) -> usize {
        self.active_streams.len()
    }
    
    pub fn get_streams_mut(&mut self) -> &mut Vec<Box<dyn ProtocolStream>> {
        &mut self.active_streams
    }
}

// HTTP/HTTPS Stream
struct HttpStream {
    stream: TcpStream,
    is_https: bool,
}

impl HttpStream {
    fn new(stream: TcpStream, is_https: bool) -> Self {
        stream.set_nonblocking(true).ok();
        Self { stream, is_https }
    }
}

#[async_trait::async_trait]
impl ProtocolStream for HttpStream {
    fn get_type(&self) -> Protocol {
        if self.is_https { Protocol::Https } else { Protocol::Http }
    }

    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

// gRPC Stream
struct GrpcStream {
    stream: TcpStream,
}

impl GrpcStream {
    fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).ok();
        Self { stream }
    }
}

#[async_trait::async_trait]
impl ProtocolStream for GrpcStream {
    fn get_type(&self) -> Protocol {
        Protocol::Grpc
    }

    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

// Tor Stream
struct TorStream {
    stream: TcpStream,
}

impl TorStream {
    fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).ok();
        Self { stream }
    }
}

#[async_trait::async_trait]
impl ProtocolStream for TorStream {
    fn get_type(&self) -> Protocol {
        Protocol::Tor
    }

    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }

    async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

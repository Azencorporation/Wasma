// WSDG GHX Open - URI and Protocol Handler
// Opens applications and all URIs with protocol support
// Applications must support protocols through in-array env
// Can be fed from any URI text
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::collections::HashMap;
use std::process::{Command, Child};
use std::path::PathBuf;
use thiserror::Error;

use crate::wsdg_open::{WsdgOpen, OpenError};
use crate::wsdg_env::WsdgEnv;

#[derive(Debug, Error)]
pub enum GhxOpenError {
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    
    #[error("No handler registered for: {0}")]
    NoHandler(String),
    
    #[error("Open error: {0}")]
    OpenError(#[from] OpenError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// URI structure
#[derive(Debug, Clone)]
pub struct Uri {
    pub scheme: String,
    pub authority: Option<String>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl Uri {
    /// Parse URI string
    pub fn parse(uri: &str) -> Result<Self, GhxOpenError> {
        let uri = uri.trim();
        
        // Split scheme
        let (scheme, rest) = if let Some(idx) = uri.find("://") {
            (uri[..idx].to_lowercase(), &uri[idx + 3..])
        } else if let Some(idx) = uri.find(':') {
            (uri[..idx].to_lowercase(), &uri[idx + 1..])
        } else {
            return Err(GhxOpenError::InvalidUri("Missing scheme".to_string()));
        };
        
        // Split fragment
        let (rest, fragment) = if let Some(idx) = rest.rfind('#') {
            (&rest[..idx], Some(rest[idx + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Split query
        let (rest, query) = if let Some(idx) = rest.find('?') {
            (&rest[..idx], Some(rest[idx + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Split authority and path
        let (authority, path) = if scheme == "file" {
            // File URIs don't have authority
            (None, rest.to_string())
        } else if rest.starts_with("//") {
            // Has authority
            let rest = &rest[2..];
            if let Some(idx) = rest.find('/') {
                (Some(rest[..idx].to_string()), rest[idx..].to_string())
            } else {
                (Some(rest.to_string()), "/".to_string())
            }
        } else {
            // No authority
            (None, rest.to_string())
        };
        
        Ok(Uri {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }
    
    /// Convert back to string
    pub fn to_string(&self) -> String {
        let mut result = format!("{}:", self.scheme);
        
        if let Some(ref authority) = self.authority {
            result.push_str("//");
            result.push_str(authority);
        } else if self.scheme != "file" {
            result.push_str("//");
        }
        
        result.push_str(&self.path);
        
        if let Some(ref query) = self.query {
            result.push('?');
            result.push_str(query);
        }
        
        if let Some(ref fragment) = self.fragment {
            result.push('#');
            result.push_str(fragment);
        }
        
        result
    }
}

/// Protocol handler function type
pub type ProtocolHandler = Box<dyn Fn(&Uri, &WsdgEnv) -> Result<Child, GhxOpenError>>;

/// WSDG GHX Open - URI and Protocol handler
pub struct WsdgGhxOpen {
    wsdg_open: WsdgOpen,
    handlers: HashMap<String, ProtocolHandler>,
}

impl WsdgGhxOpen {
    pub fn new(wsdg_open: WsdgOpen) -> Self {
        let mut ghx = Self {
            wsdg_open,
            handlers: HashMap::new(),
        };
        
        // Register default handlers
        ghx.register_default_handlers();
        ghx
    }
    
    /// Register default protocol handlers
    fn register_default_handlers(&mut self) {
        // App protocol - Custom app:// URI for manifest-based apps
        self.register_handler("app", Box::new(|uri, _env| {
            // This requires manifest support - for now delegate to system
            // In full implementation, this would use WsdgManifestOpen
            let app_target = uri.to_string();
            
            #[cfg(target_os = "linux")]
            {
                Command::new("wsdg-open")
                    .arg(&app_target)
                    .spawn()
                    .map_err(|e| GhxOpenError::IoError(e))
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                Err(GhxOpenError::UnsupportedProtocol("app".to_string()))
            }
        }));
        
        // File protocol
        self.register_handler("file", Box::new(|uri, _env| {
            let path = PathBuf::from(&uri.path);
            
            // Use system open command
            #[cfg(target_os = "linux")]
            {
                Command::new("xdg-open")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| GhxOpenError::IoError(e))
            }
            
            #[cfg(target_os = "macos")]
            {
                Command::new("open")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| GhxOpenError::IoError(e))
            }
            
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd")
                    .args(&["/C", "start", "", path.to_str().unwrap()])
                    .spawn()
                    .map_err(|e| GhxOpenError::IoError(e))
            }
        }));
        
        // HTTP/HTTPS protocols
        for scheme in &["http", "https"] {
            self.register_handler(scheme, Box::new(|uri, _env| {
                let url = uri.to_string();
                
                // Try common browsers
                let browsers = vec![
                    "firefox",
                    "chromium",
                    "google-chrome",
                    "brave",
                    "vivaldi",
                ];
                
                for browser in browsers {
                    if let Ok(child) = Command::new(browser).arg(&url).spawn() {
                        return Ok(child);
                    }
                }
                
                // Fallback to xdg-open
                #[cfg(target_os = "linux")]
                {
                    Command::new("xdg-open")
                        .arg(&url)
                        .spawn()
                        .map_err(|e| GhxOpenError::IoError(e))
                }
                
                #[cfg(not(target_os = "linux"))]
                {
                    Err(GhxOpenError::NoHandler(scheme.to_string()))
                }
            }));
        }
        
        // Mailto protocol
        self.register_handler("mailto", Box::new(|uri, _env| {
            let email = uri.to_string();
            
            // Try common email clients
            let clients = vec![
                "thunderbird",
                "evolution",
                "geary",
            ];
            
            for client in clients {
                if let Ok(child) = Command::new(client).arg(&email).spawn() {
                    return Ok(child);
                }
            }
            
            // Fallback
            #[cfg(target_os = "linux")]
            {
                Command::new("xdg-email")
                    .arg(&email)
                    .spawn()
                    .map_err(|e| GhxOpenError::IoError(e))
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                Err(GhxOpenError::NoHandler("mailto".to_string()))
            }
        }));
    }
    
    /// Register a protocol handler
    pub fn register_handler(&mut self, scheme: &str, handler: ProtocolHandler) {
        self.handlers.insert(scheme.to_lowercase(), handler);
    }
    
    /// Open URI with appropriate handler
    pub fn open_uri(&mut self, uri: &str) -> Result<Child, GhxOpenError> {
        let uri = Uri::parse(uri)?;
        
        // Get handler for scheme
        if let Some(handler) = self.handlers.get(&uri.scheme) {
            handler(&uri, self.wsdg_open.env())
        } else {
            Err(GhxOpenError::UnsupportedProtocol(uri.scheme.clone()))
        }
    }
    
    /// Open multiple URIs (in array)
    pub fn open_uris(&mut self, uris: &[&str]) -> Vec<Result<Child, GhxOpenError>> {
        uris.iter().map(|uri| self.open_uri(uri)).collect()
    }
    
    /// Check if protocol is supported
    pub fn is_protocol_supported(&self, scheme: &str) -> bool {
        self.handlers.contains_key(&scheme.to_lowercase())
    }
    
    /// List supported protocols
    pub fn supported_protocols(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
    
    /// Get environment reference
    pub fn env(&self) -> &WsdgEnv {
        self.wsdg_open.env()
    }
}

/// URI Builder for convenient URI construction
pub struct UriBuilder {
    scheme: String,
    authority: Option<String>,
    path: String,
    query_params: Vec<(String, String)>,
    fragment: Option<String>,
}

impl UriBuilder {
    pub fn new(scheme: &str) -> Self {
        Self {
            scheme: scheme.to_lowercase(),
            authority: None,
            path: String::new(),
            query_params: Vec::new(),
            fragment: None,
        }
    }
    
    pub fn authority(mut self, authority: &str) -> Self {
        self.authority = Some(authority.to_string());
        self
    }
    
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }
    
    pub fn query_param(mut self, key: &str, value: &str) -> Self {
        self.query_params.push((key.to_string(), value.to_string()));
        self
    }
    
    pub fn fragment(mut self, fragment: &str) -> Self {
        self.fragment = Some(fragment.to_string());
        self
    }
    
    pub fn build(self) -> Uri {
        let query = if self.query_params.is_empty() {
            None
        } else {
            Some(
                self.query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };
        
        Uri {
            scheme: self.scheme,
            authority: self.authority,
            path: self.path,
            query,
            fragment: self.fragment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uri_parsing() {
        let uri = Uri::parse("https://example.com/path?key=value#section").unwrap();
        assert_eq!(uri.scheme, "https");
        assert_eq!(uri.authority, Some("example.com".to_string()));
        assert_eq!(uri.path, "/path");
        assert_eq!(uri.query, Some("key=value".to_string()));
        assert_eq!(uri.fragment, Some("section".to_string()));
    }
    
    #[test]
    fn test_file_uri() {
        let uri = Uri::parse("file:///home/user/document.pdf").unwrap();
        assert_eq!(uri.scheme, "file");
        assert_eq!(uri.authority, None);
        assert_eq!(uri.path, "/home/user/document.pdf");
    }
    
    #[test]
    fn test_mailto_uri() {
        let uri = Uri::parse("mailto:user@example.com").unwrap();
        assert_eq!(uri.scheme, "mailto");
        assert_eq!(uri.path, "user@example.com");
    }
    
    #[test]
    fn test_uri_builder() {
        let uri = UriBuilder::new("https")
            .authority("example.com")
            .path("/search")
            .query_param("q", "rust")
            .fragment("results")
            .build();
        
        assert_eq!(uri.to_string(), "https://example.com/search?q=rust#results");
    }
    
    #[test]
    fn test_uri_roundtrip() {
        let original = "https://example.com:8080/path?a=1&b=2#test";
        let uri = Uri::parse(original).unwrap();
        let reconstructed = uri.to_string();
        
        // Parse again to verify
        let uri2 = Uri::parse(&reconstructed).unwrap();
        assert_eq!(uri.scheme, uri2.scheme);
        assert_eq!(uri.path, uri2.path);
    }
}

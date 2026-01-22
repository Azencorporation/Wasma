// WSDG MIME Array - MIME Type Definition and Handling
// Defines MIME types for files and formats
// Core WSDG format and file MIME definition system
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MimeError {
    #[error("Unknown MIME type for: {0}")]
    UnknownMimeType(String),
    
    #[error("Invalid extension: {0}")]
    InvalidExtension(String),
    
    #[error("Failed to read magic bytes: {0}")]
    MagicBytesError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// MIME type information
#[derive(Debug, Clone)]
pub struct MimeType {
    pub mime: String,
    pub extensions: Vec<String>,
    pub description: String,
    pub category: MimeCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimeCategory {
    Text,
    Image,
    Audio,
    Video,
    Application,
    Archive,
    Document,
    Code,
    Font,
    Model,
    Message,
}

impl MimeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Application => "application",
            Self::Archive => "archive",
            Self::Document => "document",
            Self::Code => "code",
            Self::Font => "font",
            Self::Model => "model",
            Self::Message => "message",
        }
    }
}

/// Magic byte signature for file type detection
#[derive(Debug, Clone)]
pub struct MagicSignature {
    pub offset: usize,
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// WSDG MIME Array - MIME type registry and detector
pub struct WsdgMimeArray {
    mime_by_ext: HashMap<String, MimeType>,
    ext_by_mime: HashMap<String, Vec<String>>,
    magic_signatures: Vec<MagicSignature>,
}

impl WsdgMimeArray {
    pub fn new() -> Self {
        let mut array = Self {
            mime_by_ext: HashMap::new(),
            ext_by_mime: HashMap::new(),
            magic_signatures: Vec::new(),
        };
        
        array.register_standard_types();
        array.register_magic_signatures();
        array
    }
    
    /// Register standard MIME types
    fn register_standard_types(&mut self) {
        // Text formats
        self.register("text/plain", &["txt", "text"], "Plain Text", MimeCategory::Text);
        self.register("text/html", &["html", "htm"], "HTML Document", MimeCategory::Text);
        self.register("text/css", &["css"], "Stylesheet", MimeCategory::Text);
        self.register("text/javascript", &["js", "mjs"], "JavaScript", MimeCategory::Code);
        self.register("text/markdown", &["md", "markdown"], "Markdown", MimeCategory::Text);
        self.register("text/xml", &["xml"], "XML Document", MimeCategory::Text);
        
        // Images
        self.register("image/png", &["png"], "PNG Image", MimeCategory::Image);
        self.register("image/jpeg", &["jpg", "jpeg"], "JPEG Image", MimeCategory::Image);
        self.register("image/gif", &["gif"], "GIF Image", MimeCategory::Image);
        self.register("image/webp", &["webp"], "WebP Image", MimeCategory::Image);
        self.register("image/svg+xml", &["svg"], "SVG Vector", MimeCategory::Image);
        self.register("image/bmp", &["bmp"], "Bitmap Image", MimeCategory::Image);
        self.register("image/x-icon", &["ico"], "Icon", MimeCategory::Image);
        
        // Audio
        self.register("audio/mpeg", &["mp3"], "MP3 Audio", MimeCategory::Audio);
        self.register("audio/ogg", &["ogg", "oga"], "Ogg Audio", MimeCategory::Audio);
        self.register("audio/wav", &["wav"], "WAV Audio", MimeCategory::Audio);
        self.register("audio/flac", &["flac"], "FLAC Audio", MimeCategory::Audio);
        self.register("audio/aac", &["aac"], "AAC Audio", MimeCategory::Audio);
        
        // Video
        self.register("video/mp4", &["mp4", "m4v"], "MP4 Video", MimeCategory::Video);
        self.register("video/webm", &["webm"], "WebM Video", MimeCategory::Video);
        self.register("video/ogg", &["ogv"], "Ogg Video", MimeCategory::Video);
        self.register("video/x-matroska", &["mkv"], "Matroska Video", MimeCategory::Video);
        self.register("video/avi", &["avi"], "AVI Video", MimeCategory::Video);
        
        // Documents
        self.register("application/pdf", &["pdf"], "PDF Document", MimeCategory::Document);
        self.register("application/msword", &["doc"], "Word Document", MimeCategory::Document);
        self.register("application/vnd.openxmlformats-officedocument.wordprocessingml.document", 
                     &["docx"], "Word Document (New)", MimeCategory::Document);
        self.register("application/vnd.ms-excel", &["xls"], "Excel Spreadsheet", MimeCategory::Document);
        self.register("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                     &["xlsx"], "Excel Spreadsheet (New)", MimeCategory::Document);
        self.register("application/vnd.ms-powerpoint", &["ppt"], "PowerPoint", MimeCategory::Document);
        self.register("application/vnd.openxmlformats-officedocument.presentationml.presentation",
                     &["pptx"], "PowerPoint (New)", MimeCategory::Document);
        
        // Archives
        self.register("application/zip", &["zip"], "ZIP Archive", MimeCategory::Archive);
        self.register("application/x-tar", &["tar"], "TAR Archive", MimeCategory::Archive);
        self.register("application/gzip", &["gz"], "GZip Archive", MimeCategory::Archive);
        self.register("application/x-bzip2", &["bz2"], "BZip2 Archive", MimeCategory::Archive);
        self.register("application/x-7z-compressed", &["7z"], "7-Zip Archive", MimeCategory::Archive);
        self.register("application/x-rar-compressed", &["rar"], "RAR Archive", MimeCategory::Archive);
        
        // Code
        self.register("text/x-python", &["py"], "Python Script", MimeCategory::Code);
        self.register("text/x-rust", &["rs"], "Rust Source", MimeCategory::Code);
        self.register("text/x-c", &["c", "h"], "C Source", MimeCategory::Code);
        self.register("text/x-c++", &["cpp", "hpp", "cc", "cxx"], "C++ Source", MimeCategory::Code);
        self.register("text/x-java", &["java"], "Java Source", MimeCategory::Code);
        self.register("text/x-go", &["go"], "Go Source", MimeCategory::Code);
        self.register("application/json", &["json"], "JSON Data", MimeCategory::Code);
        self.register("application/x-yaml", &["yaml", "yml"], "YAML Data", MimeCategory::Code);
        self.register("application/toml", &["toml"], "TOML Config", MimeCategory::Code);
        
        // Applications
        self.register("application/x-executable", &[""], "Executable", MimeCategory::Application);
        self.register("application/x-sharedlib", &["so"], "Shared Library", MimeCategory::Application);
        self.register("application/x-desktop", &["desktop"], "Desktop Entry", MimeCategory::Application);
        
        // Fonts
        self.register("font/ttf", &["ttf"], "TrueType Font", MimeCategory::Font);
        self.register("font/otf", &["otf"], "OpenType Font", MimeCategory::Font);
        self.register("font/woff", &["woff"], "WOFF Font", MimeCategory::Font);
        self.register("font/woff2", &["woff2"], "WOFF2 Font", MimeCategory::Font);
    }
    
    /// Register magic byte signatures for file type detection
    fn register_magic_signatures(&mut self) {
        // PNG
        self.add_magic(0, vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "image/png");
        
        // JPEG
        self.add_magic(0, vec![0xFF, 0xD8, 0xFF], "image/jpeg");
        
        // GIF
        self.add_magic(0, vec![0x47, 0x49, 0x46, 0x38], "image/gif");
        
        // PDF
        self.add_magic(0, vec![0x25, 0x50, 0x44, 0x46], "application/pdf");
        
        // ZIP
        self.add_magic(0, vec![0x50, 0x4B, 0x03, 0x04], "application/zip");
        
        // GZIP
        self.add_magic(0, vec![0x1F, 0x8B], "application/gzip");
        
        // ELF (Linux executable)
        self.add_magic(0, vec![0x7F, 0x45, 0x4C, 0x46], "application/x-executable");
        
        // MP3
        self.add_magic(0, vec![0x49, 0x44, 0x33], "audio/mpeg"); // ID3
        self.add_magic(0, vec![0xFF, 0xFB], "audio/mpeg"); // MPEG frame
        
        // OGG
        self.add_magic(0, vec![0x4F, 0x67, 0x67, 0x53], "audio/ogg");
        
        // FLAC
        self.add_magic(0, vec![0x66, 0x4C, 0x61, 0x43], "audio/flac");
    }
    
    /// Register a MIME type
    pub fn register(&mut self, mime: &str, extensions: &[&str], description: &str, category: MimeCategory) {
        let mime_type = MimeType {
            mime: mime.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
            category,
        };
        
        // Store by extension
        for ext in extensions {
            self.mime_by_ext.insert(ext.to_string(), mime_type.clone());
        }
        
        // Store extensions by MIME
        self.ext_by_mime.insert(
            mime.to_string(),
            extensions.iter().map(|s| s.to_string()).collect()
        );
    }
    
    /// Add magic signature
    pub fn add_magic(&mut self, offset: usize, bytes: Vec<u8>, mime_type: &str) {
        self.magic_signatures.push(MagicSignature {
            offset,
            bytes,
            mime_type: mime_type.to_string(),
        });
    }
    
    /// Get MIME type from file path
    pub fn from_path(&self, path: &Path) -> Result<String, MimeError> {
        // Try extension first
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(mime_type) = self.mime_by_ext.get(ext) {
                return Ok(mime_type.mime.clone());
            }
        }
        
        // Try magic bytes
        if path.exists() && path.is_file() {
            if let Ok(mime) = self.detect_from_magic(path) {
                return Ok(mime);
            }
        }
        
        // Default
        Ok("application/octet-stream".to_string())
    }
    
    /// Detect MIME type from magic bytes
    pub fn detect_from_magic(&self, path: &Path) -> Result<String, MimeError> {
        let mut file = fs::File::open(path)?;
        use std::io::Read;
        
        let mut buffer = vec![0u8; 512]; // Read first 512 bytes
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        
        // Check signatures
        for sig in &self.magic_signatures {
            if sig.offset + sig.bytes.len() <= buffer.len() {
                let slice = &buffer[sig.offset..sig.offset + sig.bytes.len()];
                if slice == sig.bytes.as_slice() {
                    return Ok(sig.mime_type.clone());
                }
            }
        }
        
        Err(MimeError::UnknownMimeType(path.display().to_string()))
    }
    
    /// Get MIME type from extension
    pub fn from_extension(&self, ext: &str) -> Option<String> {
        self.mime_by_ext.get(ext).map(|m| m.mime.clone())
    }
    
    /// Get extensions for MIME type
    pub fn get_extensions(&self, mime: &str) -> Option<&Vec<String>> {
        self.ext_by_mime.get(mime)
    }
    
    /// Get MIME type info
    pub fn get_info(&self, mime_or_ext: &str) -> Option<&MimeType> {
        // Try as MIME type first
        if mime_or_ext.contains('/') {
            // Find by MIME
            self.mime_by_ext.values().find(|m| m.mime == mime_or_ext)
        } else {
            // Try as extension
            self.mime_by_ext.get(mime_or_ext)
        }
    }
    
    /// Get category for file
    pub fn get_category(&self, path: &Path) -> MimeCategory {
        if let Ok(mime) = self.from_path(path) {
            if let Some(info) = self.get_info(&mime) {
                return info.category;
            }
        }
        MimeCategory::Application
    }
    
    /// Check if file is of specific category
    pub fn is_category(&self, path: &Path, category: MimeCategory) -> bool {
        self.get_category(path) == category
    }
}

impl Default for WsdgMimeArray {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    #[test]
    fn test_extension_detection() {
        let mime_array = WsdgMimeArray::new();
        
        assert_eq!(
            mime_array.from_extension("png"),
            Some("image/png".to_string())
        );
        
        assert_eq!(
            mime_array.from_extension("rs"),
            Some("text/x-rust".to_string())
        );
    }
    
    #[test]
    fn test_path_detection() {
        let mime_array = WsdgMimeArray::new();
        let path = Path::new("test.png");
        
        let mime = mime_array.from_path(&path).unwrap();
        assert_eq!(mime, "image/png");
    }
    
    #[test]
    fn test_magic_detection() {
        let mime_array = WsdgMimeArray::new();
        
        // Create temp PNG file
        let temp_dir = std::env::temp_dir();
        let png_path = temp_dir.join("test.dat");
        
        let mut file = fs::File::create(&png_path).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        
        let mime = mime_array.detect_from_magic(&png_path).unwrap();
        assert_eq!(mime, "image/png");
        
        let _ = fs::remove_file(png_path);
    }
    
    #[test]
    fn test_category_detection() {
        let mime_array = WsdgMimeArray::new();
        let path = Path::new("image.png");
        
        assert_eq!(mime_array.get_category(&path), MimeCategory::Image);
        assert!(mime_array.is_category(&path, MimeCategory::Image));
    }
}

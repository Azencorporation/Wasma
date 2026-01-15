// WASMA - WGClient (Wayland/X11 Graphics Client)

use std::sync::Arc;
use crate::parser::{WasmaConfig, Protocol}; // Protocol import düzeltildi
use crate::protocols::ProtocolManager;
use x11rb::connection::Connection as XConnection;
use x11rb::protocol::xproto::{self, ConnectionExt};

// Global VRAM adresleri tanımlandı
pub static WASMA_VRAM_ADDR: usize = 0xB0000000; // Örnek base adres
pub static mut WASMA_CORE_ACTIVE: bool = true;

pub struct WGClient {
    config: Arc<WasmaConfig>,
    x11_ctx: Option<(Arc<x11rb::rust_connection::RustConnection>, xproto::Window)>,
}

impl WGClient {
    pub fn new(config: WasmaConfig) -> Self {
        let is_native = config.resource_limits.scope_level > 0;
        let mut x11_ctx = None;
        
        if !is_native {
            if let Ok((conn, screen_num)) = x11rb::connect(None) {
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
        }
    }

    pub async fn run_engine(&self, mut manager: ProtocolManager) {
        let is_multi = self.config.uri_handling.multi_instances;
        let is_singularity = self.config.uri_handling.singularity_instances;
        let mut stream_count = 0;

        // active_streams artık public, direkt erişilebilir
        let streams = std::mem::take(&mut manager.active_streams);
        
        for mut stream in streams {
            if is_singularity && stream_count >= 1 { 
                break; 
            }
            
            let proto_type = stream.get_type();
            let stream_id = stream_count;
            
            tokio::spawn(async move {
                match proto_type {
                    Protocol::Tor => {
                        let mut buf = [0u8; 65536];
                        while let Ok(n) = stream.read(&mut buf).await {
                            if n == 0 { break; }
                            Self::route_to_display(&buf[..n], stream_id);
                        }
                    },
                    Protocol::Grpc => {
                        while let Ok(Some(frame)) = stream.next_message().await {
                            Self::route_to_display(&frame, stream_id);
                        }
                    },
                    Protocol::Https | Protocol::Http => {
                        while let Ok(chunk) = stream.next_chunk().await {
                            if chunk.is_empty() { break; }
                            Self::route_to_display(&chunk, stream_id);
                        }
                    }
                }
            });
            
            stream_count += 1;
            if !is_multi { break; }
        }
    }

    fn route_to_display(data: &[u8], stream_id: u8) {
        unsafe {
            if WASMA_CORE_ACTIVE {
                Self::write_raw_vram(data, stream_id);
            } else {
                // Fallback rendering (X11/Wayland)
                // Bu durumda instance'a ihtiyaç var, static olduğu için şimdilik skip
            }
        }
    }

    fn write_raw_vram(data: &[u8], stream_id: u8) {
        unsafe {
            let offset = stream_id as usize * (1024 * 1024);
            let target_ptr = (WASMA_VRAM_ADDR + offset) as *mut u8;
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                target_ptr,
                data.len()
            );
        }
    }

    pub fn write_x11_frame(&self, data: &[u8], stream_id: u8) {
        if let Some((conn, win)) = &self.x11_ctx {
            let gc = conn.generate_id().unwrap();
            conn.create_gc(gc, *win, &xproto::CreateGCAux::new()).ok();
            
            let y_pos = (stream_id as i16) * 200;
            
            conn.put_image(
                xproto::ImageFormat::Z_PIXMAP,
                *win,
                gc,
                1280, 200,
                0, y_pos, 0, 24, data
            ).ok();
            
            conn.free_gc(gc).ok();
            conn.flush().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ConfigParser;

    #[test]
    fn test_wgclient_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let _client = WGClient::new(config);
        assert!(unsafe { WASMA_CORE_ACTIVE });
    }
}

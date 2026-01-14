use std::sync::Arc;
use crate::parser::{WasmaConfig, Protocol};
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
            multitary: WindowMultitary::new((*config_arc).clone(), width, height),
            singularity: WindowSingularity::new((*config_arc).clone(), width, height),
            config: config_arc,
            width,
            height,
        }
    }

    pub fn from_config(config: Arc<WasmaConfig>, width: u32, height: u32) -> Self {
        Self {
            multitary: WindowMultitary::new((*config).clone(), width, height),
            singularity: WindowSingularity::new((*config).clone(), width, height),
            config,
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

    fn blit_native_vram(&self, data: &[u8], bounds: (i32, i32, u32, u32), _stream_id: u8) {
        let (x, y, w, h) = bounds;
        
        // Calculate offset in framebuffer
        let offset = (y as usize * self.width as usize + x as usize) * 4; // 4 bytes per pixel (RGBA)
        let max_size = (w * h * 4) as usize;
        let copy_size = data.len().min(max_size);
        
        // Simulate direct VRAM write
        // In production, this would write to actual VRAM through DRM/KMS or similar
        let _vram_operation = WasmaVramWrite {
            offset,
            size: copy_size,
            bounds: (x, y, w, h),
        };
        
        // For now, just acknowledge the operation
        if copy_size > 0 {
            // VRAM write would happen here
        }
    }

    fn blit_os_fallback(&self, data: &[u8], bounds: (i32, i32, u32, u32), _stream_id: u8) {
        let (x, y, w, h) = bounds;
        
        // Fallback: Use OS-specific rendering (X11 PutImage, Wayland subsurface, etc.)
        // This would integrate with the WGClient for actual rendering
        let _fallback_operation = WasmaFallbackRender {
            data_size: data.len(),
            bounds: (x, y, w, h),
        };
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
        self.multitary.update_resolution(new_width, new_height);
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_config(&self) -> &WasmaConfig {
        &self.config
    }

    pub fn enter_singularity(&mut self, stream_id: u8) {
        self.singularity.enter_singularity_mode(stream_id);
    }

    pub fn exit_singularity(&mut self) {
        self.singularity.exit_singularity_mode();
    }

    pub fn is_singularity_active(&self) -> bool {
        SINGULARITY_LOCK.load(Ordering::SeqCst)
    }
}

// Helper structures for VRAM operations
struct WasmaVramWrite {
    offset: usize,
    size: usize,
    bounds: (i32, i32, u32, u32),
}

struct WasmaFallbackRender {
    data_size: usize,
    bounds: (i32, i32, u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ConfigParser;

    #[test]
    fn test_window_client_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let client = WindowClient::new(config, 1920, 1080);
        assert_eq!(client.get_dimensions(), (1920, 1080));
    }

    #[test]
    fn test_singularity_toggle() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let mut client = WindowClient::new(config, 1920, 1080);
        
        assert!(!client.is_singularity_active());
        
        client.enter_singularity(0);
        assert!(client.is_singularity_active());
        
        client.exit_singularity();
        assert!(!client.is_singularity_active());
    }
}

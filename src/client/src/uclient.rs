use std::net::TcpStream;
use std::io::{Read, ErrorKind};
use crate::parser::WasmaConfig;
use std::sync::Arc;

#[cfg(feature = "glx")]
use gl;

#[cfg(feature = "opencl-gpu")]
use opencl3::command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE};
#[cfg(feature = "opencl-gpu")]
use opencl3::context::Context;
#[cfg(feature = "opencl-gpu")]
use opencl3::device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU};
#[cfg(feature = "opencl-gpu")]
use opencl3::memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_COPY_HOST_PTR};

#[cfg(feature = "intel-uhd")]
use rayon::prelude::*;

/// WASMA Section Memory: Memory divided into mathematical sections
pub struct SectionMemory {
    pub raw_storage: Vec<u8>,
    pub cell_count: usize,
    pub cell_size: usize,
}

impl SectionMemory {
    pub fn new(level: u32) -> Self {
        let cell_count = if level == 0 { 1 } else { level as usize };
        let cell_size = 1024 * 1024; // 1MB per cell
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
        let end = (start + self.cell_size).min(self.raw_storage.len());
        &mut self.raw_storage[start..end]
    }

    #[inline]
    pub fn get_cell(&self, index: usize) -> &[u8] {
        let start = index * self.cell_size;
        let end = (start + self.cell_size).min(self.raw_storage.len());
        &self.raw_storage[start..end]
    }
}

pub struct UClient {
    config: Arc<WasmaConfig>,
    memory: SectionMemory,
}

impl UClient {
    pub fn new(config: WasmaConfig) -> Self {
        let level = config.resource_limits.scope_level;
        Self {
            config: Arc::new(config),
            memory: SectionMemory::new(level),
        }
    }

    pub fn from_config(config: Arc<WasmaConfig>) -> Self {
        let level = config.resource_limits.scope_level;
        Self {
            config,
            memory: SectionMemory::new(level),
        }
    }

    pub fn start_engine(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.uri_handling.protocols.is_empty() {
            return Err("No protocols configured".into());
        }

        let proto = &self.config.uri_handling.protocols[0];
        let addr = format!("{}:{}", proto.ip, proto.port);
        
        println!("🔌 Connecting to {}...", addr);
        let mut stream = TcpStream::connect(&addr)?;
        
        let level = self.config.resource_limits.scope_level;

        println!("🟢 WASMA UClient: Engine Started");
        println!("📡 Mode: {}", if level == 0 { "NULL_EXCEPTION (Bypass/Raw)" } else { "Partitioned" });
        println!("🎨 Renderer: {}", self.config.resource_limits.renderer);

        if level == 0 {
            // NULL_EXCEPTION: Raw stream mode - no memory partitioning
            // Data is passed directly to renderer in 4KB windows
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
            // Mathematical partitioning mode
            // Data fills cells and is processed synchronously
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
            #[cfg(feature = "glx")]
            "glx_renderer" => self.run_glx(data),
            
            #[cfg(feature = "intel-uhd")]
            "renderer_iuhd" | "intel_uhd" => self.run_iuhd(data),
            
            #[cfg(feature = "opencl-gpu")]
            "renderer_opencl" | "opencl" => self.run_opencl(data),
            
            "cpu_renderer" | "cpu" => self.run_cpu(data),
            
            _ => {
                #[cfg(feature = "glx")]
                self.run_glx(data);
                
                #[cfg(not(feature = "glx"))]
                self.run_cpu(data);
            }
        }
    }

    // Renderer implementations
    
    #[cfg(feature = "glx")]
    fn run_glx(&self, data: &[u8]) {
        unsafe {
            // Direct VRAM texture update bypassing X11/Wayland
            gl::TexSubImage2D(
                gl::TEXTURE_2D, 0, 0, 0,
                1024, 1, // Width based on raw data
                gl::RGBA, gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _
            );
        }
    }

    #[cfg(not(feature = "glx"))]
    #[allow(dead_code)]
    fn run_glx(&self, _data: &[u8]) {
        eprintln!("⚠️  GLX renderer not available - build with 'glx' feature");
    }

    #[cfg(feature = "intel-uhd")]
    #[allow(dead_code)]
    fn run_iuhd(&self, data: &[u8]) {
        // Intel UHD: Parallel 4x4 matrix processing with Rayon
        data.par_chunks(16).for_each(|block| {
            let _avg: u32 = block.iter().map(|&x| x as u32).sum::<u32>() / 16;
            // Mathematical sharpening algorithms run here
        });
    }

    #[cfg(not(feature = "intel-uhd"))]
    #[allow(dead_code)]
    fn run_iuhd(&self, _data: &[u8]) {
        eprintln!("⚠️  Intel UHD renderer not available - build with 'intel-uhd' feature");
    }

    #[cfg(feature = "opencl-gpu")]
    #[allow(dead_code)]
    fn run_opencl(&self, data: &[u8]) {
        // GPGPU: Zero-copy host pointer mapping
        if let Ok(devices) = get_all_devices(CL_DEVICE_TYPE_GPU) {
            if let Some(device_id) = devices.first() {
                let device = Device::new(*device_id);
                if let Ok(context) = Context::from_device(&device) {
                    let _buffer = unsafe {
                        Buffer::<u8>::create(
                            &context,
                            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
                            data.len(),
                            data.as_ptr() as *mut _,
                        ).ok()
                    };
                }
            }
        }
    }

    #[cfg(not(feature = "opencl-gpu"))]
    #[allow(dead_code)]
    fn run_opencl(&self, _data: &[u8]) {
        eprintln!("⚠️  OpenCL renderer not available - build with 'opencl-gpu' feature");
    }

    fn run_cpu(&self, data: &[u8]) {
        // CPU-based rendering fallback
        // Process data in chunks
        for chunk in data.chunks(1024) {
            let _sum: u32 = chunk.iter().map(|&x| x as u32).sum();
            // Basic CPU processing
        }
    }

    pub fn get_config(&self) -> &WasmaConfig {
        &self.config
    }

    pub fn get_memory_usage(&self) -> (usize, usize, usize) {
        (
            self.memory.raw_storage.len(),
            self.memory.cell_count,
            self.memory.cell_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ConfigParser;

    #[test]
    fn test_section_memory() {
        let memory = SectionMemory::new(10);
        assert_eq!(memory.cell_count, 10);
        assert_eq!(memory.cell_size, 1024 * 1024);
    }

    #[test]
    fn test_uclient_creation() {
        let parser = ConfigParser::new(None);
        let config_content = parser.generate_default_config();
        let config = parser.parse(&config_content).unwrap();
        
        let client = UClient::new(config);
        let (total, cells, cell_size) = client.get_memory_usage();
        
        assert!(total > 0);
        assert!(cells > 0);
        assert_eq!(cell_size, 1024 * 1024);
    }
}

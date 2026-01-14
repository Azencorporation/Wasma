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

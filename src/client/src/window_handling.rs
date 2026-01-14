// window_handling.rs
// WASMA Pencere Yönetimi - protocols + manifest + source entegrasyonu
// Iced GUI ile pencere oluşturma

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};
use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, row, text, scrollable, Space},
    executor, window, Length, Color, Background,
};
use iced::window::{Id as WindowId, Position};

// Diğer modüllerden import (aynı crate içinde)
use crate::protocols::{ProtocolManager, WasmaConfig, Protocol};
use crate::manifest::{ManifestParser, WasmaManifest, CpuCoreServe};
use crate::source::{SourceParser, PermissionSource};

// ============================================================================
// PENCERE YAPILARI
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

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

#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    Native,
    Wayland,
    X11,
    Remote(String),
}

/// Kaynak limitleri - WBackend + manifest + wasma.in.conf
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

/// İzinler - source'tan gelen
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

/// Ana pencere yapısı
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
// WINDOW HANDLER - Manifest, Source, Protocol Entegrasyonu
// ============================================================================

pub struct WindowHandler {
    windows: Arc<Mutex<HashMap<u64, Window>>>,
    next_id: Arc<Mutex<u64>>,
    focused_window: Arc<Mutex<Option<u64>>>,
    wbackend: Arc<WBackend>,
    assignment_to_window: Arc<Mutex<HashMap<u32, u64>>>,
    
    // WASMA config
    wasma_config: Arc<Mutex<Option<WasmaConfig>>>,
}

impl WindowHandler {
    pub fn new(resource_mode: ResourceMode) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            focused_window: Arc::new(Mutex::new(None)),
            wbackend: Arc::new(WBackend::new(resource_mode)),
            assignment_to_window: Arc::new(Mutex::new(HashMap::new())),
            wasma_config: Arc::new(Mutex::new(None)),
        }
    }

    /// wasma.in.conf yükle
    pub fn load_wasma_config(&self, config_path: &str) -> Result<(), String> {
        let mut protocol_mgr = ProtocolManager::new(Some(config_path.to_string()));
        protocol_mgr.load_config()
            .map_err(|e| format!("Config yüklenemedi: {:?}", e))?;
        
        protocol_mgr.validate()
            .map_err(|e| format!("Config geçersiz: {:?}", e))?;

        let config = protocol_mgr.get_config()
            .ok_or("Config bulunamadı")?
            .clone();

        let mut wasma_cfg = self.wasma_config.lock().unwrap();
        *wasma_cfg = Some(config);

        println!("✅ WASMA Config yüklendi: {}", config_path);
        Ok(())
    }

    /// Pencere oluştur - manifest + source + wasma.in.conf birleşimi
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

        // 1. Manifest varsa onu yükle
        let (mut resource_limits, mut permissions) = if let Some(ref path) = manifest_path {
            self.load_manifest_and_source(path)?
        } else {
            (ResourceLimits::default(), PermissionScope::default())
        };

        // 2. wasma.in.conf'tan renderer ve scope_level al
        if let Some(ref wasma_cfg) = *self.wasma_config.lock().unwrap() {
            resource_limits.renderer = wasma_cfg.resource_limits.renderer.clone();
            resource_limits.pixel_load_limit = Some(wasma_cfg.resource_limits.scope_level);
            
            // Protocol izinlerini ekle
            for proto_cfg in &wasma_cfg.uri_handling.protocols {
                let proto_str = match proto_cfg.protocol {
                    Protocol::Http => "http",
                    Protocol::Https => "https",
                    Protocol::Grpc => "grpc",
                    Protocol::Tor => "tor",
                };
                if !permissions.allowed_protocols.contains(&proto_str.to_string()) {
                    permissions.allowed_protocols.push(proto_str.to_string());
                }
            }
        }

        // 3. WBackend Assignment oluştur
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

        // 4. Window yapısını oluştur
        let window = Window {
            id: window_id,
            iced_window_id: None,
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
            "🪟 Window {} oluşturuldu | Assignment {} | Mode: {:?}",
            window_id, assignment_id, resource_mode
        );

        Ok(window_id)
    }

    /// Manifest ve Source'u yükle
    fn load_manifest_and_source(&self, manifest_path: &str) -> Result<(ResourceLimits, PermissionScope), String> {
        // 1. Manifest parse et
        let parser = ManifestParser::new(manifest_path.to_string());
        let manifest = parser.load()
            .map_err(|e| format!("Manifest yüklenemedi: {:?}", e))?;

        // 2. Resource limits oluştur
        let mut limits = ResourceLimits::default();
        
        // CPU
        limits.cpu_cores = match manifest.resources.cpu_core_serve {
            CpuCoreServe::Static(n) => (0..n as usize).collect(),
            CpuCoreServe::Dynamic => Vec::new(),
            CpuCoreServe::AffinityDefault => Vec::new(),
        };
        
        // RAM
        limits.max_memory_mb = Some(manifest.resources.ram_using.size);
        
        // GPU
        limits.max_gpu_memory_mb = Some(manifest.resources.gpu_using.size);
        
        // Execution mode (manifest'te yok, default kullan)
        limits.execution_mode = ExecutionMode::GpuPreferred;

        // 3. Permission source yükle
        let source_parser = SourceParser::new(None);
        let perms = if let Ok(source) = source_parser.load_embedded(&std::fs::read_to_string(manifest_path).unwrap_or_default()) {
            if let Some(src) = source {
                self.parse_permissions(src)
            } else {
                PermissionScope::default()
            }
        } else {
            PermissionScope::default()
        };

        Ok((limits, perms))
    }

    /// Source'tan izinleri parse et
    fn parse_permissions(&self, source: PermissionSource) -> PermissionScope {
        let mut perms = PermissionScope::default();
        
        // Network
        perms.can_access_network = source.network.ethernet || source.network.wifi;
        
        // Filesystem
        perms.can_access_filesystem = !matches!(source.filesystem.file_exception, FileException::None);
        
        // GPU
        perms.can_use_gpu = true; // Her zaman true, wasma.in.conf kontrol eder
        
        // Protocols
        perms.allowed_protocols = vec!["http".to_string(), "https".to_string()];
        
        perms
    }

    // Mevcut fonksiyonlar aynen kalıyor
    pub fn run_resource_cycle(&self) {
        self.wbackend.run_cycle();
    }

    pub fn adjust_window_resources(&self, window_id: u64, new_limits: ResourceLimits) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        let window = windows.get_mut(&window_id)
            .ok_or_else(|| format!("Window {} bulunamadı", window_id))?;

        if window.resource_mode != ResourceMode::Manual {
            return Err(format!("Window {} Auto modda, manuel ayarlama yapılamaz", window_id));
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
            }
        }

        Ok(())
    }

    pub fn get_window_resource_usage(&self, window_id: u64) -> Result<ResourceUsage, String> {
        let windows = self.windows.lock().unwrap();
        let window = windows.get(&window_id)
            .ok_or_else(|| format!("Window {} bulunamadı", window_id))?;

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

        Err(format!("Assignment bulunamadı: window {}", window_id))
    }

    pub fn set_window_state(&self, id: u64, state: WindowState) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.get_mut(&id) {
            window.state = state;
            window.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(format!("Window {} bulunamadı", id))
        }
    }

    pub fn set_geometry(&self, id: u64, geometry: WindowGeometry) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        if let Some(window) = windows.get_mut(&id) {
            window.geometry = geometry;
            window.last_activity = SystemTime::now();
            Ok(())
        } else {
            Err(format!("Window {} bulunamadı", id))
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
            Err(format!("Window {} bulunamadı", id))
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
                }
                
                let mut mapping = self.assignment_to_window.lock().unwrap();
                mapping.remove(&aid);
            }
            
            windows.remove(&id);
            println!("🗑️  Window {} kapatıldı", id);
            Ok(())
        } else {
            Err(format!("Window {} bulunamadı", id))
        }
    }

    pub fn set_parent(&self, child_id: u64, parent_id: u64) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        
        if !windows.contains_key(&parent_id) {
            return Err(format!("Parent window {} bulunamadı", parent_id));
        }
        
        if let Some(child) = windows.get_mut(&child_id) {
            child.parent_id = Some(parent_id);
        } else {
            return Err(format!("Child window {} bulunamadı", child_id));
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
}

// ============================================================================
// ICED APPLICATION - GUI
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
        
        // wasma.in.conf yükle (opsiyonel)
        if let Err(e) = handler.load_wasma_config("/etc/wasma/wasma.in.conf") {
            eprintln!("⚠️  WASMA config yüklenemedi: {}", e);
        }
        
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
                    None, // manifest_path opsiyonel
                    ResourceMode::Auto,
                ) {
                    Ok(id) => {
                        println!("✅ Window {} oluşturuldu", id);
                        Command::none()
                    }
                    Err(e) => {
                        eprintln!("❌ Window oluşturulamadı: {}", e);
                        Command::none()
                    }
                }
            }
            
            Message::CloseWindow(id) => {
                if let Err(e) = self.handler.close_window(id) {
                    eprintln!("❌ Window kapatılamadı {}: {}", id, e);
                }
                if self.selected_window == Some(id) {
                    self.selected_window = None;
                }
                Command::none()
            }
            
            Message::FocusWindow(id) => {
                if let Err(e) = self.handler.focus_window(id) {
                    eprintln!("❌ Focus yapılamadı {}: {}", id, e);
                }
                self.selected_window = Some(id);
                Command::none()
            }
            
            Message::MinimizeWindow(id) => {
                if let Err(e) = self.handler.set_window_state(id, WindowState::Minimized) {
                    eprintln!("❌ Minimize yapılamadı {}: {}", id, e);
                }
                Command::none()
            }
            
            Message::MaximizeWindow(id) => {
                if let Err(e) = self.handler.set_window_state(id, WindowState::Maximized) {
                    eprintln!("❌ Maximize yapılamadı {}: {}", id, e);
                }
                Command::none()
            }
            
            Message::UpdateResourceCycle => {
                self.handler.run_resource_cycle();
                Command::none()
            }
            
            Message::AdjustResources(id) => {
                println!("🔧 Resource ayarlama: window {}", id);
                Command::none()
            }
            
            Message::ChangeExecutionMode(id, mode) => {
                if let Some(window) = self.handler.get_window(id) {
                    let mut new_limits = window.resource_limits.clone();
                    new_limits.execution_mode = mode;
                    if let Err(e) = self.handler.adjust_window_resources(id, new_limits) {
                        eprintln!("❌ Execution mode değiştirilemedi: {}", e);
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
            button("+ Yeni Pencere").on_press(Message::CreateWindow),
            Space::with_width(10),
            button("⟳ Kaynak Güncelle").on_press(Message::UpdateResourceCycle),
        ]
        .padding(20)
        .spacing(10);

        let mut window_list = column![].spacing(10).padding(20);

        if windows.is_empty() {
            window_list = window_list.push(
                text("Aktif pencere yok. 'Yeni Pencere' ile oluşturun.")
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

/// WASMA Window Manager'ı başlat
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

// ============================================================================
// TESTLER
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let handler = WindowHandler::new(ResourceMode::Auto);
        let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
        let id = handler.create_window(
            "Test Window".to_string(),
            "test.app".to_string(),
            geometry,
            None,
            ResourceMode::Auto,
        ).unwrap();
        
        assert!(handler.get_window(id).is_some());
        println!("✅ Test: Pencere oluşturuldu ID: {}", id);
    }

    #[test]
    fn test_window_lifecycle() {
        let handler = WindowHandler::new(ResourceMode::Auto);
        let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
        
        let id = handler.create_window(
            "Test".to_string(),
            "test.app".to_string(),
            geometry,
            None,
            ResourceMode::Auto,
        ).unwrap();
        
        handler.focus_window(id).unwrap();
        assert_eq!(handler.get_focused_window(), Some(id));
        
        handler.close_window(id).unwrap();
        assert!(handler.get_window(id).is_none());
        
        println!("✅ Test: Pencere yaşam döngüsü tamamlandı");
    }

    #[test]
    fn test_manifest_loading() {
        let handler = WindowHandler::new(ResourceMode::Manual);
        
        // Manifest dosyası yoksa skip et
        let manifest_path = "/tmp/test.manifest";
        if std::path::Path::new(manifest_path).exists() {
            let geometry = WindowGeometry { x: 0, y: 0, width: 1024, height: 768 };
            
            match handler.create_window(
                "Manifest Test".to_string(),
                "manifest.test".to_string(),
                geometry,
                Some(manifest_path.to_string()),
                ResourceMode::Manual,
            ) {
                Ok(id) => {
                    if let Some(window) = handler.get_window(id) {
                        println!("✅ Test: Manifest yüklendi");
                        println!("   RAM: {:?} MB", window.resource_limits.max_memory_mb);
                        println!("   GPU: {:?} MB", window.resource_limits.max_gpu_memory_mb);
                        println!("   Renderer: {}", window.resource_limits.renderer);
                    }
                }
                Err(e) => println!("⚠️  Manifest yükleme hatası: {}", e),
            }
        } else {
            println!("⚠️  Test manifest dosyası bulunamadı, test atlandı");
        }
    }

    #[test]
    fn test_resource_adjustment() {
        let handler = WindowHandler::new(ResourceMode::Manual);
        let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
        
        let id = handler.create_window(
            "Resource Test".to_string(),
            "resource.test".to_string(),
            geometry,
            None,
            ResourceMode::Manual,
        ).unwrap();

        let mut new_limits = ResourceLimits::default();
        new_limits.max_memory_mb = Some(1024);
        new_limits.max_gpu_memory_mb = Some(512);
        new_limits.execution_mode = ExecutionMode::GpuOnly;

        handler.adjust_window_resources(id, new_limits.clone()).unwrap();

        if let Some(window) = handler.get_window(id) {
            assert_eq!(window.resource_limits.max_memory_mb, Some(1024));
            assert_eq!(window.resource_limits.max_gpu_memory_mb, Some(512));
            assert_eq!(window.resource_limits.execution_mode, ExecutionMode::GpuOnly);
            println!("✅ Test: Kaynak ayarlama başarılı");
        }
    }

    #[test]
    fn test_parent_child_relationship() {
        let handler = WindowHandler::new(ResourceMode::Auto);
        let geometry = WindowGeometry { x: 0, y: 0, width: 800, height: 600 };
        
        let parent_id = handler.create_window(
            "Parent".to_string(),
            "parent.app".to_string(),
            geometry,
            None,
            ResourceMode::Auto,
        ).unwrap();

        let child_id = handler.create_window(
            "Child".to_string(),
            "child.app".to_string(),
            geometry,
            None,
            ResourceMode::Auto,
        ).unwrap();

        handler.set_parent(child_id, parent_id).unwrap();

        if let Some(parent) = handler.get_window(parent_id) {
            assert!(parent.children_ids.contains(&child_id));
        }

        if let Some(child) = handler.get_window(child_id) {
            assert_eq!(child.parent_id, Some(parent_id));
        }

        println!("✅ Test: Parent-child ilişkisi kuruldu");
    }

    #[test]
    fn test_wasma_config_loading() {
        let handler = WindowHandler::new(ResourceMode::Auto);
        
        // Config dosyası varsa test et
        if std::path::Path::new("/etc/wasma/wasma.in.conf").exists() {
            match handler.load_wasma_config("/etc/wasma/wasma.in.conf") {
                Ok(_) => {
                    println!("✅ Test: WASMA config yüklendi");
                    
                    let config_guard = handler.wasma_config.lock().unwrap();
                    if let Some(ref cfg) = *config_guard {
                        println!("   Renderer: {}", cfg.resource_limits.renderer);
                        println!("   Scope level: {}", cfg.resource_limits.scope_level);
                        println!("   Protocol sayısı: {}", cfg.uri_handling.protocols.len());
                    }
                }
                Err(e) => println!("⚠️  Config yükleme hatası: {}", e),
            }
        } else {
            println!("⚠️  WASMA config dosyası bulunamadı, test atlandı");
        }
    }
}content)
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
            button("Küçült").on_press(Message::MinimizeWindow(window.id)),
            Space::with_width(5),
            button("Büyüt").on_press(Message::MaximizeWindow(window.id)),
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
                    "{} | Durum: {}",
                    mode_str,
                    if usage.task_active { "ÇALIŞIYOR" } else { "DURDURULDU" }
                ))
                .size(14),
                text(format!(
                    "RAM: {} MiB | VRAM: {} MiB | Core: {:?}",
                    usage.ram_allocated_mb, usage.vram_allocated_mb, usage.cpu_cores
                ))
                .size(14),
                text(format!(
                    "GPU: {} | Kalan süre: {}s",
                    usage.gpu_device.unwrap_or_else(|| "Yok".to_string()),
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
            column![text("Kaynak bilgisi yok").size(14)]
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

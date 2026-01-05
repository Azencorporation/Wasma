// src/core/runtime.rs
// UBIN Runtime – Tek Otorite Döngüsü ve Lifecycle Yöneticisi
// Tüm window'ları yönetir, event'leri dispatch eder, render'ı zorlar
// Assignment enforce, lease kontrolü, platform adaptasyonu, convergence tetikleme
// Eternal loop – UBIN sonsuz egemenlik sağlar

use crate::core::abi::{UbinWidget, UbinAction};
use crate::core::convergence::UbinConvergenceEngine;
use crate::platform::{adapt_window_to_platform, UbinPlatform};
use crate::assignment::{Assignment, ExecutionMode};
use crate::WBackend;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::resource_manager::ResourceMode;
/// UBIN Runtime Window – Runtime'da yönetilen her pencere
pub struct UbinRuntimeWindow {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub root_widget: UbinWidget,
    pub assignment: Assignment,
    pub active: bool,
    pub last_frame: Instant,
    pub frame_count: u64,
}

/// UBIN Global Runtime – Tek instance, eternal loop
pub struct UbinRuntime {
    backend: Arc<WBackend>,
    convergence_engine: Mutex<UbinConvergenceEngine>,
    windows: HashMap<u32, UbinRuntimeWindow>,
    next_window_id: u32,
    start_time: Instant,
    running: bool,
}

impl UbinRuntime {
    /// UBIN runtime başlatılır – tek otorite kurulur
    pub fn initialize() -> Self {
        let backend = Arc::new(WBackend::new(ResourceMode::Auto));
        let convergence_engine = UbinConvergenceEngine::initiate_global_convergence();

        println!("♾️ UBIN RUNTIME INITIALIZED – Eternal dominion cycle ready");

        UbinRuntime {
            backend,
            convergence_engine: Mutex::new(convergence_engine),
            windows: HashMap::new(),
            next_window_id: 1,
            start_time: Instant::now(),
            running: true,
        }
    }

    /// Yeni window spawn eder – UBIN ABI ile
    pub fn spawn_window(&mut self, title: String, width: u32, height: u32, root_widget: UbinWidget, mode: ExecutionMode) -> u32 {
        let mut assignment = Assignment::new(self.next_window_id);
        assignment.execution_mode = mode;
        assignment.bind_cpu();
        if assignment.should_bind_gpu() {
            assignment.bind_gpu();
        }
        assignment.start_lease(Duration::from_secs(300));
        assignment.start_task();

        self.backend.add_assignment(assignment.clone());

        let window_id = self.next_window_id;
        self.next_window_id += 1;

        let window = UbinRuntimeWindow {
            id: window_id,
            title,
            width,
            height,
            root_widget,
            assignment,
            active: true,
            last_frame: Instant::now(),
            frame_count: 0,
        };

        // Platform adaptasyonu
        adapt_window_to_platform(&mut window);  // platform/mod.rs'den

        // Convergence uygula
        self.convergence_engine.lock().unwrap().apply_convergence_to_window(&mut window);

        self.windows.insert(window_id, window);

        println!("🖥️ UBIN window spawned – ID: {} | Title: '{}' | Assignment {}", window_id, title, assignment.id);

        window_id
    }

    /// Ana eternal döngü – 60 FPS hedef
    pub fn run_eternal_dominion(&mut self) {
        println!("🔄 UBIN ETERNAL DOMINION CYCLE STARTED – No escape from authority");

        while self.running && !self.windows.is_empty() {
            let frame_start = Instant::now();

            // Global backend cycle – tüm assignment'lar için enforce
            self.backend.run_cycle();

            // Tüm window'ları işle
            let mut terminated = vec![];

            for (id, window) in self.windows.iter_mut() {
                if window.assignment.lease_expired() {
                    window.active = false;
                    terminated.push(*id);
                    println!("⏰ Lease expired – Window {} terminated by UBIN authority", id);
                    continue;
                }

                // Render cycle
                self.render_frame(*id, window);

                // Simulated events – gerçekte platformdan gelecek
                self.dispatch_simulated_events(*id, window);

                window.frame_count += 1;
            }

            // Temizle
            for id in terminated {
                if let Some(window) = self.windows.remove(&id) {
                    window.assignment.stop_task();
                    println!("🧹 Window {} cleaned up – Task stopped", id);
                }
            }

            // FPS kontrolü
            let frame_time = frame_start.elapsed();
            if frame_time < Duration::from_millis(16) {
                std::thread::sleep(Duration::from_millis(16) - frame_time);
            }
        }

        println!("🏁 UBIN eternal dominion ended – All windows terminated gracefully");
    }

    /// Tek frame render – platforma zorla
    fn render_frame(&self, window_id: u32, window: &mut UbinRuntimeWindow) {
        window.last_frame = Instant::now();

        println!(
            "🎨 UBIN rendering frame {} for window {} – '{}' | FPS: {:.1}",
            window.frame_count,
            window_id,
            window.title,
            1.0 / window.last_frame.elapsed().as_secs_f32().max(0.001)
        );

        // Gerçekte burada platform render çağrısı olacak
        // fallback.rs veya platform adaptörleri
    }

    /// Simulated events – test için
    fn dispatch_simulated_events(&self, window_id: u32, window: &mut UbinRuntimeWindow) {
        // Her 100 frame'de bir lease yenile
        if window.frame_count % 100 == 0 {
            window.assignment.start_lease(Duration::from_secs(300));
            println!("🔄 Simulated event – Lease renewed for window {}", window_id);
        }

        // Her 500 frame'de bir close simüle
        if window.frame_count % 500 == 0 && window.frame_count > 100 {
            window.active = false;
            println!("🛑 Simulated event – Close requested for window {}", window_id);
        }
    }

    /// Runtime'ı durdur
    pub fn shutdown(&mut self) {
        self.running = false;
        println!("🛑 UBIN runtime shutdown requested – Terminating all windows");
    }

    /// Aktif window sayısı
    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }
}

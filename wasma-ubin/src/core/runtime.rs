// src/core/runtime.rs
// UBIN Runtime – Tek Otorite Döngüsü ve Lifecycle Yöneticisi

use crate::core::abi::UbinWidget;
use crate::core::convergence::UbinConvergenceEngine;
use crate::platform::adapt_window_to_platform;
// DÜZELTME: wbackend'den import
use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// UBIN Runtime Window
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

/// UBIN Global Runtime
pub struct UbinRuntime {
    backend: Arc<WBackend>,
    convergence_engine: Mutex<UbinConvergenceEngine>,
    windows: HashMap<u32, UbinRuntimeWindow>,
    next_window_id: u32,
    start_time: Instant,
    running: bool,
}

impl UbinRuntime {
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

        let mut window = UbinRuntimeWindow {
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

        adapt_window_to_platform(&mut window);
        self.convergence_engine.lock().unwrap().apply_convergence_to_window(&mut window);

        println!("🖥️ UBIN window spawned – ID: {} | Title: '{}'", window_id, window.title);
        
        self.windows.insert(window_id, window);
        window_id
    }

pub fn run_eternal_dominion(&mut self) {
    println!("🔄 UBIN ETERNAL DOMINION CYCLE STARTED");

    while self.running && !self.windows.is_empty() {
        let frame_start = Instant::now();

        self.backend.run_cycle();

        // 1. Window id’lerini önceden topla
        let window_ids: Vec<u32> = self.windows.keys().cloned().collect();

        let mut terminated = vec![];

        for id in window_ids {
            // remove ile ownership al → E0502 hatası kalkar
            if let Some(mut window) = self.windows.remove(&id) {
                if window.assignment.lease_expired() {
                    window.active = false;
                    terminated.push(id);
                    println!("⏰ Lease expired – Window {} terminated", id);
                    continue;
                }

                // render ve event dispatch artık mutable ownership ile çalışıyor
                self.render_frame(id, &mut window);
                self.dispatch_simulated_events(id, &mut window);
                window.frame_count += 1;

                // ownership geri HashMap’e ekle
                self.windows.insert(id, window);
            }
        }

        for id in terminated {
            if let Some(mut window) = self.windows.remove(&id) {
                window.assignment.stop_task();
                println!("🧹 Window {} cleaned up", id);
            }
        }

        let frame_time = frame_start.elapsed();
        if frame_time < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - frame_time);
        }
    }

    println!("🏁 UBIN eternal dominion ended");
}

    fn render_frame(&self, window_id: u32, window: &mut UbinRuntimeWindow) {
        window.last_frame = Instant::now();
        println!("🎨 Rendering frame {} for window {}", window.frame_count, window_id);
    }

    fn dispatch_simulated_events(&self, window_id: u32, window: &mut UbinRuntimeWindow) {
        if window.frame_count % 100 == 0 {
            window.assignment.start_lease(Duration::from_secs(300));
            println!("🔄 Lease renewed for window {}", window_id);
        }

        if window.frame_count % 500 == 0 && window.frame_count > 100 {
            window.active = false;
            println!("🛑 Close requested for window {}", window_id);
        }
    }

    pub fn shutdown(&mut self) {
        self.running = false;
        println!("🛑 UBIN runtime shutdown requested");
    }

    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }
}

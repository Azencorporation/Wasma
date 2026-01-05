// src/assignment.rs
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use core_affinity;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum ExecutionMode {
    #[clap(alias = "cpu")]
    CpuOnly,
    #[clap(alias = "gpu-pref")]
    GpuPreferred,
    #[clap(alias = "gpu-only")]
    GpuOnly,
    #[clap(alias = "hybrid")]
    Hybrid,
}

#[derive(Debug)]
pub struct Assignment {
    pub id: u32,
    pub cpu_cores: Vec<usize>,
    pub gpu_device: Option<String>,
    pub ram_limit: usize,
    pub vram_limit: usize,

    pub cpu_priority: u8,
    pub gpu_priority: u8,
    pub cpu_affinity_mask: Option<u64>,
    pub gpu_id: Option<String>,

    pub lease_duration: Option<Duration>,
    pub lease_start: Option<Instant>,

    pub task_handle: Option<JoinHandle<()>>,
    pub task_active: Arc<Mutex<bool>>,
    pub cgroup_path: Option<String>,

    pub execution_mode: ExecutionMode,
}

impl Assignment {
    pub fn new(id: u32) -> Self {
        Assignment {
            id,
            cpu_cores: Vec::new(),
            gpu_device: None,
            ram_limit: 512 * 1024 * 1024,
            vram_limit: 256 * 1024 * 1024,
            cpu_priority: 5,
            gpu_priority: 5,
            cpu_affinity_mask: None,
            gpu_id: None,
            lease_duration: None,
            lease_start: None,
            task_handle: None,
            task_active: Arc::new(Mutex::new(false)),
            cgroup_path: None,
            execution_mode: ExecutionMode::GpuPreferred,
        }
    }

    pub fn start_lease(&mut self, dur: Duration) {
        self.lease_duration = Some(dur);
        self.lease_start = Some(Instant::now());
    }

    pub fn lease_expired(&self) -> bool {
        match (self.lease_duration, self.lease_start) {
            (Some(d), Some(s)) => s.elapsed() >= d,
            _ => false,
        }
    }

    pub fn bind_cpu(&mut self) {
        if self.cpu_cores.is_empty() {
            if let Some(core_ids) = core_affinity::get_core_ids() {
                if let Some(core) = core_ids.first() {
                    self.cpu_cores.push(core.id);
                    let _ = core_affinity::set_for_current(*core);
                    println!("🔗 CPU pinned to core {} for assignment {}", core.id, self.id);
                }
            }
        }
    }

    pub fn bind_gpu(&mut self) {
        if !self.should_bind_gpu() {
            return;
        }

        #[cfg(target_os = "linux")]
        {
            // 1. NVIDIA dGPU
            if std::process::Command::new("nvidia-smi")
                .arg("--query-gpu=name")
                .output()
                .is_ok()
            {
                self.gpu_device = Some("nvidia-dgpu".to_string());
                self.gpu_id = Some("cuda:0".to_string());
                println!("✅ Discrete GPU detected: NVIDIA dGPU");
                return;
            }

            // 2. AMD dGPU
            if std::process::Command::new("rocminfo").output().is_ok() {
                self.gpu_device = Some("amd-dgpu".to_string());
                self.gpu_id = Some("rocm:0".to_string());
                println!("✅ Discrete GPU detected: AMD dGPU");
                return;
            }

            // 3. iGPU – /dev/dri üzerinden
            if std::path::Path::new("/dev/dri/renderD128").exists()
                || std::path::Path::new("/dev/dri/renderD129").exists()
                || std::path::Path::new("/dev/dri/card0").exists()
                || std::path::Path::new("/dev/dri/card1").exists()
            {
                self.gpu_device = Some("integrated-gpu".to_string());
                self.gpu_id = Some("igpu:0".to_string());
                println!("✅ Integrated GPU (iGPU) detected via /dev/dri");
                return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            self.gpu_device = Some("apple-igpu".to_string());
            self.gpu_id = Some("metal:0".to_string());
            println!("✅ Apple Silicon iGPU detected");
            return;
        }

        #[cfg(target_os = "windows")]
        {
            self.gpu_device = Some("windows-igpu".to_string());
            self.gpu_id = Some("dxgi:0".to_string());
            println("✅ Windows GPU detected (likely iGPU)");
            return;
        }

        // Hiç GPU bulunamadı
        println!("ℹ️ No GPU detected – falling back to CPU-only mode");
    }

    pub fn should_bind_gpu(&self) -> bool {
        !matches!(self.execution_mode, ExecutionMode::CpuOnly)
    }

    pub fn requires_gpu(&self) -> bool {
        matches!(self.execution_mode, ExecutionMode::GpuOnly | ExecutionMode::Hybrid)
    }

    pub fn start_task(&mut self) {
        if self.task_handle.is_some() {
            return;
        }

        let task_active = Arc::clone(&self.task_active);
        let id = self.id;
        let cpu_cores = self.cpu_cores.clone();
        let gpu_device = self.gpu_device.clone();
        let mode = self.execution_mode;

        let handle = thread::spawn(move || {
            // CPU affinity tekrardan uygula (thread içinde)
            if let Some(core_id) = cpu_cores.first() {
                if let Some(cores) = core_affinity::get_core_ids() {
                    if let Some(core) = cores.iter().find(|c| c.id == *core_id) {
                        let _ = core_affinity::set_for_current(*core);
                    }
                }
            }

            let mode_str = match mode {
                ExecutionMode::CpuOnly => "🔵 CPU-Only",
                ExecutionMode::GpuPreferred => "🟢 GPU Preferred",
                ExecutionMode::GpuOnly => "🟡 GPU-Only",
                ExecutionMode::Hybrid => "⚡ Hybrid",
            };

            println!("🚀 Task {} STARTED → {}", id, mode_str);

            let mut counter = 0u64;
            while *task_active.lock().unwrap() {
                counter += 1;
                if counter % 5_000_000 == 0 {  // Çıktıyı seyrelttik
                    println!("📊 Assignment {} alive ({}M cycles) | GPU: {:?}", id, counter / 1_000_000, gpu_device);
                }
                thread::yield_now();
            }

            println!("🛑 Task {} terminated", id);
        });

        self.task_handle = Some(handle);
        *self.task_active.lock().unwrap() = true;
    }

    pub fn stop_task(&mut self) {
        *self.task_active.lock().unwrap() = false;
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.join();
            println!("🛑 Task {} gracefully stopped", self.id);
        }
    }
}

// JoinHandle Clone edilemediği için manuel Clone
impl Clone for Assignment {
    fn clone(&self) -> Self {
        Assignment {
            id: self.id,
            cpu_cores: self.cpu_cores.clone(),
            gpu_device: self.gpu_device.clone(),
            ram_limit: self.ram_limit,
            vram_limit: self.vram_limit,
            cpu_priority: self.cpu_priority,
            gpu_priority: self.gpu_priority,
            cpu_affinity_mask: self.cpu_affinity_mask,
            gpu_id: self.gpu_id.clone(),
            lease_duration: self.lease_duration,
            lease_start: self.lease_start,
            task_handle: None,
            task_active: Arc::new(Mutex::new(*self.task_active.lock().unwrap())),
            cgroup_path: self.cgroup_path.clone(),
            execution_mode: self.execution_mode,
        }
    }
}

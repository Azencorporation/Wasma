// src/scheduler.rs
use crate::assignment::{Assignment, ExecutionMode};

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Scheduler
    }

    /// Scheduler'ın ana görevi: Assignment'ı doğrula ve çalıştırılabilirliğini onayla
    /// Gerçek task execution ResourceManager tarafından yapılıyor (start_task)
    /// Burada sadece scheduling kararı + monitoring logu basıyoruz
    pub fn schedule(&self, assignment: &Assignment) {
        // 1. Lease kontrolü
        if assignment.lease_expired() {
            println!("⏰ Scheduler: Skipping EXPIRED assignment {}", assignment.id);
            return;
        }

        // 2. Task durumu kontrolü
        let task_running = assignment.task_handle.is_some() && *assignment.task_active.lock().unwrap();

        // 3. Execution Mode'a göre net tanımlama
        let mode_str = if task_running {
            match (assignment.execution_mode, assignment.gpu_device.is_some()) {
                (ExecutionMode::CpuOnly, _) => "🔵 CPU-Only (Deterministic)",
                (ExecutionMode::GpuPreferred, true) => "🟢 Hybrid (GPU Active)",
                (ExecutionMode::GpuPreferred, false) => "🔵 CPU-Only (GPU Unavailable)",
                (ExecutionMode::GpuOnly, true) => "🟡 GPU-Only (Enforced)",
                (ExecutionMode::GpuOnly, false) => "⚠️  GPU-Only Requested but Unavailable",
                (ExecutionMode::Hybrid, true) => "⚡ Full Hybrid Mode",
                (ExecutionMode::Hybrid, false) => "🔵 Hybrid → Fallback to CPU-Only",
            }
        } else {
            "⏳ Not Started Yet"
        };

        // 4. CPU core bilgisi
        let cpu_info = if assignment.cpu_cores.is_empty() {
            "No affinity".to_string()
        } else {
            format!("Cores {:?}", assignment.cpu_cores)
        };

        // 5. RAM / VRAM
        let ram_mb = assignment.ram_limit / (1024 * 1024);
        let vram_mb = assignment.vram_limit / (1024 * 1024);

        // 6. Ana scheduling logu – WASMA otoritesi burada konuşuyor
        println!(
            "🗓️  Scheduler: EXECUTING Assignment {:2} | {} | {} | RAM: {:4} MiB | VRAM: {:3} MiB | GPU: {:?}",
            assignment.id,
            mode_str,
            cpu_info,
            ram_mb,
            vram_mb,
            assignment.gpu_device.as_deref().unwrap_or("None")
        );

        // Gelecekte buraya eklenebilir:
        // - Priority queue (cpu_priority'ye göre sıralama)
        // - Fair sharing (round-robin)
        // - Preemptive scheduling
        // - Real-time guarantees
    }
}

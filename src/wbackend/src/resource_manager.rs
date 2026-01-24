// src/resource_manager.rs
use crate::assignment::{Assignment, ExecutionMode};
use std::collections::HashMap;
use std::time::Duration;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum ResourceMode {
    #[clap(alias = "m")]
    Manual,
    #[clap(alias = "a")]
    Auto,
}

pub struct ResourceManager {
    mode: ResourceMode,
}

impl ResourceManager {
    pub fn new(mode: ResourceMode) -> Self {
        ResourceManager { mode }
    }

    pub fn allocate(&self, assignment: &mut Assignment) {
        match self.mode {
            ResourceMode::Manual => {
                println!("📋 Manual mode: Assignment {} – configure manually", assignment.id);
            }
            ResourceMode::Auto => {
                // CPU her zaman bind edilir
                if assignment.cpu_cores.is_empty() {
                    assignment.bind_cpu();
                }

                // GPU sadece gerekliyse ve mevcutsa bind et
                if assignment.gpu_device.is_none() && assignment.should_bind_gpu() {
                    assignment.bind_gpu();
                }

                // Task başlat
                if assignment.task_handle.is_none() {
                    assignment.start_task();
                }

                // Lease başlat
                if assignment.lease_start.is_none() {
                    assignment.start_lease(Duration::from_secs(30));
                }

                // Kullanıcıya bilgi
                let mode_str = match assignment.execution_mode {
                    ExecutionMode::CpuOnly => "🔵 Pure CPU-Only",
                    ExecutionMode::GpuPreferred => "🟢 GPU Preferred",
                    ExecutionMode::GpuOnly => "🟡 Strict GPU-Only",
                    ExecutionMode::Hybrid => "⚡ Full Hybrid",
                };

                println!("✅ Allocation complete → Assignment {} | Requested Mode: {}", assignment.id, mode_str);
            }
        }
    }

    pub fn enforce_leases(&self, assignments: &mut HashMap<u32, Assignment>) {
        let expired_ids: Vec<u32> = assignments
            .iter()
            .filter(|(_, a)| a.lease_expired())
            .map(|(&id, _)| id)
            .collect();

        for id in expired_ids {
            if let Some(mut expired) = assignments.remove(&id) {
                println!("🗑️ Lease expired → Gracefully stopping and removing assignment {}", id);
                expired.stop_task();
            }
        }
    }

    pub fn monitor(&self, assignments: &HashMap<u32, Assignment>) {
        println!("\n🌀 WASMA v1.0 – Live Resource Monitor (2 Ocak 2026) 🌀\n");

        if assignments.is_empty() {
            println!("   📭 No active assignments currently.\n");
            return;
        }

        for (_, a) in assignments {
            let task_status = if a.task_handle.is_some() && *a.task_active.lock().unwrap() {
                "🟢 RUNNING"
            } else {
                "🔴 STOPPED"
            };

            // Gerçek GPU durumuna göre akıllı sınıflandırma
            let effective_mode = match a.gpu_device.as_deref() {
                Some("nvidia-dgpu") => "🟢 Discrete GPU (NVIDIA dGPU)",
                Some("amd-dgpu") => "🟢 Discrete GPU (AMD dGPU)",
                Some("integrated-gpu") => "🟡 Integrated GPU (iGPU via /dev/dri)",
                Some("apple-igpu") => "🟡 Apple Silicon iGPU",
                Some("windows-igpu") => "🟡 Windows iGPU",
                None => "🔵 Pure CPU-Only (No GPU available)",
                _ => "🟢 GPU Active",
            };
            let requested_mode = match a.execution_mode {
                ExecutionMode::CpuOnly => "Requested: Pure CPU",
                ExecutionMode::GpuPreferred => "Requested: GPU Preferred",
                ExecutionMode::GpuOnly => "Requested: Strict GPU",
                ExecutionMode::Hybrid => "Requested: Full Hybrid",
            };

            let remaining = a
                .lease_duration
                .and_then(|d| a.lease_start.map(|s| d.as_secs().saturating_sub(s.elapsed().as_secs())))
                .unwrap_or(0);

            println!(
                "ID {:2} | {} | {} | {} | Cores: {:?} | GPU: {:18} | RAM: {:4} MiB | Lease: {}s",
                a.id,
                task_status,
                effective_mode,
                requested_mode,
                a.cpu_cores,
                a.gpu_device.as_deref().unwrap_or("None"),
                a.ram_limit >> 20,
                remaining
            );
        }

        println!("──────────────────────────────────────────────────────────────────────\n");
    }
}

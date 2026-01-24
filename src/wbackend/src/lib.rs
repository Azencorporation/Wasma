// src/lib.rs
pub mod assignment;
pub mod resource_manager;
pub mod scheduler;

pub use assignment::{Assignment, ExecutionMode};
pub use resource_manager::{ResourceManager, ResourceMode};
pub use scheduler::Scheduler;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// WASMA'nın ana backend'i – Resource-first otorite merkezi
pub struct WBackend {
    pub scheduler: Scheduler,
    pub resource_manager: ResourceManager,

    // Thread-safe assignment yönetimi
    assignments: Arc<Mutex<HashMap<u32, Assignment>>>,

    mode: ResourceMode,
}

impl WBackend {
    pub fn new(mode: ResourceMode) -> Self {
        WBackend {
            scheduler: Scheduler::new(),
            resource_manager: ResourceManager::new(mode),
            assignments: Arc::new(Mutex::new(HashMap::new())),
            mode,
        }
    }

    /// Yeni assignment ekle
    pub fn add_assignment(&self, mut assignment: Assignment) {
        let id = assignment.id;

        if self.mode == ResourceMode::Auto {
            // CPU binding
            if assignment.cpu_cores.is_empty() {
                assignment.bind_cpu();
            }

            // GPU binding (opsiyonel)
            if assignment.gpu_device.is_none() && assignment.should_bind_gpu() {
                assignment.bind_gpu();
            }

            // Lease başlat
            if assignment.lease_start.is_none() {
                assignment.start_lease(std::time::Duration::from_secs(30));
            }

            // Task'ı hemen başlat
            if assignment.task_handle.is_none() {
                assignment.start_task();
            }
        } else {
            // Manual mod: sadece lease
            if assignment.lease_start.is_none() {
                assignment.start_lease(std::time::Duration::from_secs(30));
            }
        }

        // HashMap'e ekle
        let mut assignments = self.assignments.lock().unwrap();
        assignments.insert(id, assignment);
        println!("➕ Assignment {} added to WBackend | Mode: {:?}", id, self.mode);
    }

    /// Ana döngü – WASMA'nın kalbi
    pub fn run_cycle(&self) {
        let mut assignments = self.assignments.lock().unwrap();

        // 1. Allocate + Schedule
        for assignment in assignments.values_mut() {
            self.resource_manager.allocate(assignment);
            self.scheduler.schedule(assignment);
        }

        // 2. Lease enforce
        self.resource_manager.enforce_leases(&mut assignments);

        // 3. Monitor
        self.resource_manager.monitor(&assignments);
    }

    /// Yardımcı: Aktif assignment listesi
    pub fn list_assignments(&self) -> Vec<Assignment> {
        let assignments = self.assignments.lock().unwrap();
        assignments.values().cloned().collect()
    }

    /// Yardımcı: ID ile assignment al
    pub fn get_assignment(&self, id: u32) -> Option<Assignment> {
        let assignments = self.assignments.lock().unwrap();
        assignments.get(&id).cloned()
    }
}

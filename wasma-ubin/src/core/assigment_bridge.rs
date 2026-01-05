// src/core/assignment_bridge.rs
// UBIN Assignment Bridge – UBIN ile WASMA backend arasında köprü
// UBIN window'ları Assignment'a bağlar, lease/task enforce eder
// Backend'den Assignment alır, UBIN'e zorla uygular

use crate::Assignment::{Assignment, ExecutionMode};
use crate::WBackend;
use crate::resource_manager::ResourceMode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// UBIN Assignment Bridge – Backend ile UBIN runtime arasında iletişim
pub struct UbinAssignmentBridge {
    backend: Arc<WBackend>,
}

impl UbinAssignmentBridge {
    /// Bridge kurulur – backend zorunlu
    pub fn new(resource_mode: ResourceMode) -> Self {
        let backend = Arc::new(WBackend::new(resource_mode));
        println!("🔗 UBIN ASSIGNMENT BRIDGE ESTABLISHED – Backend linked to UBIN runtime");

        UbinAssignmentBridge { backend }
    }

    /// Yeni Assignment yarat ve backend'e ekle
    pub fn create_assignment(&self, mode: ExecutionMode) -> Assignment {
        let mut assignment = Assignment::new(self.backend.list_assignments().len() as u32 + 1);
        assignment.execution_mode = mode;

        // Otomatik bind ve task başlat
        assignment.bind_cpu();
        if assignment.should_bind_gpu() {
            assignment.bind_gpu();
        }
        assignment.start_lease(Duration::from_secs(300));
        assignment.start_task();

        // Backend'e ekle
        self.backend.add_assignment(assignment.clone());

        println!("🆕 UBIN new assignment created – ID: {} | Mode: {:?}", assignment.id, mode);

        assignment
    }

    /// Varolan Assignment'ı güncelle (lease yenile, mode değiştir vs.)
    pub fn update_assignment(&self, assignment: &mut Assignment, new_mode: Option<ExecutionMode>, renew_lease: bool) {
        if let Some(mode) = new_mode {
            assignment.execution_mode = mode;
            if assignment.should_bind_gpu() {
                assignment.bind_gpu();
            }
            println!("🔄 Assignment {} mode updated to {:?}", assignment.id, mode);
        }

        if renew_lease {
            assignment.start_lease(Duration::from_secs(300));
            println!("🔄 Assignment {} lease renewed (300s)", assignment.id);
        }

        // Backend cycle – enforce
        self.backend.run_cycle();
    }

    /// Assignment'ı backend'den kaldır ve task'ı durdur
    pub fn terminate_assignment(&self, assignment: &mut Assignment) {
        assignment.stop_task();
        println!("🛑 Assignment {} terminated – Task stopped", assignment.id);

        // Backend cycle – cleanup
        self.backend.run_cycle();
    }

    /// Aktif Assignment listesini al
    pub fn get_active_assignments(&self) -> Vec<Assignment> {
        let assignments = self.backend.list_assignments();
        println!("📋 UBIN bridge reports {} active assignments", assignments.len());
        assignments
    }

    /// Backend monitor çıktısını tetikle
    pub fn monitor_backend(&self) {
        self.backend.run_cycle();  // monitor içinde
    }
}

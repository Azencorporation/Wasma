use core_affinity::{get_core_ids, set_for_current};
use libc::{sched_param, pthread_setschedparam, SCHED_FIFO};
use crate::protocols::WasmaConfig;

pub struct ResourcerEngineering {
    config: WasmaConfig,
}

impl ResourcerEngineering {
    pub fn new(config: WasmaConfig) -> Self {
        Self { config }
    }

    pub fn apply_hardware_affinity(&self) {
        let is_nonprof = false;
        if is_nonprof {
            self.apply_dynamic_allocation();
        } else {
            self.apply_static_allocation();
        }
        self.set_realtime_priority();
    }

    fn apply_static_allocation(&self) {
        let core_ids = get_core_ids().expect("Cores missing");
        let static_core_count = 5;
        for i in 0..static_core_count {
            if i < core_ids.len() {
                set_for_current(core_ids[i]);
            }
        }
        if self.config.resource_limits.scope_level > 0 {
            self.init_gpu_context(2048);
        }
    }

    fn apply_dynamic_allocation(&self) {
        let core_ids = get_core_ids().expect("Cores missing");
        if let Some(core) = core_ids.first() {
            set_for_current(*core);
        }
    }

    fn init_gpu_context(&self, vram_mb: u64) {
        unsafe {
            let level = self.config.resource_limits.scope_level;
            self.enable_glx_pipeline(level, vram_mb);
        }
    }

    fn enable_glx_pipeline(&self, level: u32, vram_mb: u64) {
        let _allocation_size = vram_mb * 1024 * 1024;
        let _scale = level as f32 / 100.0;
    }

    fn set_realtime_priority(&self) {
        unsafe {
            let param = sched_param { sched_priority: 99 };
            let thread_id = libc::pthread_self();
            pthread_setschedparam(thread_id, SCHED_FIFO, &param);
        }
    }

    pub fn isolate_task_by_config(&self) {
        let core_ids = get_core_ids().unwrap();
        if core_ids.len() > 2 {
            set_for_current(core_ids[2]);
        }
    }
}

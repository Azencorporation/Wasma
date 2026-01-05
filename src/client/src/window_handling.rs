// window_handling.rs
// WASMA Universal Window Handling – iced + WASMA library integration
// User manages assignments from the window (add, start, stop, lease, monitor)

use iced::{
    widget::{column, row, text, button, scrollable, container, slider},
    Alignment, Element, Length, Sandbox, Settings, Theme,
};
use iced::alignment::Horizontal;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::assignment::{Assignment, ExecutionMode}; // WASMA library
use crate::resource_manager::{ResourceManager, ResourceMode};
use crate::scheduler::Scheduler;
use crate::WBackend; // WASMA backend

#[derive(Debug, Clone)]
pub enum Message {
    AddAssignment,
    StartTask(u32),
    StopTask(u32),
    RenewLease(u32, u64), // seconds
    ChangeMode(u32, ExecutionMode),
    RefreshMonitor,
    Tick, // lease check every second
}

pub struct WasmaWindow {
    backend: Arc<WBackend>,
    assignments: Vec<Assignment>,
    selected_id: Option<u32>,
    lease_input: u64, // new lease duration in seconds
}

impl Sandbox for WasmaWindow {
    type Message = Message;

    fn new() -> Self {
        let backend = Arc::new(WBackend::new(ResourceMode::Auto));
        WasmaWindow {
            backend,
            assignments: Vec::new(),
            selected_id: None,
            lease_input: 300,
        }
    }

    fn title(&self) -> String {
        "WASMA Authority Window – Resource Manager 🌀".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AddAssignment => {
                let id = (self.assignments.len() as u32) + 1;
                let mut assignment = Assignment::new(id);
                assignment.execution_mode = ExecutionMode::GpuPreferred;
                self.backend.add_assignment(assignment.clone());
                self.assignments.push(assignment);
            }

            Message::StartTask(id) => {
                if let Some(ass) = self.assignments.iter_mut().find(|a| a.id == id) {
                    ass.start_task();
                    println!("🚀 Task {} started", id);
                }
            }

            Message::StopTask(id) => {
                if let Some(ass) = self.assignments.iter_mut().find(|a| a.id == id) {
                    ass.stop_task();
                    println!("🛑 Task {} stopped", id);
                }
            }

            Message::RenewLease(id, secs) => {
                if let Some(ass) = self.assignments.iter_mut().find(|a| a.id == id) {
                    ass.start_lease(Duration::from_secs(secs));
                    println!("🔄 Lease renewed: {} → {}s", id, secs);
                }
            }

            Message::ChangeMode(id, mode) => {
                if let Some(ass) = self.assignments.iter_mut().find(|a| a.id == id) {
                    ass.execution_mode = mode;
                    ass.bind_gpu(); // re-check GPU when mode changes
                    println!("🔄 Mode changed: {} → {:?}", id, mode);
                }
            }

            Message::RefreshMonitor => {
                self.assignments = self.backend.list_assignments();
                self.backend.run_cycle(); // run backend cycle
            }

            Message::Tick => {
                // Expired lease handling is automatic in backend
                self.assignments.retain(|a| !a.lease_expired());
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = text("WASMA AUTHORITY CONTROL PANEL 🌀")
            .size(32)
            .horizontal_alignment(Horizontal::Center);

        let add_btn = button("Add New Assignment")
            .padding(15)
            .on_press(Message::AddAssignment);

        let monitor_btn = button("Refresh & Monitor")
            .padding(15)
            .on_press(Message::RefreshMonitor);

        let control_panel = row![
            add_btn,
            monitor_btn,
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        let assignment_list = self.assignments.iter().fold(
            column![].spacing(10),
            |list, ass| {
                let id = ass.id;
                let status = if ass.task_active.lock().unwrap().clone() { "🟢 ACTIVE" } else { "🔴 STOPPED" };
                let mode = format!("{:?}", ass.execution_mode);
                let gpu = ass.gpu_device.as_deref().unwrap_or("None");
                let lease = ass.lease_remaining().unwrap_or(0);

                let row = row![
                    text(format!("ID: {}", id)).width(Length::FillPortion(1)),
                    text(status).width(Length::FillPortion(1)),
                    text(mode).width(Length::FillPortion(2)),
                    text(gpu).width(Length::FillPortion(2)),
                    text(format!("Lease: {}s", lease)).width(Length::FillPortion(1)),
                    button("Start").on_press(Message::StartTask(id)),
                    button("Stop").on_press(Message::StopTask(id)),
                    button("Renew Lease").on_press(Message::RenewLease(id, 300)),
                ]
                .spacing(15)
                .align_items(Alignment::Center);

                list.push(row)
            },
        );

        let scrollable_list = scrollable(assignment_list);

        column![
            header,
            control_panel,
            text("Active Assignments").size(24),
            scrollable_list,
        ]
        .spacing(20)
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

// Helper function – remaining lease seconds
impl Assignment {
    fn lease_remaining(&self) -> Option<u64> {
        match (self.lease_duration, self.lease_start) {
            (Some(d), Some(s)) => Some(d.as_secs().saturating_sub(s.elapsed().as_secs())),
            _ => None,
        }
    }
}

// Start the WASMA window
pub fn run_wasma_window() -> iced::Result {
    WasmaWindow::run(Settings::default())
}

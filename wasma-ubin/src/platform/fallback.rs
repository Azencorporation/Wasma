// src/platform/fallback.rs
// UBIN Fallback Platform Adaptör – Ghost Mod & Safety Net
// Platform adaptörleri başarısız olursa veya ghost mod aktifse devreye girer
// Tam iced + wgpu tabanlı – zero native dependency
// UBIN portable widget ABI'sini doğrudan render eder

use crate::core::abi::{UbinWidget, UbinAction, UbinLayoutDirection};
use iced::{
    widget::{button, column, container, row, text, text_input, scrollable, progress_bar, slider, checkbox},
    Alignment, Element, Length, Theme, Application, Command, Settings as IcedSettings,
};

use iced::theme;
/// Fallback Runtime – iced Application
pub struct UbinFallbackApp {
    windows: Vec<UbinFallbackWindow>,
}

struct UbinFallbackWindow {
    pub id: u32,
    pub title: String,
    pub root_widget: UbinWidget,
    pub assignment_id: u32,
}

#[derive(Debug, Clone)]
pub enum FallbackMessage {
    UbinAction(UbinAction, u32), // action + window_id
    NoOp,
}

impl Application for UbinFallbackApp {
    type Executor = iced::executor::Default;
    type Message = FallbackMessage;
    type Theme = Theme;
    type Flags = Vec<(u32, String, UbinWidget, u32)>;// id, title, root_widget, assignment_id

    fn new(flags: Self::Flags) -> (Self, Command<FallbackMessage>) {
        let windows = flags.into_iter().map(|(id, title, root_widget, assignment_id)| UbinFallbackWindow {
            id,
            title,
            root_widget,
            assignment_id,
        }).collect();

        println!("🌑 UBIN FALLBACK MODE ACTIVATED – Ghost rendering engaged");
        println!("   {} windows loaded in pure GPU mode", windows.len());

        (UbinFallbackApp { windows }, Command::none())
    }

    fn title(&self) -> String {
        "UBIN Ghost Dominion – Fallback Runtime 🌀".to_string()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: FallbackMessage) -> Command<FallbackMessage> {
        match message {
            FallbackMessage::UbinAction(action, _window_id) => {
                println!("⚡ Fallback action triggered: {:?}", action);
                // Gerçekte backend'e yönlendirilecek
            }
            FallbackMessage::NoOp => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<FallbackMessage> {
        if self.windows.is_empty() {
            return container(text("No active UBIN windows in fallback mode"))
                .center_x()
                .center_y()
                .into();
        }

        let mut content = column![].spacing(30).padding(20);

        for window in &self.windows {
            let window_view = container(
                column![
                    text(&window.title).size(32),
                    text(format!("Assignment ID: {}", window.assignment_id)).size(20),
                    self.translate_widget_to_iced(&window.root_widget, window.id),
                ]
                .spacing(20)
                .align_items(Alignment::Center)
            )
            .style(theme::Container::Box)
            .padding(20);

            content = content.push(window_view);
        }

        scrollable(content).into()
    }
}

impl UbinFallbackApp {
    fn translate_widget_to_iced(&self, widget: &UbinWidget, window_id: u32) -> Element<FallbackMessage> {
        match widget {
            UbinWidget::Window { title, child, .. } => {
                column![
                    text(title).size(28),
                    self.translate_widget_to_iced(child, window_id),
                ]
                .spacing(20)
                .into()
            }
            UbinWidget::Button { label, action, enabled, .. } => {
                let mut btn = button(text(label).size(24)).padding(15);
                if *enabled {
                    btn = btn.on_press(FallbackMessage::UbinAction(action.clone(), window_id));
                }
                btn.into()
            }
            UbinWidget::Spacer { size } => {
               container(text("")).width(*size as f32).height(*size as f32).into()
            }
            UbinWidget::TextInput { placeholder, value, on_change, .. } => {
                text_input(placeholder, value)
                    .on_input(move |_| FallbackMessage::UbinAction(on_change.clone(), window_id))
                    .padding(10)
                    .size(20)
                    .into()
            }
            UbinWidget::Checkbox { label, checked, on_toggle } => {
                checkbox(label, *checked)
                    .on_toggle(move |_| FallbackMessage::UbinAction(on_toggle.clone(), window_id))
                    .into()
            }
            UbinWidget::Slider { min, max, value, on_change, step, .. } => {
                slider(*min..=*max, *value, move |_| FallbackMessage::UbinAction(on_change.clone(), window_id))
                    .step(*step)
                    .into()
            }
            UbinWidget::ProgressBar { progress, label, .. } => {
                let bar = progress_bar(0.0..=1.0, *progress);
                if let Some(l) = label {
                    column![text(l).size(18), bar].spacing(8).into()
                } else {
                    bar.into()
                }
            }
            UbinWidget::Layout { direction, spacing, children } => {
                let spacing_f32 = *spacing as f32;

                let mut elements = vec![];
                for child in children {
                    elements.push(self.translate_widget_to_iced(child, window_id));
                }

                match direction {
                    UbinLayoutDirection::Horizontal => {
                        row(elements).spacing(spacing_f32).into()
                    }
                    UbinLayoutDirection::Vertical => {
                        column(elements).spacing(spacing_f32).into()
                    }
                    UbinLayoutDirection::Grid(_, _) => {
                        column![text("Grid layout not supported in fallback yet")]
                            .spacing(spacing_f32)
                            .into()
                    }
                }
            }
             UbinWidget::Spacer { size } => {
                container(text("")).width(*size as f32).height(*size as f32).into()
               }
            _ => {
                container(text("Unsupported")).into()
              }            
             UbinWidget::Divider { vertical, thickness } => {
                let thickness_f32 = *thickness as f32;
                if *vertical {
                    container(text(""))
                        .width(Length::Fixed(thickness_f32))
                        .height(Length::Fill)
                        .style(theme::Container::Custom(Box::new(|_| container::Appearance {
                            background: Some(iced::Color::from_rgb8(100, 100, 100).into()),
                            ..Default::default()
                        })))
                        .into()
                } else {
                    container(text(""))
                        .width(Length::Fill)
                        .height(Length::Fixed(thickness_f32))
                        .style(theme::Container::Custom(Box::new(|_| container::Appearance {
                            background: Some(iced::Color::from_rgb8(100, 100, 100).into()),
                            ..Default::default()
                        })))
                        .into()
                }
            }
            _ => {
                container(text("⚠️ Unsupported widget in fallback mode"))
                    .padding(10)
                    .style(theme::Container::Box)
                    .into()
            }
        }
    }
}


// src/platform/fallback.rs (corrected and complete launch function)

/// Fallback runtime başlatıcı – platform adaptörü yokken kullanılır
pub fn launch_fallback_mode(windows: Vec<(u32, String, UbinWidget, u32)>) {
    // IcedSettings doğru şekilde yapılandırılır
    let settings = IcedSettings {
        flags: windows,
        window: iced::window::Settings {
            size: iced::Size::new(1280.0, 720.0),
            resizable: true,
            decorations: true,           // başlık çubuğu vb. gösterilsin
            exit_on_close_request: true, // kapatma isteğiyle çık
            ..iced::window::Settings::default()
        },
        ..IcedSettings::default()
    };

    // Run çağrısı – hata durumunda bile panic yerine loglama
    if let Err(e) = UbinFallbackApp::run(settings) {
        eprintln!("❌ UBIN fallback runtime başlatılamadı: {:?}", e);
    } else {
        println!("🌑 UBIN fallback runtime başarıyla başlatıldı");
    }
}
/// Runtime window'ı fallback'e uyarla
pub fn adapt_to_fallback(_window: &mut UbinRuntimeWindow) {
    println!("🌑 Platform adaptation failed – switching to UBIN fallback ghost mode");
}

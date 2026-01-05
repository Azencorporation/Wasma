// src/main.rs
// WASMA-UBIN – Ana Giriş Noktası
// Tüm modülleri başlatır, UBIN runtime'ı kurar, test window'lar spawn eder
// Eternal dominion döngüsü burada başlar – WASMA otoritesi konuşur
// Tarih: 6 Ocak 2026 – Alpha 1.0

use wasma_ubin::core::abi::{UbinWidget, UbinAction};
use wasma_ubin::core::runtime::UbinRuntime;
use wasma_ubin::core::convergence::UbinConvergenceEngine;
use wasma_ubin::utils::logging::{info, warn, error, critical, debug};
use wasma_ubin::utils::safety::UbinSafetyGuard;
use wasma_ubin::widget::builder::UbinBuilder;
use wasma_ubin::assignment::ExecutionMode;
use wasma_ubin::resource_manager::ResourceMode;

fn main() {
    // 1. Safety bastion & logger başlat
    UbinSafetyGuard::establish_safety_bastion();
    UbinLogger::init(LogLevel::Info, true);

    info("🌀 WASMA-UBIN vAlpha 1.0 starting – Authority active (6 Ocak 2026)");
    info("🏴‍☠️ Unlimited Jurisdiction engaged – Lejyon hazır");

    // 2. UBIN runtime başlat
    let mut runtime = UbinRuntime::initialize();

    // 3. Test UI'lar – UBIN builder ile
    let terminal_ui = UbinBuilder::window("WASMA Sovereign Terminal 🌀")
        .size(1400, 900)
        .child(
            UbinBuilder::column()
                .spacing(20)
                .push(UbinBuilder::label("WASMA UBIN CONTROL PANEL").size(32))
                .push(
                    UbinBuilder::row()
                        .spacing(30)
                        .push(UbinBuilder::button("Lease Yenile", UbinAction::RenewLease(1)))
                        .push(UbinBuilder::button("Dark Mode", UbinAction::ToggleDarkMode))
                        .push(UbinBuilder::primary_button("Close All", UbinAction::CloseWindow))
                )
                .push(UbinBuilder::progress_bar(0.66).label("Convergence Progress"))
                .push(
                    UbinBuilder::column()
                        .spacing(10)
                        .push(UbinBuilder::label("Active Assignments"))
                        .push(UbinBuilder::text_input("Yeni komut gir..."))
                )
                .build()
        )
        .build();

    let monitor_ui = UbinBuilder::window("UBIN Live Monitor")
        .size(1000, 600)
        .child(
            UbinBuilder::column()
                .spacing(15)
                .push(UbinBuilder::label("🟢 Runtime Status"))
                .push(UbinBuilder::progress_bar(1.0).label("Full Convergence Achieved"))
                .push(UbinBuilder::label("Platform: Native Unified"))
                .push(UbinBuilder::label("Features: All injected"))
                .build()
        )
        .build();

    // 4. Window'ları spawn et
    let _terminal_id = runtime.spawn_window(
        "WASMA Sovereign Terminal".to_string(),
        1400,
        900,
        terminal_ui,
        ExecutionMode::GpuPreferred,
    );

    let _monitor_id = runtime.spawn_window(
        "UBIN Live Monitor".to_string(),
        1000,
        600,
        monitor_ui,
        ExecutionMode::Hybrid,
    );

    info("🖥️ 2 sovereign windows spawned – UBIN UI active");

    // 5. Eternal dominion döngüsü başlat
    runtime.run_eternal_dominion();

    critical("🏁 UBIN runtime terminated – Authority eternal");
}

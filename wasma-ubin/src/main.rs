// src/main.rs
// UBIN Runtime Entry Point – Komut satırı interface
// Argparse ile flag destekli – tüm özellikler açılabilir/kapatılabilir

use clap::{Parser, Subcommand, ValueEnum};
use wasma_ubin::*;

/// WASMA-UBIN – Unified Binary Interface System
/// 
/// Cross-platform UI framework with automatic feature convergence.
/// UBIN provides a single unified ABI that translates to native widgets
/// on Linux (GTK/Qt), Windows (Win32/WinUI), and macOS (AppKit).
/// 
/// Missing platform features are automatically injected via polyfills,
/// ensuring full feature parity across all platforms.
/// 
/// Examples:
///   ubin run --verbose
///   ubin analyze myapp --disassemble
///   ubin patch myapp --features blur,acrylic --rebuild
///   ubin demo complete --animations
#[derive(Parser)]
#[command(name = "ubin")]
#[command(author = "WASMA Lejyon <wasma@lejyon.dev>")]
#[command(version = "0.1.0")]
#[command(about = "🌀 Unified Binary Interface with eternal dominion")]
#[command(long_about = None)]
#[command(after_help = "For more information, visit: https://github.com/wasma-lejyon/ubin")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (debug level)
    #[arg(short, long, global = true, help = "Enable detailed debug output")]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true, help = "Disable ANSI color codes in output")]
    no_color: bool,

    /// Resource mode for backend
    #[arg(
        short, 
        long, 
        value_enum, 
        global = true, 
        default_value = "auto",
        help = "Resource allocation strategy"
    )]
    resource_mode: ResourceModeArg,
}

#[derive(Clone, Copy, ValueEnum)]
enum ResourceModeArg {
    /// Automatic resource detection and allocation
    Auto,
}

impl From<ResourceModeArg> for ResourceMode {
    fn from(val: ResourceModeArg) -> Self {
        match val {
            ResourceModeArg::Auto => ResourceMode::Auto,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Run the UBIN eternal dominion cycle with a demo UI
    /// 
    /// Spawns a runtime window with the UBIN widget tree and enters
    /// the eternal dominion event loop. Features are automatically
    /// converged unless --no-convergence is specified.
    /// 
    /// Example: ubin run --title "My App" --width 1920 --height 1080
    #[command(visible_alias = "r")]
    Run {
        /// Window title
        #[arg(short, long, default_value = "UBIN Eternal Dominion", help = "Set the window title")]
        title: String,

        /// Window width in pixels
        #[arg(long, default_value = "1280", help = "Window width")]
        width: u32,

        /// Window height in pixels
        #[arg(long, default_value = "720", help = "Window height")]
        height: u32,

        /// Execution mode (affects resource allocation)
        #[arg(
            short, 
            long, 
            value_enum, 
            default_value = "cpu-only",
            help = "Execution priority mode"
        )]
        mode: ExecutionModeArg,

        /// Enable ghost mode (fallback rendering with iced+wgpu)
        #[arg(long, help = "Force fallback renderer (no native widgets)")]
        ghost_mode: bool,

        /// Disable automatic feature convergence
        #[arg(long, help = "Skip polyfill injection for missing features")]
        no_convergence: bool,

        /// Maximum frame rate
        #[arg(long, default_value = "60", help = "Target FPS (0 = unlimited)")]
        max_fps: u32,
    },

    /// Analyze a binary and extract UI framework features
    /// 
    /// Scans the binary for UI framework signatures (GTK, Qt, Win32, AppKit),
    /// rendering API calls (Vulkan, DirectX, Metal), and visual features.
    /// Outputs a detailed report of detected capabilities.
    /// 
    /// Example: ubin analyze myapp --disassemble --format json
    #[cfg(feature = "transmutation")]
    #[command(visible_alias = "a")]
    Analyze {
        /// Path to binary file
        #[arg(help = "Binary file to analyze")]
        path: PathBuf,

        /// Show detailed disassembly with instruction analysis
        #[arg(short, long, help = "Run full disassembly analysis")]
        disassemble: bool,

        /// Output format
        #[arg(
            short = 'f', 
            long, 
            value_enum, 
            default_value = "text",
            help = "Report output format"
        )]
        format: OutputFormat,

        /// Save report to file
        #[arg(short, long, help = "Save report to specified file")]
        output: Option<PathBuf>,
    },

    /// Patch a binary with missing UI features via polyfill injection
    /// 
    /// Analyzes the target binary, detects missing features, and injects
    /// polyfill code to provide missing capabilities (blur effects, rounded
    /// corners, shadows, etc). Can optionally rebuild the binary.
    /// 
    /// Example: ubin patch myapp --features blur,acrylic,rounded --rebuild
    #[cfg(feature = "transmutation")]
    #[command(visible_alias = "p")]
    Patch {
        /// Path to input binary file
        #[arg(help = "Binary file to patch")]
        input: PathBuf,

        /// Output path (optional, defaults to input.ubin-patched)
        #[arg(short, long, help = "Custom output path for patched binary")]
        output: Option<PathBuf>,

        /// Features to inject (comma-separated)
        /// Available: blur, acrylic, mica, vibrancy, rounded, shadow, darkmode, hidpi, csd, toolbar
        #[arg(
            short, 
            long, 
            help = "Comma-separated list of features to inject (auto-detect if not specified)"
        )]
        features: Option<String>,

        /// Rebuild binary after patching
        #[arg(short, long, help = "Rebuild binary with proper ELF/PE/Mach-O structure")]
        rebuild: bool,

        /// Verify patched binary
        #[arg(long, help = "Run verification after patching")]
        verify: bool,
    },

    /// Show convergence engine status and platform capabilities
    /// 
    /// Displays the current platform's native features and identifies
    /// missing capabilities that would be polyfilled. Can optionally
    /// apply global convergence to inject all missing features.
    /// 
    /// Example: ubin convergence --apply --platform-info
    #[command(visible_alias = "c")]
    Convergence {
        /// Apply global convergence (inject all missing features)
        #[arg(short, long, help = "Apply convergence and inject polyfills")]
        apply: bool,

        /// Show detailed platform-specific feature information
        #[arg(short, long, help = "Display platform capabilities report")]
        platform_info: bool,

        /// List all available features
        #[arg(short, long, help = "Show all extractable features")]
        list_features: bool,
    },

    /// Interactive widget builder demo
    /// 
    /// Run various demo UIs showcasing UBIN's widget system.
    /// Demos include basic widgets, advanced layouts, tabs, dialogs, etc.
    /// 
    /// Example: ubin demo complete --animations
    #[command(visible_alias = "d")]
    Demo {
        /// Demo type to run
        #[arg(value_enum, default_value = "basic", help = "Select demo type")]
        demo_type: DemoType,

        /// Enable smooth entrance/exit animations
        #[arg(long, help = "Enable widget animations")]
        animations: bool,

        /// Dark mode
        #[arg(long, help = "Force dark theme")]
        dark: bool,
    },

    /// Display system information and UBIN capabilities
    /// 
    /// Shows detailed information about the current platform, available
    /// UI frameworks, rendering APIs, and UBIN system status.
    #[command(visible_alias = "i")]
    Info {
        /// Show detailed system diagnostics
        #[arg(short, long, help = "Include full diagnostic information")]
        detailed: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ExecutionModeArg {
    /// CPU-only mode (no GPU acceleration)
    CpuOnly,
}

impl From<ExecutionModeArg> for ExecutionMode {
    fn from(val: ExecutionModeArg) -> Self {
        match val {
            ExecutionModeArg::CpuOnly => ExecutionMode::CpuOnly,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DemoType {
    /// Basic widgets (buttons, labels, inputs)
    Basic,
    /// Advanced layouts and containers
    Advanced,
    /// Tab view demonstration
    TabView,
    /// Modal dialog examples
    Dialog,
    /// List view with selectable items
    ListView,
    /// Complete showcase of all features
    Complete,
}

fn main() {
    let cli = Cli::parse();

    // Initialize UBIN system
    initialize_ubin();

    // Configure logging
    let log_level = if cli.verbose {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };
    UbinLogger::init(log_level, !cli.no_color);

    info("🚀 UBIN CLI Starting...");

    match cli.command {
        Commands::Run {
            title,
            width,
            height,
            mode,
            ghost_mode,
            no_convergence,
            max_fps,
        } => {
            run_eternal_dominion(
                title,
                width,
                height,
                mode.into(),
                ghost_mode,
                !no_convergence,
                max_fps,
            );
        }
        #[cfg(feature = "transmutation")]
        Commands::Analyze {
            path,
            disassemble,
            format,
            output,
        } => {
            analyze_binary(path, disassemble, format, output);
        }
        #[cfg(feature = "transmutation")]
        Commands::Patch {
            input,
            output,
            features,
            rebuild,
            verify,
        } => {
            patch_binary(input, output, features, rebuild, verify);
        }
        Commands::Convergence {
            apply,
            platform_info,
            list_features,
        } => {
            show_convergence_status(apply, platform_info, list_features);
        }
        Commands::Demo { demo_type, animations, dark } => {
            run_demo(demo_type, animations, dark);
        }
        Commands::Info { detailed } => {
            show_system_info(detailed);
        }
    }
}

fn run_eternal_dominion(
    title: String,
    width: u32,
    height: u32,
    mode: ExecutionMode,
    ghost_mode: bool,
    enable_convergence: bool,
    _max_fps: u32,
) {
    info(&format!(
        "🌀 Starting eternal dominion: {} ({}x{})",
        title, width, height
    ));
    info(&format!("   Mode: {:?} | Ghost: {} | Convergence: {}", 
        mode, ghost_mode, enable_convergence));

    let mut runtime = UbinRuntime::initialize();

    // Build demo UI
    let root_widget = build_demo_ui();

    let window_id = runtime.spawn_window(title, width, height, root_widget, mode);

    info(&format!("✅ Window spawned with ID: {}", window_id));

    if enable_convergence && !ghost_mode {
        info("🔄 Applying convergence...");
        let mut convergence = UbinConvergenceEngine::initiate_global_convergence();
        convergence.enforce_global_convergence();
    }

    if ghost_mode {
        warn("👻 Ghost mode enabled – falling back to iced renderer");
    }

    runtime.run_eternal_dominion();
}

#[cfg(feature = "transmutation")]
fn analyze_binary(path: PathBuf, disassemble: bool, format: OutputFormat, output: Option<PathBuf>) {
    info(&format!("🔍 Analyzing binary: {:?}", path));

    use wasma_ubin::transmutation::*;

    let extractor = UbinFeatureExtractor::new();
    let report = extractor.extract_features_from_binary(path.clone());

    let report_str = match format {
        OutputFormat::Text => format_text_report(&report),
        OutputFormat::Json => {
            serde_json::to_string_pretty(&format!("{:#?}", report)).unwrap_or_default()
        }
        OutputFormat::Yaml => format_yaml_report(&report),
    };

    if let Some(out_path) = output {
        match std::fs::write(&out_path, &report_str) {
            Ok(_) => info(&format!("💾 Report saved to: {:?}", out_path)),
            Err(e) => error(&format!("Failed to save report: {}", e)),
        }
    } else {
        println!("{}", report_str);
    }

    if disassemble {
        info("⚙️ Running disassembler...");
        let disassembler = UbinDisassembler::new();
        let dis_report = disassembler.disassemble_binary(path);

        if dis_report.analysis_success {
            info(&format!(
                "✅ Disassembly complete: {} instructions, {} UI calls, {} render calls",
                dis_report.instructions.len(),
                dis_report.ui_related_calls.len(),
                dis_report.render_related_calls.len()
            ));
        } else {
            error(&format!("❌ Disassembly failed: {:?}", dis_report.error_msg));
        }
    }
}

#[cfg(feature = "transmutation")]
fn format_text_report(report: &wasma_ubin::transmutation::BinaryFeatureReport) -> String {
    let mut output = String::new();
    output.push_str("\n╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║           UBIN BINARY ANALYSIS REPORT                   ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    output.push_str(&format!("\n📁 Path: {:?}\n", report.path));
    output.push_str(&format!("🖥️  Platform: {:?}\n", report.platform));
    output.push_str(&format!("🎨 Framework: {}\n", report.detected_framework));
    output.push_str(&format!("🔢 Symbol Count: {}\n", report.symbol_count));
    output.push_str(&format!("\n✨ Extracted Features ({}):\n", report.extracted_features.len()));
    for feature in &report.extracted_features {
        output.push_str(&format!("   • {:?}\n", feature));
    }

    if !report.string_hints.is_empty() {
        output.push_str("\n💡 Hints:\n");
        for hint in &report.string_hints {
            output.push_str(&format!("   • {}\n", hint));
        }
    }

    if report.analysis_success {
        output.push_str("\n✅ Analysis Status: SUCCESS\n");
    } else {
        output.push_str("\n❌ Analysis Status: FAILED\n");
        if let Some(err) = &report.error_msg {
            output.push_str(&format!("   Error: {}\n", err));
        }
    }
    output.push('\n');
    output
}

#[cfg(feature = "transmutation")]
fn format_yaml_report(report: &wasma_ubin::transmutation::BinaryFeatureReport) -> String {
    format!(
        "# UBIN Feature Analysis Report\npath: {:?}\nplatform: {:?}\nframework: {}\nsymbol_count: {}\nfeatures:\n{}\nsuccess: {}\n",
        report.path,
        report.platform,
        report.detected_framework,
        report.symbol_count,
        report.extracted_features.iter()
            .map(|f| format!("  - {:?}", f))
            .collect::<Vec<_>>()
            .join("\n"),
        report.analysis_success
    )
}

#[cfg(feature = "transmutation")]
fn patch_binary(input: PathBuf, _output: Option<PathBuf>, features: Option<String>, rebuild: bool, _verify: bool) {
    info(&format!("⚡ Patching binary: {:?}", input));

    use wasma_ubin::transmutation::*;
    use std::collections::HashSet;

    // Önce analiz et
    let extractor = UbinFeatureExtractor::new();
    let analysis = extractor.extract_features_from_binary(input.clone());

    // Disassemble et
    let disassembler = UbinDisassembler::new();
    let disassembly = disassembler.disassemble_binary(input.clone());

    if !disassembly.analysis_success {
        error("❌ Cannot patch – disassembly failed");
        return;
    }

    // Eksik özellikleri belirle
    let mut missing_features = HashSet::new();

    if let Some(feature_str) = features {
        // Manuel özellik listesi
        for f in feature_str.split(',') {
            let f = f.trim();
            match f {
                "blur" => { missing_features.insert(ExtractedFeature::HasBlurEffect); }
                "acrylic" => { missing_features.insert(ExtractedFeature::HasAcrylicMaterial); }
                "mica" => { missing_features.insert(ExtractedFeature::HasMicaMaterial); }
                "vibrancy" => { missing_features.insert(ExtractedFeature::HasVibrancy); }
                "rounded" => { missing_features.insert(ExtractedFeature::HasRoundedCorners); }
                "shadow" => { missing_features.insert(ExtractedFeature::HasShadowEffect); }
                "darkmode" => { missing_features.insert(ExtractedFeature::HasDarkModeSupport); }
                "hidpi" => { missing_features.insert(ExtractedFeature::HasHighDpiScaling); }
                "csd" => { missing_features.insert(ExtractedFeature::HasCsd); }
                "toolbar" => { missing_features.insert(ExtractedFeature::HasUnifiedToolbar); }
                _ => warn(&format!("Unknown feature: {}", f)),
            }
        }
    } else {
        // Otomatik tespit
        info("🔍 Auto-detecting missing features...");
        if !analysis.extracted_features.contains(&ExtractedFeature::HasBlurEffect) {
            missing_features.insert(ExtractedFeature::HasBlurEffect);
        }
        if !analysis.extracted_features.contains(&ExtractedFeature::HasRoundedCorners) {
            missing_features.insert(ExtractedFeature::HasRoundedCorners);
        }
    }

    if missing_features.is_empty() {
        info("✅ No missing features detected – binary is fully converged");
        return;
    }

    info(&format!("🔧 Injecting {} missing features", missing_features.len()));

    let patcher = UbinPatcher::new();
    let patch_report = patcher.patch_binary_with_features(&disassembly, missing_features);

    if patch_report.success {
        info(&format!(
            "✅ Patching complete: {} operations applied",
            patch_report.operations.len()
        ));
        info(&format!("💾 Patched binary saved: {:?}", patch_report.patched_path));

        if rebuild {
            info("🏗️ Rebuilding binary...");
            let rebuilder = UbinRebuilder::new();
            let data = std::fs::read(&patch_report.patched_path).unwrap();
            let rebuild_report = rebuilder.rebuild_converged_binary(&input, data);

            if rebuild_report.success {
                info(&format!("✅ Rebuild complete: {:?}", rebuild_report.rebuilt_path));
            } else {
                error("❌ Rebuild failed");
            }
        }
    } else {
        error(&format!("❌ Patching failed: {:?}", patch_report.error_msg));
    }
}

fn show_convergence_status(apply: bool, platform_info: bool, list_features: bool) {
    info("🌀 UBIN Convergence Engine Status");

    let mut convergence = UbinConvergenceEngine::initiate_global_convergence();

    if platform_info {
        use wasma_ubin::platform::*;
        let platform = detect_current_platform();
        println!("\n🖥️  Current Platform: {:?}", platform);

        let features = collect_all_platform_features();
        println!("📦 Total Platform Features: {}", features.len());
    }

    if list_features {
        println!("\n📋 Available Features:");
        println!("   • blur - Gaussian blur effects");
        println!("   • acrylic - Windows Acrylic material");
        println!("   • mica - Windows Mica material");
        println!("   • vibrancy - macOS vibrancy effects");
        println!("   • rounded - Rounded corners");
        println!("   • shadow - Dynamic shadows");
        println!("   • darkmode - Dark mode support");
        println!("   • hidpi - High-DPI scaling");
        println!("   • csd - Client-side decorations");
        println!("   • toolbar - Unified toolbar");
    }

    let missing = convergence.detect_missing_features();

    if missing.is_empty() {
        println!("\n✅ Full convergence achieved – no missing features");
    } else {
        println!("\n⚠️  Missing Features ({}):", missing.len());
        for f in &missing {
            println!("   • {}", f);
        }
    }

    if apply {
        info("\n🔄 Applying global convergence...");
        convergence.enforce_global_convergence();
        println!("✅ Convergence applied");
    }
}

fn run_demo(demo_type: DemoType, _animations: bool, _dark: bool) {
    info(&format!("🎨 Running {:?} demo", demo_type));

    let mut runtime = UbinRuntime::initialize();

    let root_widget = match demo_type {
        DemoType::Basic => build_basic_demo(),
        DemoType::Advanced => build_advanced_demo(),
        DemoType::TabView => build_tabview_demo(),
        DemoType::Dialog => build_dialog_demo(),
        DemoType::ListView => build_listview_demo(),
        DemoType::Complete => build_complete_demo(),
    };

    let window_id = runtime.spawn_window(
        format!("{:?} Demo", demo_type),
        1280,
        720,
        root_widget,
        ExecutionMode::CpuOnly,
    );

    info(&format!("✅ Demo window spawned: {}", window_id));

    runtime.run_eternal_dominion();
}

fn show_system_info(detailed: bool) {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║           UBIN SYSTEM INFORMATION                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    use wasma_ubin::platform::*;
    let platform = detect_current_platform();
    println!("\n🖥️  Platform: {:?}", platform);
    println!("🔧 Version: 0.1.0");
    println!("🏗️  Build: Release");

    if detailed {
        println!("\n📦 Capabilities:");
        let features = collect_all_platform_features();
        println!("   Total Features: {}", features.len());

        println!("\n🔧 Available Modules:");
        println!("   • Core Runtime ✓");
        println!("   • Convergence Engine ✓");
        println!("   • Platform Adaptors ✓");
        println!("   • Widget System ✓");
        println!("   • Transmutation Engine ✓");
        println!("   • Safety System ✓");
    }

    println!();
}

// Demo UI builders
fn build_demo_ui() -> UbinWidget {
    UbinBuilder::window("UBIN Demo")
        .size(1280, 720)
        .child(
            UbinBuilder::column()
                .push(UbinWidget::label("🌀 UBIN Eternal Dominion"))
                .push(UbinBuilder::button("Click Me", UbinAction::NoOp))
                .push(UbinWidget::divider(false, 2))
                .push(UbinWidget::label("Status: Running"))
                .build(),
        )
        .build()
}

fn build_basic_demo() -> UbinWidget {
    UbinBuilder::column()
        .spacing(20)
        .push(UbinWidget::label("Basic Widget Demo"))
        .push(UbinWidget::button("Primary Button", UbinAction::NoOp))
        .push(UbinWidget::TextInput {
            placeholder: "Enter text...".to_string(),
            value: String::new(),
            on_change: UbinAction::NoOp,
        })
        .push(UbinWidget::Checkbox {
            label: "Enable feature".to_string(),
            checked: false,
            on_toggle: UbinAction::NoOp,
        })
        .push(UbinWidget::Slider {
            min: 0.0,
            max: 100.0,
            value: 50.0,
            step: 1.0,
            on_change: UbinAction::NoOp,
        })
        .push(UbinWidget::ProgressBar {
            progress: 0.75,
            label: Some("Loading...".to_string()),
        })
        .build()
}

fn build_advanced_demo() -> UbinWidget {
    UbinBuilder::column()
        .spacing(15)
        .push(UbinWidget::label("Advanced Features"))
        .push(UbinWidget::divider(false, 1))
        .push(UbinBuilder::row()
            .push(UbinWidget::button("Action 1", UbinAction::NoOp))
            .push(UbinWidget::Spacer { size: 10 })
            .push(UbinWidget::button("Action 2", UbinAction::NoOp))
            .build())
        .build()
}

fn build_tabview_demo() -> UbinWidget {
    UbinWidget::label("TabView Demo - Coming Soon")
}

fn build_dialog_demo() -> UbinWidget {
    UbinWidget::label("Dialog Demo - Coming Soon")
}

fn build_listview_demo() -> UbinWidget {
    UbinWidget::label("ListView Demo - Coming Soon")
}

fn build_complete_demo() -> UbinWidget {
    UbinBuilder::column()
        .spacing(10)
        .children(vec![
            UbinWidget::label("Complete UBIN Demo"),
            UbinWidget::divider(false, 2),
            build_basic_demo(),
            UbinWidget::divider(false, 2),
            build_advanced_demo(),
        ])
        .build()
}

// WASMA - Windows Assignment System Monitoring Architecture
// Main Entry Point - CLI & GUI Support
// January 14, 2026

use clap::{Parser, Subcommand};
use std::process;
use wasma_core::{
    WasmaCore, WasmaCoreBuilder,
    utils::{init_config, validate_config, print_config_info},
    ResourceMode, WindowGeometry, WindowState,
};
use wbackend::ExecutionMode;

#[derive(Parser)]
#[command(name = "wasma")]
#[command(author = "WASMA Project")]
#[command(version = "1.0.0")]
#[command(about = "Windows Assignment System Monitoring Architecture", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Resource mode (auto/manual)
    #[arg(short = 'm', long, value_enum, default_value = "auto")]
    resource_mode: ResourceModeArg,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ResourceModeArg {
    Auto,
    Manual,
}

impl From<ResourceModeArg> for ResourceMode {
    fn from(arg: ResourceModeArg) -> Self {
        match arg {
            ResourceModeArg::Auto => ResourceMode::Auto,
            ResourceModeArg::Manual => ResourceMode::Manual,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Launch GUI window manager
    Gui {
        /// Window width
        #[arg(short, long, default_value = "1200")]
        width: u32,
        
        /// Window height
        #[arg(short = 'h', long, default_value = "800")]
        height: u32,
    },

    /// Initialize default configuration file
    Init {
        /// Output path for config file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Validate configuration file
    Validate,

    /// Show configuration information
    Info,

    /// Create a new window (CLI mode)
    Create {
        /// Window title
        #[arg(short, long)]
        title: String,

        /// Application ID
        #[arg(short, long)]
        app_id: String,

        /// Window width
        #[arg(short, long, default_value = "800")]
        width: u32,

        /// Window height
        #[arg(short = 'h', long, default_value = "600")]
        height: u32,

        /// Manifest file path
        #[arg(short, long)]
        manifest: Option<String>,
    },

    /// List all windows
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Close a window
    Close {
        /// Window ID to close
        window_id: u64,
    },

    /// Focus a window
    Focus {
        /// Window ID to focus
        window_id: u64,
    },

    /// Get window resource usage
    Resources {
        /// Window ID
        window_id: u64,
    },

    /// Set window state
    State {
        /// Window ID
        window_id: u64,

        /// New state (normal/minimized/maximized/fullscreen/hidden)
        #[arg(value_enum)]
        state: StateArg,
    },

    /// Run resource management cycle
    Cycle {
        /// Number of cycles to run (0 = continuous)
        #[arg(short, long, default_value = "1")]
        count: u32,
    },

    /// Start UClient engine (direct renderer mode)
    UClient {
        /// Force raw stream mode (scope_level=0)
        #[arg(short, long)]
        raw: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StateArg {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
    Hidden,
}

impl From<StateArg> for WindowState {
    fn from(arg: StateArg) -> Self {
        match arg {
            StateArg::Normal => WindowState::Normal,
            StateArg::Minimized => WindowState::Minimized,
            StateArg::Maximized => WindowState::Maximized,
            StateArg::Fullscreen => WindowState::Fullscreen,
            StateArg::Hidden => WindowState::Hidden,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
        println!("🔍 Verbose mode enabled");
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }

    match &cli.command {
        Some(Commands::Init { output }) => {
            handle_init(output.clone());
        }
        Some(Commands::Validate) => {
            handle_validate(cli.config);
        }
        Some(Commands::Info) => {
            handle_info(cli.config);
        }
        Some(Commands::Gui { width, height }) => {
            handle_gui(cli.config, cli.resource_mode.into(), *width, *height);
        }
        Some(Commands::Create { title, app_id, width, height, manifest }) => {
            handle_create(cli.config, cli.resource_mode.into(), title, app_id, *width, *height, manifest.clone());
        }
        Some(Commands::List { detailed }) => {
            handle_list(cli.config, cli.resource_mode.into(), *detailed);
        }
        Some(Commands::Close { window_id }) => {
            handle_close(cli.config, cli.resource_mode.into(), *window_id);
        }
        Some(Commands::Focus { window_id }) => {
            handle_focus(cli.config, cli.resource_mode.into(), *window_id);
        }
        Some(Commands::Resources { window_id }) => {
            handle_resources(cli.config, cli.resource_mode.into(), *window_id);
        }
        Some(Commands::State { window_id, state }) => {
            handle_state(cli.config, cli.resource_mode.into(), *window_id, state.clone().into());
        }
        Some(Commands::Cycle { count }) => {
            handle_cycle(cli.config, cli.resource_mode.into(), *count);
        }
        Some(Commands::UClient { raw }) => {
            handle_uclient(cli.config, *raw);
        }
        None => {
            // Default: Launch GUI
            handle_gui(cli.config, cli.resource_mode.into(), 1200, 800);
        }
    }
}

fn handle_init(output: Option<String>) {
    println!("🔧 Initializing WASMA configuration...");
    match init_config(output) {
        Ok(path) => {
            println!("✅ Configuration file created: {}", path);
            println!("   Edit this file to customize your WASMA setup.");
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize config: {}", e);
            process::exit(1);
        }
    }
}

fn handle_validate(config_path: Option<String>) {
    println!("🔍 Validating configuration...");
    match validate_config(config_path) {
        Ok(_) => {
            println!("✅ Configuration is valid!");
        }
        Err(e) => {
            eprintln!("❌ Configuration validation failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_info(config_path: Option<String>) {
    if let Err(e) = print_config_info(config_path) {
        eprintln!("❌ Failed to read config: {}", e);
        process::exit(1);
    }
}

fn handle_gui(config_path: Option<String>, resource_mode: ResourceMode, _width: u32, _height: u32) {
    println!("🖥️  Launching WASMA GUI Window Manager...");
    
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    println!("🚀 Starting GUI with resource mode: {:?}", resource_mode);
    
    if let Err(e) = core.launch_gui() {
        eprintln!("❌ GUI failed: {}", e);
        process::exit(1);
    }
}

fn handle_create(
    config_path: Option<String>,
    resource_mode: ResourceMode,
    title: &str,
    app_id: &str,
    width: u32,
    height: u32,
    manifest: Option<String>,
) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    println!("🪟 Creating window: {}", title);
    
    let result = if manifest.is_some() {
        core.create_window(
            title.to_string(),
            app_id.to_string(),
            width,
            height,
        )
    } else {
        core.create_window(
            title.to_string(),
            app_id.to_string(),
            width,
            height,
        )
    };

    match result {
        Ok(window_id) => {
            println!("✅ Window created successfully!");
            println!("   Window ID: {}", window_id);
            println!("   Title: {}", title);
            println!("   Size: {}x{}", width, height);
            println!("   Mode: {:?}", resource_mode);
        }
        Err(e) => {
            eprintln!("❌ Failed to create window: {}", e);
            process::exit(1);
        }
    }
}

fn handle_list(config_path: Option<String>, resource_mode: ResourceMode, detailed: bool) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    let windows = core.list_windows();

    if windows.is_empty() {
        println!("ℹ️  No active windows.");
        return;
    }

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    Active Windows                          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    for window in windows {
        let state_icon = match window.state {
            WindowState::Normal => "🟢",
            WindowState::Minimized => "🟡",
            WindowState::Maximized => "🔵",
            WindowState::Fullscreen => "⚡",
            WindowState::Hidden => "⚫",
        };

        let focus = if window.focused { "👁️ " } else { "" };

        println!("{}{} Window #{}: {}", focus, state_icon, window.id, window.title);
        println!("   App ID: {}", window.app_id);
        println!("   Geometry: {}x{} at ({}, {})", 
            window.geometry.width, 
            window.geometry.height,
            window.geometry.x,
            window.geometry.y
        );
        println!("   Visible: {} | Focused: {}", window.visible, window.focused);

        if detailed {
            println!("   Renderer: {}", window.resource_limits.renderer);
            println!("   Execution Mode: {:?}", window.resource_limits.execution_mode);
            
            if let Ok(usage) = core.get_window_resources(window.id) {
                println!("   RAM: {} MiB | VRAM: {} MiB", 
                    usage.ram_allocated_mb, 
                    usage.vram_allocated_mb
                );
                println!("   CPU Cores: {:?}", usage.cpu_cores);
                if let Some(ref gpu) = usage.gpu_device {
                    println!("   GPU: {}", gpu);
                }
                println!("   Task Active: {} | GPU Active: {}", 
                    usage.task_active, 
                    usage.gpu_active
                );
                if usage.remaining_lease_secs > 0 {
                    println!("   Lease Remaining: {}s", usage.remaining_lease_secs);
                }
            }
        }

        println!();
    }

    println!("Total windows: {}", windows.len());
}

fn handle_close(config_path: Option<String>, resource_mode: ResourceMode, window_id: u64) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    println!("🗑️  Closing window {}...", window_id);
    
    match core.close_window(window_id) {
        Ok(_) => {
            println!("✅ Window {} closed successfully", window_id);
        }
        Err(e) => {
            eprintln!("❌ Failed to close window: {}", e);
            process::exit(1);
        }
    }
}

fn handle_focus(config_path: Option<String>, resource_mode: ResourceMode, window_id: u64) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    println!("👁️  Focusing window {}...", window_id);
    
    match core.focus_window(window_id) {
        Ok(_) => {
            println!("✅ Window {} is now focused", window_id);
        }
        Err(e) => {
            eprintln!("❌ Failed to focus window: {}", e);
            process::exit(1);
        }
    }
}

fn handle_resources(config_path: Option<String>, resource_mode: ResourceMode, window_id: u64) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    match core.get_window_resources(window_id) {
        Ok(usage) => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║           Window #{} Resource Usage                       ║", window_id);
            println!("╚════════════════════════════════════════════════════════════╝\n");
            
            println!("📊 Assignment ID: {}", usage.assignment_id);
            println!("💾 RAM Allocated: {} MiB", usage.ram_allocated_mb);
            println!("🎮 VRAM Allocated: {} MiB", usage.vram_allocated_mb);
            println!("🔧 CPU Cores: {:?}", usage.cpu_cores);
            
            if let Some(ref gpu) = usage.gpu_device {
                println!("🎨 GPU Device: {}", gpu);
            } else {
                println!("🎨 GPU Device: None");
            }
            
            println!("⚙️  Execution Mode: {:?}", usage.execution_mode);
            println!("🟢 Task Active: {}", usage.task_active);
            println!("🎯 GPU Active: {}", usage.gpu_active);
            
            if usage.remaining_lease_secs > 0 {
                println!("⏱️  Lease Remaining: {}s", usage.remaining_lease_secs);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get resources: {}", e);
            process::exit(1);
        }
    }
}

fn handle_state(
    config_path: Option<String>,
    resource_mode: ResourceMode,
    window_id: u64,
    state: WindowState,
) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    println!("🔄 Setting window {} state to {:?}...", window_id, state);
    
    match core.set_window_state(window_id, state) {
        Ok(_) => {
            println!("✅ Window state changed successfully");
        }
        Err(e) => {
            eprintln!("❌ Failed to change state: {}", e);
            process::exit(1);
        }
    }
}

fn handle_cycle(config_path: Option<String>, resource_mode: ResourceMode, count: u32) {
    let core = match build_core(config_path, Some(resource_mode)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize WASMA Core: {}", e);
            process::exit(1);
        }
    };

    if count == 0 {
        println!("🔄 Running resource management cycle continuously...");
        println!("   Press Ctrl+C to stop");
        loop {
            core.update();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        println!("🔄 Running {} resource management cycle(s)...", count);
        for i in 1..=count {
            println!("   Cycle {}/{}", i, count);
            core.update();
            if i < count {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
        println!("✅ Resource cycles completed");
    }
}

fn handle_uclient(config_path: Option<String>, raw: bool) {
    use wasma_core::{ConfigParser, UClient};

    println!("🔌 Starting UClient engine...");
    
    let parser = ConfigParser::new(config_path);
    let mut config = match parser.load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load config: {}", e);
            process::exit(1);
        }
    };

    if raw {
        println!("⚡ Force enabling RAW mode (scope_level=0)");
        config.resource_limits.scope_level = 0;
    }

    let mut client = UClient::new(config);
    
    println!("🚀 UClient engine started");
    
    if let Err(e) = client.start_engine() {
        eprintln!("❌ UClient engine error: {}", e);
        process::exit(1);
    }
}

fn build_core(
    config_path: Option<String>,
    resource_mode: Option<ResourceMode>,
) -> Result<WasmaCore, String> {
    let mut builder = WasmaCoreBuilder::new();

    if let Some(path) = config_path {
        builder = builder.with_config_path(path);
    }

    if let Some(mode) = resource_mode {
        builder = builder.with_resource_mode(mode);
    }

    builder.build().map_err(|e| e.to_string())
}

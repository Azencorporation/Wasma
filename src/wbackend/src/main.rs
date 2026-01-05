// src/main.rs
use wbackend::{Assignment, ExecutionMode, ResourceMode, WBackend};
use clap::Parser;
use std::thread;
use std::time::Duration;

/// WASMA – Resource-first Runtime Authority (January 02, 2026)
#[derive(Parser, Debug)]
#[command(name = "wasma")]
#[command(version = "1.0.0")]
#[command(about = "WASMA: CPU-priority, GPU-optional, lease-based assignment runtime")]
#[command(author = "Zaman Huseyinli <zamanhuseynli23@gmail.com>")]
struct Cli {
    /// Resource allocation mode
    #[arg(long, default_value = "auto")]
    mode: ResourceMode,

    /// Number of cycles to run (0 = infinite)
    #[arg(short, long, default_value_t = 10)]
    cycles: usize,

    /// Add assignment
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Add a new assignment
    Add {
        /// Assignment ID
        id: u32,

        /// Execution mode
        #[arg(value_enum)]
        exec: ExecutionMode,
    },
}

fn main() {
    let cli = Cli::parse();

    println!("🌀 WASMA v1.0 starting – Authority active (January 02, 2026)");
    println!(
        "Mode: {:?} | Cycles: {}",
        cli.mode,
        if cli.cycles == 0 {
            "infinite".to_string()
        } else {
            cli.cycles.to_string()
        }
    );

    let backend = WBackend::new(cli.mode);

    // Add assignments from CLI
    match cli.command {
        Some(Commands::Add { id, exec }) => {
            let mut assignment = Assignment::new(id);
            assignment.execution_mode = exec;
            backend.add_assignment(assignment);
            println!("➕ Assignment {} added | Mode: {:?}", id, exec);
        }
        None => {
            // Default: Add 3 test assignments
            println!("📋 Adding default 3 assignments (test mode)...");

            let mut a1 = Assignment::new(1);
            a1.execution_mode = ExecutionMode::CpuOnly;
            backend.add_assignment(a1);

            let mut a2 = Assignment::new(2);
            a2.execution_mode = ExecutionMode::GpuPreferred;
            backend.add_assignment(a2);

            let mut a3 = Assignment::new(3);
            a3.execution_mode = ExecutionMode::GpuOnly;
            backend.add_assignment(a3);
        }
    }

    // Cycle loop
    let cycle_count = if cli.cycles == 0 { usize::MAX } else { cli.cycles };

    for i in 1..=cycle_count {
        backend.run_cycle();

        thread::sleep(Duration::from_secs(2));

        if i % 5 == 0 || i == 1 {
            println!("🔄 Cycle {} completed\n", i);
        }

        if cli.cycles == 0 && i % 50 == 0 {
            println!("♾️  Infinite mode active – {} cycles passed", i);
        }
    }

    println!("🏁 WASMA completed – Authority shutting down.");
}

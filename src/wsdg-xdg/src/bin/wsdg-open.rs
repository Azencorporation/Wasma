// WSDG-Open CLI Tool
// Command-line interface for opening files and applications
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::env;
use std::process;
use wsdg_xdg::{WsdgEnv, WsdgOpen, WsdgGhxOpen, XdgWsdgTranslator};

fn print_usage() {
    eprintln!("Usage: wsdg-open [OPTIONS] <FILE|URL|APP>");
    eprintln!();
    eprintln!("Open files, URLs, or applications using WSDG environment");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help              Show this help message");
    eprintln!("  -v, --version           Show version information");
    eprintln!("  -a, --app <NAME>        Open application by name");
    eprintln!("  -u, --uri               Treat argument as URI");
    eprintln!("  --list-apps             List available applications");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  wsdg-open document.pdf              # Open PDF file");
    eprintln!("  wsdg-open -a firefox                # Open Firefox");
    eprintln!("  wsdg-open -u https://example.com    # Open URL");
    eprintln!("  wsdg-open app://myapp               # Open via app:// URI");
}

fn print_version() {
    println!("wsdg-open {}", wsdg_xdg::VERSION);
    println!("{}", wsdg_xdg::LIBRARY_INFO);
}

fn list_applications(opener: &WsdgOpen) {
    let apps = opener.list_applications();
    
    if apps.is_empty() {
        println!("No applications found");
        return;
    }
    
    println!("Available applications:");
    println!();
    
    for app in apps {
        println!("  {} - {}", app.name, app.exec);
        if let Some(ref icon) = app.icon {
            println!("    Icon: {}", icon);
        }
        if !app.categories.is_empty() {
            println!("    Categories: {}", app.categories.join(", "));
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // Parse arguments
    let mut mode = "file";
    let mut target = String::new();
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "-v" | "--version" => {
                print_version();
                return;
            }
            "-a" | "--app" => {
                mode = "app";
                if i + 1 < args.len() {
                    target = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: --app requires an argument");
                    process::exit(1);
                }
            }
            "-u" | "--uri" => {
                mode = "uri";
                if i + 1 < args.len() {
                    target = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: --uri requires an argument");
                    process::exit(1);
                }
            }
            "--list-apps" => {
                mode = "list";
            }
            arg if !arg.starts_with('-') => {
                target = arg.to_string();
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
        i += 1;
    }
    
    // Initialize WSDG environment
    let env = WsdgEnv::new();
    
    // Try to load translator (optional)
    let translator = XdgWsdgTranslator::from_default().ok();
    
    let mut opener = if let Some(trans) = translator {
        WsdgOpen::new(env.clone()).with_translator(trans)
    } else {
        WsdgOpen::new(env.clone())
    };
    
    // Execute command
    match mode {
        "list" => {
            list_applications(&opener);
        }
        "app" => {
            if target.is_empty() {
                eprintln!("Error: No application specified");
                process::exit(1);
            }
            
            match opener.open_app(&target, &[]) {
                Ok(_) => {
                    println!("Launched: {}", target);
                }
                Err(e) => {
                    eprintln!("Error launching {}: {}", target, e);
                    process::exit(1);
                }
            }
        }
        "uri" => {
            if target.is_empty() {
                eprintln!("Error: No URI specified");
                process::exit(1);
            }
            
            let mut ghx_opener = WsdgGhxOpen::new(opener);
            
            match ghx_opener.open_uri(&target) {
                Ok(_) => {
                    println!("Opened URI: {}", target);
                }
                Err(e) => {
                    eprintln!("Error opening URI {}: {}", target, e);
                    process::exit(1);
                }
            }
        }
        "file" | _ => {
            if target.is_empty() {
                eprintln!("Error: No file specified");
                process::exit(1);
            }
            
            // Auto-detect if it's a URI
            if target.contains("://") {
                let mut ghx_opener = WsdgGhxOpen::new(opener);
                match ghx_opener.open_uri(&target) {
                    Ok(_) => {
                        println!("Opened: {}", target);
                    }
                    Err(e) => {
                        eprintln!("Error opening {}: {}", target, e);
                        process::exit(1);
                    }
                }
            } else {
                match opener.open(&target) {
                    Ok(_) => {
                        println!("Opened: {}", target);
                    }
                    Err(e) => {
                        eprintln!("Error opening {}: {}", target, e);
                        process::exit(1);
                    }
                }
            }
        }
    }
}

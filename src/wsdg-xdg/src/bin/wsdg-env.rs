// WSDG-Env CLI Tool
// Command-line interface for WSDG environment management
// Part of WASMA (Windows Assignment System Monitoring Architecture)

use std::env;
use std::process;
use wsdg_xdg::{WsdgEnv, XdgWsdgTranslator, AutoCompileHelper, ShellStandard};

fn print_usage() {
    eprintln!("Usage: wsdg-env [COMMAND] [OPTIONS]");
    eprintln!();
    eprintln!("WSDG environment variable management and XDG translation");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  get <VAR>               Get environment variable value");
    eprintln!("  set <VAR> <VALUE>       Set environment variable (current session only)");
    eprintln!("  list                    List all WSDG variables");
    eprintln!("  translate <XDG_VAR>     Translate XDG variable to WSDG path");
    eprintln!("  export [--shell SHELL]  Generate shell export statements");
    eprintln!("  compile                 Compile XDG-WSDG translation buffer");
    eprintln!("  paths                   Show standard WSDG directory paths");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help              Show this help message");
    eprintln!("  -v, --version           Show version information");
    eprintln!("  --shell <SHELL>         Shell type (bash, zsh, fish) [default: bash]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  wsdg-env get HOME                    # Get HOME variable");
    eprintln!("  wsdg-env translate XDG_CONFIG_HOME   # Translate to WSDG path");
    eprintln!("  wsdg-env export --shell fish         # Generate fish export statements");
    eprintln!("  wsdg-env paths                       # Show all WSDG paths");
}

fn print_version() {
    println!("wsdg-env {}", wsdg_xdg::VERSION);
    println!("{}", wsdg_xdg::LIBRARY_INFO);
}

fn show_paths(env: &WsdgEnv) {
    println!("WSDG Directory Paths:");
    println!();
    
    if let Ok(home) = env.home_dir() {
        println!("  HOME:    {}", home.display());
    }
    
    if let Ok(config) = env.config_dir() {
        println!("  CONFIG:  {}", config.display());
    }
    
    if let Ok(local) = env.local_dir() {
        println!("  LOCAL:   {}", local.display());
    }
    
    if let Ok(share) = env.share_dir() {
        println!("  SHARE:   {}", share.display());
    }
    
    if let Ok(cache) = env.cache_dir() {
        println!("  CACHE:   {}", cache.display());
    }
    
    if let Ok(runtime) = env.runtime_dir() {
        println!("  RUNTIME: {}", runtime.display());
    }
    
    if let Ok(state) = env.state_dir() {
        println!("  STATE:   {}", state.display());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "-h" | "--help" => {
            print_usage();
            return;
        }
        "-v" | "--version" => {
            print_version();
            return;
        }
        _ => {}
    }
    
    // Initialize WSDG environment
    let mut env = WsdgEnv::new();
    
    match command.as_str() {
        "get" => {
            if args.len() < 3 {
                eprintln!("Error: Variable name required");
                eprintln!("Usage: wsdg-env get <VAR>");
                process::exit(1);
            }
            
            let var_name = &args[2];
            match env.get(var_name) {
                Some(value) => println!("{}", value),
                None => {
                    eprintln!("Variable not found: {}", var_name);
                    process::exit(1);
                }
            }
        }
        
        "set" => {
            if args.len() < 4 {
                eprintln!("Error: Variable name and value required");
                eprintln!("Usage: wsdg-env set <VAR> <VALUE>");
                process::exit(1);
            }
            
            let var_name = &args[2];
            let value = &args[3];
            
            env.set(var_name, value);
            println!("Set {} = {}", var_name, value);
        }
        
        "list" => {
            let vars = env.all_vars();
            
            if vars.is_empty() {
                println!("No WSDG variables set");
                return;
            }
            
            println!("WSDG Environment Variables:");
            println!();
            
            let mut sorted: Vec<_> = vars.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            
            for (key, value) in sorted {
                println!("  {} = {}", key, value);
            }
        }
        
        "translate" => {
            if args.len() < 3 {
                eprintln!("Error: XDG variable name required");
                eprintln!("Usage: wsdg-env translate <XDG_VAR>");
                process::exit(1);
            }
            
            let xdg_var = &args[2];
            
            match XdgWsdgTranslator::from_default() {
                Ok(mut translator) => {
                    match translator.translate_xdg(xdg_var) {
                        Ok(path) => println!("{}", path.display()),
                        Err(e) => {
                            eprintln!("Translation error: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize translator: {}", e);
                    process::exit(1);
                }
            }
        }
        
        "export" => {
            let mut shell = ShellStandard::Bash;
            
            // Parse shell option
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--shell" && i + 1 < args.len() {
                    match ShellStandard::from_str(&args[i + 1]) {
                        Ok(s) => shell = s,
                        Err(e) => {
                            eprintln!("Invalid shell: {}", e);
                            process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            
            let exports = env.export_to_shell(shell.as_str());
            
            for export in exports {
                println!("{}", export);
            }
        }
        
        "compile" => {
            println!("Compiling XDG-WSDG translation buffer...");
            
            match XdgWsdgTranslator::from_default() {
                Ok(translator) => {
                    match AutoCompileHelper::load_or_compile(translator) {
                        Ok(compiler) => {
                            if let Err(e) = compiler.save_cache() {
                                eprintln!("Warning: Failed to save cache: {}", e);
                            }
                            
                            println!("Compilation complete!");
                            println!("Compiled {} XDG paths", compiler.buffer().xdg_paths.len());
                        }
                        Err(e) => {
                            eprintln!("Compilation error: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize translator: {}", e);
                    process::exit(1);
                }
            }
        }
        
        "paths" => {
            show_paths(&env);
        }
        
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    }
}

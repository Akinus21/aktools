use clap::Parser;
use std::path::PathBuf;

mod commands;
mod modules;
mod registry;

use commands::{add, build_command, edit, list, rm, update, doctor, run};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "aktools", about = "Modular CLI tool runner", version = VERSION)]
struct Args {
    #[arg(short, long, help = "Print debug info")]
    debug: bool,
    #[arg(hide = true)]
    command: Option<String>,
    #[arg(hide = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn get_config_dir() -> PathBuf {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if let Ok(home) = std::env::var("HOME") {
            if home.contains(&sudo_user) || home == format!("/var/home/{}", sudo_user) || home == format!("/home/{}", sudo_user) {
                return PathBuf::from(&home).join(".aktools");
            }
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aktools")
}

fn get_modules_dir() -> PathBuf {
    get_config_dir().join("modules")
}

fn get_registry_path() -> PathBuf {
    get_config_dir().join("registry.json")
}

fn main() {
    let args = Args::parse();
    let config_dir = get_config_dir();
    let modules_dir = get_modules_dir();
    let registry_path = get_registry_path();

    if args.debug {
        eprintln!("Config dir: {:?}", config_dir);
        eprintln!("Modules dir: {:?}", modules_dir);
        eprintln!("Registry: {:?}", registry_path);
    }

    let exit_code = match args.command.as_deref() {
        Some("add") => add::execute(&modules_dir, &registry_path, args.args.first().cloned()),
        Some("edit") => edit::execute(&modules_dir, &registry_path, args.args.first().cloned()),
        Some("build-command") => build_command::execute(&modules_dir, &registry_path),
        Some("rm") => rm::execute(&modules_dir, &registry_path, args.args.first().cloned()),
        Some("update") => update::execute(&modules_dir, &registry_path),
        Some("list") => list::execute(&modules_dir),
        Some("doctor") => {
            let no_fix = args.args.iter().any(|a| a == "--no-fix" || a == "--dry-run");
            doctor::execute(&config_dir, &modules_dir, no_fix)
        }
Some("build-command") => build_command::execute(&modules_dir, &registry_path),
        Some("help") => {
            println!("AKTools - Modular CLI tool runner\n");
            println!("Commands:");
            println!("  aktools build-command   Create a new command module interactively");
            println!("  aktools add <file>     Add a script as a module");
            println!("  aktools edit [name]    Edit a module's manifest");
            println!("  aktools list           List installed modules");
            println!("  aktools rm <name>      Remove a module");
            println!("  aktools update         Rebuild the registry");
            println!("  aktools doctor         Diagnose and auto-fix issues");
            println!("  aktools <module> [args...]  Run a module");
            0
        }
        Some(module_name) => run::execute(&modules_dir, &registry_path, module_name, args.args),
        None => {
            println!("AKTools - Modular CLI tool runner\n");
            println!("Commands:");
            println!("  add      Add a script as a module");
            println!("  edit     Edit a module's manifest");
            println!("  list     List installed modules");
            println!("  rm       Remove a module");
            println!("  update   Rebuild the registry");
            println!("  doctor   Diagnose and auto-fix issues\n");
            println!("Run 'aktools <module> [args]' to execute a module.");
            0
        }
    };

    std::process::exit(exit_code);
}
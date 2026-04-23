use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod modules;
mod registry;

use commands::{add, edit, list, rm, update, doctor};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "Akinus21/aktools";

#[derive(Parser, Debug)]
#[command(name = "aktools", about = "Modular CLI tool runner")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(short, long, help = "Print debug info")]
    debug: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Add {
        #[arg(help = "Add a script as a module")]
        filename: Option<String>,
    },
    Edit {
        #[arg(help = "Edit a module's manifest")]
        module_name: Option<String>,
    },
    Rm {
        #[arg(help = "Remove a module")]
        module_name: Option<String>,
    },
    Update {
        #[arg(help = "Rebuild the module registry")]
    },
    List {
        #[arg(help = "List installed modules")]
    },
    Doctor {
        #[arg(short, long, help = "Show issues without fixing them", alias = "dry-run")]
        no_fix: bool,
    },
}

fn get_config_dir() -> PathBuf {
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

    std::process::exit(match args.command {
        Some(Command::Add { filename }) => add::execute(&modules_dir, &registry_path, filename),
        Some(Command::Edit { module_name }) => edit::execute(&modules_dir, &registry_path, module_name),
        Some(Command::Rm { module_name }) => rm::execute(&modules_dir, &registry_path, module_name),
        Some(Command::Update) => update::execute(&modules_dir, &registry_path),
        Some(Command::List) => list::execute(&modules_dir),
        Some(Command::Doctor { no_fix }) => doctor::execute(&config_dir, &modules_dir, no_fix),
        None => {
            println!("AKTools - Modular CLI tool runner\n");
            println!("Commands:");
            println!("  add      Add a script as a module");
            println!("  edit     Edit a module's manifest");
            println!("  list     List installed modules");
            println!("  rm       Remove a module");
            println!("  update   Rebuild the registry");
            println!("  doctor   Diagnose and auto-fix issues");
            println!("  help     Show this help message\n");
            println!("Run 'aktools <command> --help' for more info.");
            0
        }
    });
}
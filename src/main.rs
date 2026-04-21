use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod modules;
mod registry;

use commands::{add, edit, rm, update, doctor, help_cmd::help};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "Akinus21/aktools";

fn check_for_updates() {
    if let Ok(response) = ureq::get(&format!("https://api.github.com/repos/{}/releases/latest", REPO))
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        if let Some(tag) = response.get("tag_name").and_then(|t| t.as_str()) {
            let latest = tag.trim_start_matches('v');
            if latest != VERSION {
                println!("Update available: v{} -> v{}", VERSION, latest);
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "aktools")]
#[command(about = "AKTools - Modular CLI tool runner", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(short, long, help = "Print debug info")]
    debug: bool,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Add {
        #[arg(help = "File to add as module")]
        filename: Option<String>,
    },
    Edit {
        #[arg(help = "Module name to edit")]
        module_name: Option<String>,
    },
    Rm {
        #[arg(help = "Module name to remove")]
        module_name: Option<String>,
    },
    Update,
    Doctor,
    Help,
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
        Some(Command::Doctor) => doctor::execute(&config_dir, &modules_dir),
        Some(Command::Help) => help(),
        None => {
            check_for_updates();
            println!("AKTools - Modular CLI tool runner");
            println!("Run 'aktools help' for usage information");
            0
        }
    });
}
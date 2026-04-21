use std::path::Path;
use crate::modules::ModuleManager;
use crate::registry::Registry;

pub fn execute(modules_dir: &Path, registry_path: &Path, module_name: Option<String>) -> i32 {
    let modules = match ModuleManager::scan_modules(modules_dir) {
        Ok(m) => m,
        Err(e) => {
            println!("Error scanning modules: {}", e);
            return 1;
        }
    };

    let selected_name = if let Some(name) = module_name {
        if !modules.contains_key(&name) {
            println!("Error: module '{}' not found", name);
            return 1;
        }
        name
    } else {
        loop {
            println!("\nInstalled modules:");
            let mut names: Vec<_> = modules.keys().collect();
            names.sort();
            for (i, name) in names.iter().enumerate() {
                println!("  {} - {}", i + 1, name);
            }
            println!("  q - quit");

            print!("\nSelect module to remove: ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Error reading input");
                continue;
            }

            let input = input.trim();
            if input == "q" {
                return 0;
            }

            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= names.len() {
                    break names[idx - 1].clone();
                }
            }
            println!("Invalid selection");
        }
    };

    print!("Are you sure you want to remove '{}'? (y/N): ", selected_name);
    std::io::stdout().flush().unwrap();
    let mut confirm = String::new();
    if std::io::stdin().read_line(&mut confirm).is_err() {
        println!("Error reading input");
        return 1;
    }

    if confirm.trim().to_lowercase() != "y" {
        println!("Cancelled");
        return 0;
    }

    let module_dir = modules_dir.join(&selected_name);
    match std::fs::remove_dir_all(&module_dir) {
        Ok(_) => {
            println!("Removed module directory: {:?}", module_dir);
            let mut registry = match Registry::load(registry_path) {
                Ok(r) => r,
                Err(e) => {
                    println!("Warning: failed to load registry: {}", e);
                    return 0;
                }
            };
            registry.remove_module(&selected_name);
            if let Err(e) = registry.save(registry_path) {
                println!("Warning: failed to update registry: {}", e);
            }
            0
        }
        Err(e) => {
            println!("Error removing module: {}", e);
            1
        }
    }
}
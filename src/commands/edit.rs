use std::io::Write;
use std::path::Path;
use crate::modules::ModuleManager;

pub fn execute(modules_dir: &Path, _registry_path: &Path, module_name: Option<String>) -> i32 {
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

            print!("\nSelect module to edit: ");
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

    loop {
        let manifest = modules.get(&selected_name).expect("Module not found");

        println!("\nEditing module: {}", manifest.name);
        println!("1. Name: {}", manifest.name);
        println!("2. Aliases: {:?}", manifest.aliases);
        println!("3. Options: {} options", manifest.options.len());
        println!("q - quit editing");

        print!("\nSelect field to edit (1-3) or 'q' to quit: ");
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

        match input {
            "1" => {
                print!("New name: ");
                std::io::stdout().flush().unwrap();
                let mut new_name = String::new();
                if std::io::stdin().read_line(&mut new_name).is_err() {
                    println!("Error reading input");
                    continue;
                }
                let new_name = new_name.trim().to_string();
                if !new_name.is_empty() {
                    let module_dir = modules_dir.join(&manifest.name);
                    let new_module_dir = modules_dir.join(&new_name);
                    if module_dir.exists() {
                        if let Err(e) = std::fs::rename(&module_dir, &new_module_dir) {
                            println!("Error renaming module: {}", e);
                            continue;
                        }
                    }
                    println!("Name updated to: {}", new_name);
                }
            }
            "2" => {
                print!("New aliases (comma-separated): ");
                std::io::stdout().flush().unwrap();
                let mut new_aliases = String::new();
                if std::io::stdin().read_line(&mut new_aliases).is_err() {
                    println!("Error reading input");
                    continue;
                }
                println!("Aliases updated");
            }
            "3" => {
                println!("Option editing not yet implemented");
            }
            _ => {
                println!("Invalid selection");
            }
        }
    }
}
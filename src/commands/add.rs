use std::path::Path;
use crate::modules::ModuleManager;
use crate::registry::Registry;

pub fn execute(modules_dir: &Path, registry_path: &Path, filename: Option<String>) -> i32 {
    let filename = match filename {
        Some(f) => f,
        None => {
            println!("Error: filename required");
            println!("Usage: aktools add <filename>");
            return 1;
        }
    };

    let source_path = Path::new(&filename);
    if !source_path.exists() {
        println!("Error: file '{}' not found", filename);
        return 1;
    }

    print!("Module name: ");
    std::io::stdout().flush().unwrap();
    let mut name = String::new();
    if std::io::stdin().read_line(&mut name).is_err() {
        println!("Error reading input");
        return 1;
    }
    let name = name.trim().to_string();
    if name.is_empty() {
        println!("Error: module name cannot be empty");
        return 1;
    }

    print!("Aliases (comma-separated): ");
    std::io::stdout().flush().unwrap();
    let mut aliases_input = String::new();
    if std::io::stdin().read_line(&mut aliases_input).is_err() {
        println!("Error reading input");
        return 1;
    }
    let aliases: Vec<String> = aliases_input
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut registry = match Registry::load(registry_path) {
        Ok(r) => r,
        Err(e) => {
            println!("Error loading registry: {}", e);
            return 1;
        }
    };

    for alias in &aliases {
        for module in registry.modules.values() {
            if module.aliases.contains(alias) {
                println!("Error: alias '{}' is already used by module '{}'", alias, module.name);
                return 1;
            }
        }
    }

    match ModuleManager::create_module_folder(modules_dir, &name, &aliases, source_path) {
        Ok(module_dir) => {
            println!("Created module at: {:?}", module_dir);
            let modules = ModuleManager::scan_modules(modules_dir).unwrap_or_default();
            if let Some(manifest) = modules.get(&name) {
                let module = crate::registry::Module {
                    name: manifest.name.clone(),
                    folder: name.clone(),
                    aliases: manifest.aliases.clone(),
                    commands: manifest.options.iter().map(|opt| {
                        (opt.flags.get(0).cloned().unwrap_or_default(), opt.commands.clone())
                    }).collect(),
                };
                registry.add_module(module);
                if let Err(e) = registry.save(registry_path) {
                    println!("Warning: failed to save registry: {}", e);
                }
            }
            0
        }
        Err(e) => {
            println!("Error creating module: {}", e);
            1
        }
    }
}
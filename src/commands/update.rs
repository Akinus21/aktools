use std::path::Path;
use crate::modules::ModuleManager;
use crate::registry::{Registry, Module};
use std::collections::HashMap;

pub fn execute(modules_dir: &Path, registry_path: &Path) -> i32 {
    println!("Scanning modules from: {:?}", modules_dir);

    let modules = match ModuleManager::scan_modules(modules_dir) {
        Ok(m) => {
            println!("Found {} modules", m.len());
            m
        }
        Err(e) => {
            println!("Error scanning modules: {}", e);
            return 1;
        }
    };

    let mut registry = Registry::default();

    for (name, manifest) in modules {
        let commands: HashMap<String, Vec<String>> = manifest.options.iter()
            .map(|opt| {
                let flag = opt.flags.get(0).cloned().unwrap_or_default().trim_start_matches('*').to_string();
                (flag, opt.commands.clone())
            })
            .collect();

        let module = Module {
            name: manifest.name.clone(),
            folder: name.clone(),
            aliases: manifest.aliases.clone(),
            commands,
        };
        registry.add_module(module);
    }

    match registry.save(registry_path) {
        Ok(_) => {
            println!("Registry saved to: {:?}", registry_path);
            0
        }
        Err(e) => {
            println!("Error saving registry: {}", e);
            1
        }
    }
}
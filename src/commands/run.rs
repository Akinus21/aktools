use std::path::Path;
use std::process::{Command, Stdio};
use crate::registry::Registry;

pub fn execute(modules_dir: &Path, registry_path: &Path, module_name: &str, args: Vec<String>) -> i32 {
    let registry = match Registry::load(registry_path) {
        Ok(r) => r,
        Err(e) => {
            println!("Error loading registry: {}", e);
            return 1;
        }
    };

    let module = match registry.modules.get(module_name) {
        Some(m) => m,
        None => {
            println!("Error: module '{}' not found", module_name);
            println!("Run 'aktools list' to see available modules.");
            return 1;
        }
    };

    let module_path = modules_dir.join(&module.folder);
    if !module_path.exists() {
        println!("Error: module folder not found at {:?}", module_path);
        return 1;
    }

    let manifest = match crate::modules::ModuleManager::load_manifest(&module_path) {
        Ok(m) => m,
        Err(e) => {
            println!("Error loading module manifest: {}", e);
            return 1;
        }
    };

    let default_opt = manifest.options.iter().find(|opt| opt._is_default).or(manifest.options.first());

    let (opt, remaining_args) = if let Some(first_arg) = args.first() {
        let matched_opt = manifest.options.iter().find(|o| o.flags.iter().any(|f| f.trim_start_matches('*') == first_arg));
        match matched_opt {
            Some(o) => (o, args[1..].to_vec()),
            None => {
                let opt = match default_opt {
                    Some(o) => o,
                    None => {
                        println!("Error: no options defined for module '{}'", module_name);
                        return 1;
                    }
                };
                (opt, args.clone())
            }
        }
    } else {
        match default_opt {
            Some(o) => (o, vec![]),
            None => {
                println!("Error: no options defined for module '{}'", module_name);
                return 1;
            }
        }
    };

    let cmd_str = opt.commands.first().map(|s| s.as_str()).unwrap_or("");
    let mut parts = cmd_str.split_whitespace();
    let program = parts.next().unwrap_or("");
    let program_path = module_path.join(program);

    let mut cmd = if program_path.exists() {
        let ext = program_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.as_ref() {
            "py" => {
                let mut c = Command::new("python3");
                c.arg(&program_path);
                c
            }
            "sh" | "bash" => {
                let mut c = Command::new("bash");
                c.arg(&program_path);
                c
            }
            _ => {
                let c = Command::new(&program_path);
                c
            }
        }
    } else {
        let c = Command::new(program);
        c
    };

    cmd.args(&remaining_args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    match cmd.spawn() {
        Ok(mut child) => child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1),
        Err(e) => {
            println!("Error executing module: {}", e);
            1
        }
    }
}
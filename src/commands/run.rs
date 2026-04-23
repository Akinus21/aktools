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

    let flag = if args.is_empty() {
        manifest.options.iter()
            .find(|opt| opt._is_default)
            .or(manifest.options.first())
            .map(|_| String::new())
    } else {
        Some(args.remove(0))
    };

    let flag = match flag {
        Some(f) => f,
        None => {
            println!("Error: no default flag defined for module '{}'", module_name);
            return 1;
        }
    };

    let opt = manifest.options.iter()
        .find(|o| o.flags.iter().any(|f| f.trim_start_matches('*') == flag))
        .or(manifest.options.first());

    let opt = match opt {
        Some(o) => o,
        None => {
            println!("Error: flag '{}' not found in module '{}'", flag, module_name);
            return 1;
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
                let mut c = Command::new(&program_path);
                c
            }
        }
    } else {
        let mut c = Command::new(program);
        c
    };

    cmd.args(&args);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.current_dir(&module_path);

    match cmd.spawn() {
        Ok(mut child) => child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1),
        Err(e) => {
            println!("Error executing module: {}", e);
            1
        }
    }
}
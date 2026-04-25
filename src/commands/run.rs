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

    if !manifest.executable.is_empty() {
        let executable_path = module_path.join(&manifest.executable);
        let ext = executable_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut cmd = match ext.as_ref() {
            "py" => {
                let mut c = Command::new("python3");
                c.arg(&executable_path);
                c
            }
            "sh" | "bash" => {
                let mut c = Command::new("bash");
                c.arg(&executable_path);
                c
            }
            _ => Command::new(&executable_path),
        };

        cmd.args(&args);
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        match cmd.spawn() {
            Ok(mut child) => child.wait().map(|s| s.code().unwrap_or(1)).unwrap_or(1),
            Err(e) => {
                println!("Error executing module: {}", e);
                1
            }
        }
    } else {
        let matched_opt = if let Some(first_arg) = args.first() {
            manifest.options.iter().find(|o| o.flags.iter().any(|f| f.trim_start_matches('*') == first_arg))
        } else {
            None
        };

        let opt = match matched_opt {
            Some(o) => o,
            None => {
                println!("Error: no matching flag found for '{}'", args.first().unwrap_or(&"".to_string()));
                return 1;
            }
        };

        for cmd_str in &opt.commands {
            let has_shell_operator = cmd_str.contains("&&") || cmd_str.contains("||")
                || cmd_str.contains("|") || cmd_str.contains(";")
                || cmd_str.starts_with("sudo") || cmd_str.contains(" &")
                || cmd_str.ends_with(" &") || cmd_str.trim_end().ends_with("&");

            if has_shell_operator {
                let full_cmd = format!("sh -c '{}'", cmd_str.replace("'", "'\"'\"'"));
                let mut cmd = Command::new("sh");
                cmd.arg("-c").arg(&full_cmd);
                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());
                if let Err(e) = cmd.spawn() {
                    println!("Error executing command '{}': {}", cmd_str, e);
                    return 1;
                }
            } else {
                let mut parts = cmd_str.split_whitespace();
                let program = parts.next().unwrap_or("");
                let mut cmd = Command::new(program);
                cmd.args(parts);
                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());
                if let Err(e) = cmd.spawn() {
                    println!("Error executing command '{}': {}", cmd_str, e);
                    return 1;
                }
            }
        }
        0
    }
}
use std::path::Path;
use std::process::Command;

pub fn execute(config_dir: &Path, modules_dir: &Path) -> i32 {
    println!("AKTools Doctor - Diagnosing issues...\n");

    let mut issues_found = 0;

    println!("Checking module directories...");
    if !config_dir.exists() {
        println!("  [WARN] Config directory does not exist: {:?}", config_dir);
        issues_found += 1;
    }

    if !modules_dir.exists() {
        println!("  [WARN] Modules directory does not exist: {:?}", modules_dir);
        issues_found += 1;
    }

    println!("\nChecking shell configuration...");
    let home = dirs::home_dir().unwrap_or_default();
    let shell_files = vec![
        home.join(".bashrc"),
        home.join(".zshrc"),
    ];

    for shell_file in &shell_files {
        if shell_file.exists() {
            let content = std::fs::read_to_string(shell_file).unwrap_or_default();
            if content.contains("aktools") {
                println!("  [OK] aktools found in {:?}", shell_file);
            }
        }
    }

    let aliases_file = config_dir.join("aliases.sh");
    if aliases_file.exists() {
        println!("  [OK] Aliases file exists: {:?}", aliases_file);
    } else {
        println!("  [WARN] Aliases file not found: {:?}", aliases_file);
        issues_found += 1;
    }

    println!("\nChecking for updates...");
    println!("  (Update check not yet implemented)");

    println!("\nChecking module integrity...");
    if let Ok(modules) = crate::modules::ModuleManager::scan_modules(modules_dir) {
        if modules.is_empty() {
            println!("  [INFO] No modules installed");
        } else {
            println!("  [OK] {} modules found", modules.len());
        }
    }

    if issues_found > 0 {
        println!("\n[ERROR] {} issues found. Run 'aktools update' to fix registry.", issues_found);
        1
    } else {
        println!("\n[OK] No issues detected");
        0
    }
}
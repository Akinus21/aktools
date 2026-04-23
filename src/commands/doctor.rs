use std::path::Path;

pub fn execute(config_dir: &Path, modules_dir: &Path, no_fix: bool) -> i32 {
    println!("AKTools Doctor - Diagnosing issues...\n");

    let mut issues_found = 0;
    let mut fixed = 0;

    println!("Checking module directories...");
    if !config_dir.exists() {
        if !no_fix {
            match std::fs::create_dir_all(config_dir) {
                Ok(_) => {
                    println!("  [FIXED] Created config directory: {:?}", config_dir);
                    fixed += 1;
                }
                Err(e) => {
                    println!("  [ERROR] Failed to create config directory: {}", e);
                    issues_found += 1;
                }
            }
        } else {
            println!("  [WARN] Config directory does not exist: {:?}", config_dir);
            println!("         Run without --no-fix to create it automatically");
            issues_found += 1;
        }
    } else {
        println!("  [OK] Config directory exists");
    }

    if !modules_dir.exists() {
        if !no_fix {
            match std::fs::create_dir_all(modules_dir) {
                Ok(_) => {
                    println!("  [FIXED] Created modules directory: {:?}", modules_dir);
                    fixed += 1;
                }
                Err(e) => {
                    println!("  [ERROR] Failed to create modules directory: {}", e);
                    issues_found += 1;
                }
            }
        } else {
            println!("  [WARN] Modules directory does not exist: {:?}", modules_dir);
            println!("         Run without --no-fix to create it automatically");
            issues_found += 1;
        }
    } else {
        println!("  [OK] Modules directory exists");
    }

    println!("\nChecking shell configuration...");
    let home = dirs::home_dir().unwrap_or_default();
    let shell_files = vec![
        home.join(".bashrc"),
        home.join(".zshrc"),
    ];

    let mut aktools_in_shell = false;
    for shell_file in &shell_files {
        if shell_file.exists() {
            let content = std::fs::read_to_string(shell_file).unwrap_or_default();
            if content.contains("aktools") {
                println!("  [OK] aktools found in {:?}", shell_file);
                aktools_in_shell = true;
            }
        }
    }

    if !aktools_in_shell && !no_fix {
        let export_line = format!("\n# AKTools\nexport AKTOOLS_HOME=\"{}\"\nexport PATH=\"$AKTOOLS_HOME/bin:$PATH\"\n", config_dir.display());
        for shell_file in &shell_files {
            if shell_file.exists() {
                match std::fs::read_to_string(shell_file) {
                    Ok(content) => {
                        if !content.contains("aktools") {
                            match std::fs::write(shell_file, content + &export_line) {
                                Ok(_) => {
                                    println!("  [FIXED] Added AKTools to {:?}", shell_file);
                                    fixed += 1;
                                }
                                Err(e) => {
                                    println!("  [ERROR] Failed to update {:?}: {}", shell_file, e);
                                    issues_found += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("  [ERROR] Failed to read {:?}: {}", shell_file, e);
                        issues_found += 1;
                    }
                }
            }
        }
    } else if !aktools_in_shell {
        println!("  [WARN] AKTools not found in shell configuration");
        println!("         Run without --no-fix to add it automatically");
        issues_found += 1;
    }

    let aliases_file = config_dir.join("aliases.sh");
    if aliases_file.exists() {
        println!("  [OK] Aliases file exists: {:?}", aliases_file);
    } else {
        if !no_fix {
            let default_aliases = r#"# AKTools Aliases
# This file is auto-generated

alias ak='aktools'
alias aktools-update='aktools update'
alias aktools-doctor='aktools doctor'
alias aktools-add='aktools add'
alias aktools-rm='aktools rm'
alias aktools-edit='aktools edit'
"#;
            match std::fs::write(&aliases_file, default_aliases) {
                Ok(_) => {
                    println!("  [FIXED] Created aliases file: {:?}", aliases_file);
                    fixed += 1;
                }
                Err(e) => {
                    println!("  [ERROR] Failed to create aliases file: {}", e);
                    issues_found += 1;
                }
            }
        } else {
            println!("  [WARN] Aliases file not found: {:?}", aliases_file);
            println!("         Run without --no-fix to create it automatically");
            issues_found += 1;
        }
    }

    let registry_file = config_dir.join("registry.json");
    if !registry_file.exists() {
        if !no_fix {
            match std::fs::write(&registry_file, "{\"modules\": []}") {
                Ok(_) => {
                    println!("  [FIXED] Created registry file: {:?}", registry_file);
                    fixed += 1;
                }
                Err(e) => {
                    println!("  [ERROR] Failed to create registry: {}", e);
                    issues_found += 1;
                }
            }
        } else {
            println!("  [WARN] Registry file not found: {:?}", registry_file);
            println!("         Run without --no-fix to create it automatically");
            issues_found += 1;
        }
    }

    println!("\nChecking binary installation...");
    let bin_dir = config_dir.join("bin");
    let aktools_bin = bin_dir.join("aktools");
    if aktools_bin.exists() {
        if let Ok(metadata) = std::fs::metadata(&aktools_bin) {
            println!("  [OK] aktools binary found: {} bytes", metadata.len());
        }
    } else {
        println!("  [WARN] aktools binary not found in {:?}", aktools_bin);
        println!("         Run 'aktools upgrade' to install");
        issues_found += 1;
    }

    println!("\nChecking for updates...");
    if let Ok(response) = ureq::get("https://api.github.com/repos/Akinus21/aktools/releases/latest")
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        if let Ok(body) = response.into_string() {
            if let Ok(releases) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(tag_name) = releases.get("tag_name").and_then(|t| t.as_str()) {
                    let current_version = env!("CARGO_PKG_VERSION");
                    if tag_name != format!("v{}", current_version) {
                        println!("  [INFO] Update available: {} -> {}", current_version, tag_name);
                    } else {
                        println!("  [OK] aktools is up to date (v{})", current_version);
                    }
                }
            }
        } else {
            println!("  [INFO] Could not parse releases");
        }
    } else {
        println!("  [INFO] Could not connect to check for updates");
    }

    println!("\nChecking module integrity...");
    if let Ok(modules) = crate::modules::ModuleManager::scan_modules(modules_dir) {
        if modules.is_empty() {
            println!("  [INFO] No modules installed");
        } else {
            println!("  [OK] {} modules found", modules.len());
        }
    }

    if !no_fix && fixed > 0 {
        println!("\n[FIXED] {} issues fixed automatically", fixed);
    }

    if issues_found > 0 {
        if !no_fix {
            println!("\n[ERROR] {} issues found (some auto-fixed).", issues_found);
        } else {
            println!("\n[ERROR] {} issues found. Run without --no-fix to auto-fix.", issues_found);
        }
        1
    } else if !no_fix && fixed == 0 {
        println!("\n[OK] Everything looks good!");
        0
    } else {
        println!("\n[OK] No issues detected");
        0
    }
}
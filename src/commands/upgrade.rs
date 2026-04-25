use std::path::{Path, PathBuf};
use std::fs;

const DEFAULT_COMMUNITY_REPO: &str = "Akinus21/aktools-modules";

#[derive(serde::Serialize, serde::Deserialize)]
struct RepoConfig {
    repos: Vec<Repo>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Repo {
    user: String,
    repo: String,
    is_default: bool,
}

#[derive(serde::Deserialize)]
struct RegistryJson {
    version: u32,
    modules: Vec<RegistryModule>,
}

#[derive(serde::Deserialize)]
struct RegistryModule {
    id: String,
    name: String,
    version: String,
    author: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    min_aktools_version: Option<String>,
    last_updated: Option<String>,
}

pub fn execute(config_dir: &Path, args: Vec<String>) -> i32 {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("all");

    match subcommand {
        "aktools" | "self" => upgrade_aktools(),
        "modules" | "mods" => {
            let repos_file = config_dir.join("repos.json");
            let modules_dir = config_dir.join("modules");
            upgrade_modules(&repos_file, &modules_dir)
        }
        "all" | _ => {
            let result = upgrade_aktools();
            if result != 0 {
                return result;
            }
            let repos_file = config_dir.join("repos.json");
            let modules_dir = config_dir.join("modules");
            upgrade_modules(&repos_file, &modules_dir)
        }
    }
}

fn upgrade_aktools() -> i32 {
    println!("Upgrading AKTools via Homebrew...");

    let update_result = std::process::Command::new("brew")
        .args(["update"])
        .output();

    match update_result {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Warning: 'brew update' failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Error running brew update: {}", e);
            eprintln!("Make sure Homebrew is installed and in your PATH.");
            return 1;
        }
    }

    let upgrade_result = std::process::Command::new("brew")
        .args(["upgrade", "aktools"])
        .output();

    match upgrade_result {
        Ok(output) => {
            if output.status.success() {
                println!("AKTools upgraded successfully!");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                0
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Not a keyword") || stderr.contains("Cask") {
                    println!("AKTools is a cask. Trying 'brew upgrade --cask aktools'...");
                    let cask_result = std::process::Command::new("brew")
                        .args(["upgrade", "--cask", "aktools"])
                        .output();
                    if let Ok(cask_output) = cask_result {
                        if cask_output.status.success() {
                            println!("AKTools upgraded successfully!");
                            println!("{}", String::from_utf8_lossy(&cask_output.stdout));
                            return 0;
                        }
                    }
                }
                eprintln!("Warning: 'brew upgrade aktools' may have failed:");
                eprintln!("{}", stderr);
                1
            }
        }
        Err(e) => {
            eprintln!("Error running brew upgrade: {}", e);
            1
        }
    }
}

fn load_repos_config(repos_file: &Path) -> RepoConfig {
    if let Ok(content) = fs::read_to_string(repos_file) {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    RepoConfig { repos: Vec::new() }
}

fn upgrade_modules(repos_file: &Path, modules_dir: &Path) -> i32 {
    println!("Checking for module updates...\n");

    let config = load_repos_config(repos_file);

    let repos_to_check: Vec<Repo> = if config.repos.is_empty() {
        vec![Repo {
            user: DEFAULT_COMMUNITY_REPO.split('/').next().unwrap().to_string(),
            repo: DEFAULT_COMMUNITY_REPO.split('/').nth(1).unwrap().to_string(),
            is_default: true,
        }]
    } else {
        config.repos.clone()
    };

    if modules_dir.exists() {
        if let Ok(entries) = fs::read_dir(modules_dir) {
            let local_modules: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();

            let mut updated = 0;
            let mut failed: Vec<String> = Vec::new();

            for module_name in local_modules {
                let module_path = modules_dir.join(&module_name);

                for repo in &repos_to_check {
                    let registry_url = format!(
                        "https://raw.githubusercontent.com/{}/{}/main/registry.json",
                        repo.user, repo.repo
                    );

                    match ureq::get(&registry_url).call() {
                        Ok(resp) => {
                            if let Ok(body) = resp.into_string() {
                                if let Ok(registry) = serde_json::from_str::<RegistryJson>(&body) {
                                    if let Some(remote_module) = registry.modules.iter().find(|m| m.id.to_lowercase() == module_name.to_lowercase()) {
                                        let manifest_path = module_path.join("manifest.xml");
                                        if let Ok(local_content) = fs::read_to_string(&manifest_path) {
                                            if let Some(local_version) = extract_version_from_manifest(&local_content) {
                                                if remote_module.version != local_version {
                                                    println!("Updating '{}': {} -> {}",
                                                        module_name, local_version, remote_module.version);

                                                    if let Err(e) = download_module(&module_name, &repo, modules_dir) {
                                                        eprintln!("  Failed to update: {}", e);
                                                        failed.push(module_name.clone());
                                                    } else {
                                                        updated += 1;
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            if updated > 0 {
                let _ = crate::modules::ModuleManager::_write_aliases_to_file(modules_dir, &modules_dir.parent().unwrap().join("aliases.sh"));
                let _ = crate::commands::update::execute(modules_dir, &modules_dir.parent().unwrap().join("registry.json"));
            }

            println!("\nModule update complete: {} updated, {} failed", updated, failed.len());
            if !failed.is_empty() {
                println!("Failed: {}", failed.join(", "));
            }

            return if failed.is_empty() { 0 } else { 1 };
        }
    }

    println!("No modules to check or modules directory not found.");
    0
}

fn extract_version_from_manifest(content: &str) -> Option<String> {
    if let Some(start) = content.find("<version>") {
        let start = start + 9;
        if let Some(end) = content.find("</version>") {
            return Some(content[start..end].to_string());
        }
    }
    None
}

fn download_module(module_name: &str, repo: &Repo, modules_dir: &Path) -> Result<(), String> {
    let module_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/{}/manifest.xml",
        repo.user, repo.repo, module_name
    );

    let response = ureq::get(&module_url)
        .call()
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    let manifest_xml = response
        .into_string()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let module_path = modules_dir.join(module_name);
    fs::create_dir_all(&module_path)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    fs::write(module_path.join("manifest.xml"), manifest_xml)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    Ok(())
}

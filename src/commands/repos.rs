use std::path::Path;
use std::fs;

const DEFAULT_COMMUNITY_REPO: &str = "noctalia-dev/noctalia-plugins";

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
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("list-repos");
    let module_dir = config_dir.join("modules");
    let repos_file = config_dir.join("repos.json");

    match subcommand {
        "add-repo" => add_repo(&repos_file, &args[1..]),
        "list-repos" => list_repos(&repos_file),
        "search-mods" => search_modules(&repos_file, &args[1..]),
        "install-mods" => install_modules(&repos_file, &module_dir, config_dir, &args[1..]),
        _ => {
            println!("Unknown subcommand: {}", subcommand);
            println!("Usage:");
            println!("  aktools add-repo <user/repo>   Add a repo to track");
            println!("  aktools list-repos             List configured repos");
            println!("  aktools search-mods <term>     Search modules");
            println!("  aktools install-mods <mod> [<mod>...]  Install module(s)");
            1
        }
    }
}

fn add_repo(repos_file: &Path, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Usage: aktools add-repo <user/repo>");
        println!("Example: aktools add-repo myname/my-plugins");
        return 1;
    }

    let input = &args[0];
    if !input.contains('/') {
        println!("Invalid repo format. Use: user/repo");
        return 1;
    }

    let parts: Vec<&str> = input.split('/').collect();
    let user = parts[0].to_string();
    let repo = parts[1].to_string();

    let mut config = load_repos_config(repos_file);

    if config.repos.iter().any(|r| r.user == user && r.repo == repo) {
        println!("Repo {}/{} is already tracked", user, repo);
        return 1;
    }

    let test_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/registry.json",
        user, repo
    );

    println!("Checking repo...");
    if let Err(e) = ureq::get(&test_url).call() {
        println!("Error: Could not fetch {}/{} - {}", user, repo, e);
        return 1;
    }

    config.repos.push(Repo {
        user,
        repo,
        is_default: false,
    });

    if let Err(e) = fs::write(repos_file, serde_json::to_string_pretty(&config).unwrap()) {
        println!("Error saving repos config: {}", e);
        return 1;
    }

    println!("Added repo successfully");
    0
}

fn list_repos(repos_file: &Path) -> i32 {
    let config = load_repos_config(repos_file);

    if config.repos.is_empty() {
        println!("No repos configured. Using default community repo:");
        println!("  {} (default)", DEFAULT_COMMUNITY_REPO);
        return 0;
    }

    println!("Configured repos:");
    for repo in &config.repos {
        let mark = if repo.is_default { " [default]" } else { "" };
        println!("  {}/{}{}", repo.user, repo.repo, mark);
    }
    0
}

fn search_modules(repos_file: &Path, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Usage: aktools search-mods <term>");
        return 1;
    }

    let query = args.join(" ").to_lowercase();
    let config = load_repos_config(repos_file);
    let mut all_modules: Vec<(String, RegistryModule)> = Vec::new();

    let repos_to_search: Vec<Repo> = if config.repos.is_empty() {
        vec![Repo {
            user: DEFAULT_COMMUNITY_REPO.split('/').next().unwrap().to_string(),
            repo: DEFAULT_COMMUNITY_REPO.split('/').nth(1).unwrap().to_string(),
            is_default: true,
        }]
    } else {
        config.repos
    };

    println!("Searching for: {}", query);
    for repo in repos_to_search {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/main/registry.json",
            repo.user, repo.repo
        );

        match ureq::get(&url).call() {
            Ok(response) => {
                if let Ok(body) = response.into_string() {
                    if let Ok(registry) = serde_json::from_str::<RegistryJson>(&body) {
                        for module in registry.modules {
                            if module.id.to_lowercase().contains(&query)
                                || module.name.to_lowercase().contains(&query)
                                || module.description.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
                                || module.tags.as_ref().map(|t| t.iter().any(|tag| tag.to_lowercase().contains(&query))).unwrap_or(false)
                            {
                                all_modules.push((format!("{}/{}", repo.user, repo.repo), module));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not fetch {}: {}", repo.repo, e);
            }
        }
    }

    if all_modules.is_empty() {
        println!("No modules found matching: {}", query);
    } else {
        println!("\nFound {} module(s):", all_modules.len());
        for (repo_name, module) in all_modules {
            println!("\n  [{}]", repo_name);
            println!("  {:20} ({})", module.name, module.id);
            if let Some(desc) = &module.description {
                println!("  {}", desc);
            }
            if let Some(tags) = &module.tags {
                if !tags.is_empty() {
                    println!("  Tags: {}", tags.join(", "));
                }
            }
        }
    }
    0
}

fn install_modules(repos_file: &Path, modules_dir: &Path, config_dir: &Path, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Usage: aktools install-mods <module> [<module>...]");
        return 1;
    }

    let module_names: Vec<String> = args.iter().map(|s| s.to_lowercase()).collect();
    let config = load_repos_config(repos_file);

    let repos_to_search: Vec<Repo> = if config.repos.is_empty() {
        vec![Repo {
            user: DEFAULT_COMMUNITY_REPO.split('/').next().unwrap().to_string(),
            repo: DEFAULT_COMMUNITY_REPO.split('/').nth(1).unwrap().to_string(),
            is_default: true,
        }]
    } else {
        config.repos
    };

    let mut installed = 0;
    let mut failed: Vec<String> = Vec::new();

    for module_name in &module_names {
        println!("\nInstalling: {}", module_name);
        let mut found = false;

        for repo in &repos_to_search {
            let registry_url = format!(
                "https://raw.githubusercontent.com/{}/{}/main/registry.json",
                repo.user, repo.repo
            );

            let module_url = format!(
                "https://raw.githubusercontent.com/{}/{}/main/{}/manifest.xml",
                repo.user, repo.repo, module_name
            );

            match ureq::get(&registry_url).call() {
                Ok(response) => {
                    if let Ok(body) = response.into_string() {
                        if let Ok(registry) = serde_json::from_str::<RegistryJson>(&body) {
                            if registry.modules.iter().any(|m| m.id.to_lowercase() == *module_name) {
                                match ureq::get(&module_url).call() {
                                    Ok(xml_response) => {
                                        if let Ok(manifest_xml) = xml_response.into_string() {
                                            let target_dir = modules_dir.join(module_name);
                                            if target_dir.exists() {
                                                println!("  Module already exists, skipping");
                                                found = true;
                                            } else {
                                                match fs::create_dir_all(&target_dir) {
                                                    Ok(_) => {
                                                        let manifest_path = target_dir.join("manifest.xml");
                                                        if fs::write(&manifest_path, manifest_xml).is_ok() {
                                                            println!("  Installed from {}/{}", repo.user, repo.repo);
                                                            found = true;
                                                            installed += 1;
                                                        } else {
                                                            println!("  Error: Could not write manifest");
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("  Error: Could not create module directory: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }

            if found {
                break;
            }
        }

        if !found {
            println!("  Error: Module '{}' not found in any repo", module_name);
            failed.push(module_name.clone());
        }
    }

    println!("\nInstalled {}/{} module(s)", installed, module_names.len());
    if !failed.is_empty() {
        println!("Failed: {}", failed.join(", "));
    }

    if installed > 0 {
        let _ = crate::modules::ModuleManager::_write_aliases_to_file(modules_dir, &config_dir.join("aliases.sh"));
        let _ = crate::commands::update::execute(modules_dir, &config_dir.join("registry.json"));
    }

    if failed.is_empty() {
        0
    } else {
        1
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
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
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("list-repos");
    let module_dir = config_dir.join("modules");
    let repos_file = config_dir.join("repos.json");
    let remaining_args = args[1..].to_vec();

    match subcommand {
        "add-repo" => add_repo(&repos_file, &remaining_args),
        "list-repos" => list_repos(&repos_file),
        "search-mods" => search_modules(&repos_file, &remaining_args),
        "install-mods" => install_modules(&repos_file, &module_dir, config_dir, &remaining_args),
        "add-mod" => add_mod(&repos_file, &module_dir, config_dir, &remaining_args),
        _ => {
            println!("Unknown subcommand: {}", subcommand);
            println!("Usage:");
            println!("  aktools add-repo <user/repo>   Add a repo to track");
            println!("  aktools list-repos             List configured repos");
            println!("  aktools search-mods <term>     Search modules");
            println!("  aktools install-mods <mod> [<mod>...]  Install module(s)");
            println!("  aktools add-mod <module>       Submit module to community repo");
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

fn add_mod(_repos_file: &Path, modules_dir: &Path, _config_dir: &Path, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Usage: aktools add-mod <module-name>");
        println!("Submit a local module to the community repo for review.");
        println!("This will fork the repo, add your module, and create a pull request.");
        return 1;
    }

    let module_name = &args[0];
    let module_path = modules_dir.join(module_name);

    if !module_path.exists() {
        println!("Error: Module '{}' not found in ~/.aktools/modules/", module_name);
        return 1;
    }

    let manifest_path = module_path.join("manifest.xml");
    if !manifest_path.exists() {
        println!("Error: Module '{}' has no manifest.xml", module_name);
        return 1;
    }

    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Error reading manifest: {}", e);
            return 1;
        }
    };

    let gh_token = std::env::var("GH_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .unwrap_or_default();

    if gh_token.is_empty() {
        println!("Error: GH_TOKEN or GITHUB_TOKEN environment variable not set.");
        println!("Create a token at: https://github.com/settings/tokens");
        println!("Required scope: repo");
        return 1;
    }

    println!("Submitting '{}' to community repo...", module_name);

    let repo_owner = "Akinus21";
    let repo_name = "aktools-modules";
    let api_base = "https://api.github.com";

    let client = ureq::Agent::new();

    let fork_url = format!("{}/repos/{}/{}/forks", api_base, repo_owner, repo_name);
    println!("Forking repository...");

    let fork_response = client.post(&fork_url)
        .set("Authorization", &format!("Bearer {}", gh_token))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .send_string("{}");

    let fork_full_name = match fork_response {
        Ok(resp) => {
            if resp.status() == 202 || resp.status() == 201 {
                if let Ok(body) = resp.into_string() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        json.get("full_name").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if resp.status() == 202 {
                let body = resp.into_string().unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    json.get("full_name").and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                eprintln!("Error: Could not fork repo (status {})", resp.status());
                None
            }
        }
        Err(e) => {
            eprintln!("Error forking repository: {}", e);
            None
        }
    };

    let fork_full_name = match fork_full_name {
        Some(name) => name,
        None => {
            let existing_fork_name = format!("{}/{}", repo_owner, repo_name);
            println!("Using existing fork or rate limited. Trying: {}", existing_fork_name);
            existing_fork_name
        }
    };

    let user_login = match client.get(&format!("{}/repos/{}/{}/", api_base, repo_owner, repo_name))
        .set("Authorization", &format!("Bearer {}", gh_token))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()
    {
        Ok(resp) => {
            if let Ok(body) = resp.into_string() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    json.get("owner").and_then(|o| o.get("login")).and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let actual_fork_name = fork_full_name.as_str();

    println!("Creating module files in your fork...");

    let files: Vec<PathBuf> = collect_files_recursive(&module_path, &module_path)
        .into_iter()
        .filter(|p| {
            p.ends_with("manifest.xml") || p.ends_with(".sh") || p.ends_with(".bash") || p.ends_with(".py") || p.ends_with(".pl") || p.ends_with(".rb")
        })
        .collect();

    for file_path in &files {
        let relative_path = file_path.strip_prefix(&module_path).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| file_path.to_string_lossy().to_string());
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path.display(), e);
                continue;
            }
        };

        let encoded_content = base64_encode(&content);
        let file_path_api = format!("{}/repos/{}/contents/{}", api_base, actual_fork_name, format!("{}/{}", module_name, relative_path));

        let file_body = serde_json::json!({
            "message": format!("Add {} file from aktools add-mod", relative_path),
            "content": encoded_content
        });

        let file_response = client.put(&file_path_api)
            .set("Authorization", &format!("Bearer {}", gh_token))
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Content-Type", "application/json")
            .send_string(&serde_json::to_string(&file_body).unwrap());

        match file_response {
            Ok(resp) => {
                if resp.status() == 201 {
                    println!("  Added: {}", relative_path);
                } else if resp.status() == 200 {
                    println!("  Updated: {}", relative_path);
                } else {
                    let err_body = resp.into_string().unwrap_or_default();
                    eprintln!("  Warning: {} returned status {}: {}", relative_path, resp.status(), err_body);
                }
            }
            Err(e) => {
                eprintln!("  Error adding {}: {}", relative_path, e);
            }
        }
    }

    let registry_url = format!("{}/repos/{}/contents/registry.json", api_base, actual_fork_name);

    let registry_sha = match client.get(&registry_url)
        .set("Authorization", &format!("Bearer {}", gh_token))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()
    {
        Ok(resp) => {
            if resp.status() == 200 {
                if let Ok(body) = resp.into_string() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        json.get("sha").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let base_registry: serde_json::Value = serde_json::from_str(r#"{"version":1,"modules":[]}"#).unwrap();
    let updated_registry = if let Some(sha) = registry_sha {
        let current_registry: serde_json::Value = serde_json::from_str(r#"{"version":1,"modules":[]}"#).unwrap();

        let new_module = serde_json::json!({
            "id": module_name,
            "name": module_name,
            "version": "1.0.0",
            "author": user_login.as_deref().unwrap_or("unknown"),
            "description": format!("Module submitted via aktools add-mod"),
            "tags": []
        });

        let mut modules = current_registry.get("modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone())
            .unwrap_or_default();

        modules.retain(|m| m.get("id").and_then(|v| v.as_str()) != Some(module_name.as_str()));
        modules.push(new_module);

        serde_json::json!({
            "version": current_registry.get("version").unwrap_or(&serde_json::json!(1)),
            "modules": modules
        })
    } else {
        let new_module = serde_json::json!({
            "id": module_name,
            "name": module_name,
            "version": "1.0.0",
            "author": user_login.as_deref().unwrap_or("unknown"),
            "description": format!("Module submitted via aktools add-mod"),
            "tags": []
        });

        serde_json::json!({
            "version": 1,
            "modules": [new_module]
        })
    };

    let registry_content = serde_json::to_string_pretty(&updated_registry).unwrap();
    let encoded_registry = base64_encode(&registry_content);

    let registry_body = serde_json::json!({
        "message": format!("Update registry.json to add {} module", module_name),
        "content": encoded_registry,
        "sha": registry_sha
    });

    let registry_response = client.put(&registry_url)
        .set("Authorization", &format!("Bearer {}", gh_token))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&registry_body).unwrap());

    match registry_response {
        Ok(resp) => {
            if resp.status() == 200 || resp.status() == 201 {
                println!("Updated registry.json");
            } else {
                println!("Warning: registry.json update returned status {}", resp.status());
            }
        }
        Err(e) => {
            eprintln!("Error updating registry.json: {}", e);
        }
    }

    let pr_url = format!("{}/repos/{}/{}/pulls", api_base, repo_owner, repo_name);
    let head_branch = format!("add-{}", module_name);

    let pr_body = serde_json::json!({
        "title": format!("feat: add {} module", module_name),
        "head": format!("{}:{}", actual_fork_name.split('/').next().unwrap_or(""), head_branch),
        "base": "main",
        "body": format!(
            "## Module: {}\n\n\
            Submitted via `aktools add-mod {}`\n\n\
            ### manifest.xml\n\
            ```xml\n{}\n```",
            module_name, module_name, manifest_content
        )
    });

    println!("Creating pull request...");

    let pr_response = client.post(&pr_url)
        .set("Authorization", &format!("Bearer {}", gh_token))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&pr_body).unwrap());

    match pr_response {
        Ok(resp) => {
            if resp.status() == 201 {
                if let Ok(pr_body_resp) = resp.into_string() {
                    if let Ok(pr) = serde_json::from_str::<serde_json::Value>(&pr_body_resp) {
                        if let Some(html_url) = pr.get("html_url").and_then(|v| v.as_str()) {
                            println!("\nSuccess! Pull request created: {}", html_url);
                            println!("\nWhen the PR is merged, the module will be added to the registry.");
                            return 0;
                        }
                    }
                }
                println!("\nSuccess! Pull request created.");
                0
            } else {
                let err_body = resp.into_string().unwrap_or_default();
                eprintln!("Error: PR creation returned status {}: {}", resp.status(), err_body);
                1
            }
        }
        Err(e) => {
            eprintln!("Error creating pull request: {}", e);
            1
        }
    }
}

fn base64_encode(input: &str) -> String {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    let bytes = input.as_bytes();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        output.push(alphabet[b0 >> 2] as char);
        output.push(alphabet[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            output.push(alphabet[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(alphabet[b2 & 0x3f] as char);
        } else {
            output.push('=');
        }
    }

    output
}

fn collect_files_recursive(dir: &Path, base: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_files_recursive(&path, base));
            } else {
                files.push(path);
            }
        }
    }
    files
}
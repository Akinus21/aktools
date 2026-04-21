use std::io::copy;
use std::path::PathBuf;

const REPO: &str = "Akinus21/aktools";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn execute() -> i32 {
    println!("Checking for updates...");

    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let response = match ureq::get(&url)
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to check for updates: {}", e);
            return 1;
        }
    };

    let body = match response.into_string() {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to read response: {}", e);
            return 1;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(e) => {
            println!("Failed to parse response: {}", e);
            return 1;
        }
    };

    let latest_tag = match json.get("tag_name").and_then(|t| t.as_str()) {
        Some(t) => t.trim_start_matches('v'),
        None => {
            println!("Could not find latest version tag");
            return 1;
        }
    };

    if latest_tag == VERSION {
        println!("You're running the latest version: v{}", VERSION);
        return 0;
    }

    println!("Update available: v{} -> v{}", VERSION, latest_tag);
    println!("Downloading...");

    let os = if cfg!(target_os = "windows") { "windows" } else { "linux" };
    let arch = if cfg!(target_arch = "x86_64") { "x86_64" } else if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" };
    let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };

    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/aktools-v{}-{}-{}.{}",
        REPO, latest_tag, latest_tag, os, arch, ext
    );

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(if cfg!(target_os = "windows") {
        "aktools.exe"
    } else {
        "aktools"
    });

    let mut file = match std::fs::File::create(&temp_file) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to create temp file: {}", e);
            return 1;
        }
    };

    match ureq::get(&download_url).call() {
        Ok(response) => {
            if let Err(e) = copy(&mut response.into_reader(), &mut file) {
                println!("Failed to download: {}", e);
                return 1;
            }
        }
        Err(e) => {
            println!("Failed to download: {}", e);
            println!("URL: {}", download_url);
            return 1;
        }
    }

    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&temp_file).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&temp_file, perms);
        }
    }

    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("aktools"));
    let backup_exe = temp_dir.join("aktools.old");

    if current_exe.exists() {
        if let Err(e) = std::fs::rename(&current_exe, &backup_exe) {
            println!("Warning: could not backup current binary: {}", e);
        }
    }

    if let Err(e) = std::fs::rename(&temp_file, &current_exe) {
        println!("Failed to replace binary: {}", e);
        if backup_exe.exists() {
            let _ = std::fs::rename(&backup_exe, &current_exe);
        }
        return 1;
    }

    println!("Successfully updated to v{}", latest_tag);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        println!("Restarting...");
        let _ = std::process::Command::new(&current_exe)
            .args(std::env::args().skip(1))
            .exec();
    }

    #[cfg(not(unix))]
    {
        println!("Please restart aktools to use the new version.");
    }

    0
}
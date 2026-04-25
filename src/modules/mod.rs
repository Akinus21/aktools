use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct OptionSwitch {
    pub flags: Vec<String>,
    pub _is_default: bool,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleManifest {
    pub name: String,
    pub aliases: Vec<String>,
    pub executable: String,
    pub options: Vec<OptionSwitch>,
}

pub struct ModuleManager;

impl ModuleManager {
    pub fn load_manifest(module_path: &Path) -> std::io::Result<ModuleManifest> {
        let manifest_path = module_path.join("manifest.xml");
        if !manifest_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("manifest.xml not found in {:?}", module_path),
            ));
        }
        let content = fs::read_to_string(&manifest_path)?;
        Self::parse_manifest(&content)
    }

    fn parse_manifest(content: &str) -> std::io::Result<ModuleManifest> {
        let mut name = String::new();
        let mut aliases = Vec::new();
        let mut executable = String::new();
        let mut options = Vec::new();
        let mut current_option: Option<OptionSwitch> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("<!--") || line.is_empty() {
                continue;
            }

            if line.starts_with("<name>") && line.ends_with("</name>") {
                name = line.trim_start_matches("<name>").trim_end_matches("</name>").to_string();
            } else if line.starts_with("<alias>") && line.ends_with("</alias>") {
                let alias = line.trim_start_matches("<alias>").trim_end_matches("</alias>").to_string();
                aliases.push(alias);
            } else if line.starts_with("<executable>") && line.ends_with("</executable>") {
                executable = line.trim_start_matches("<executable>").trim_end_matches("</executable>").to_string();
            } else if line.starts_with("<option") {
                current_option = Some(OptionSwitch {
                    flags: Vec::new(),
                    _is_default: line.contains('*'),
                    commands: Vec::new(),
                });
            } else if line.starts_with("</option>") {
                if let Some(opt) = current_option.take() {
                    options.push(opt);
                }
            } else if line.starts_with("<flag") && current_option.is_some() {
                if let Some(ref mut opt) = current_option {
                    let flag = line.trim_start_matches("<flag>").trim_end_matches("</flag>").to_string();
                    opt.flags.push(flag);
                }
            } else if line.starts_with("<command>") && line.ends_with("</command>") && current_option.is_some() {
                if let Some(ref mut opt) = current_option {
                    let cmd = line.trim_start_matches("<command>").trim_end_matches("</command>").to_string();
                    opt.commands.push(cmd);
                }
            }
        }

        Ok(ModuleManifest {
            name,
            aliases,
            executable,
            options,
        })
    }

    pub fn scan_modules(modules_dir: &Path) -> std::io::Result<HashMap<String, ModuleManifest>> {
        let mut modules = HashMap::new();
        if !modules_dir.exists() {
            return Ok(modules);
        }

        for entry in fs::read_dir(modules_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(manifest) = Self::load_manifest(&path) {
                    modules.insert(manifest.name.clone(), manifest);
                }
            }
        }

        Ok(modules)
    }

    pub fn create_module_folder(
        modules_dir: &Path,
        name: &str,
        aliases: &[String],
        source_file: &Path,
    ) -> std::io::Result<std::path::PathBuf> {
        let module_dir = modules_dir.join(name);
        fs::create_dir_all(&module_dir)?;

        let file_name = source_file.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let dest_file = module_dir.join(&file_name);
        fs::copy(source_file, &dest_file)?;

        let manifest = format!(
            r#"<?xml version="1.0"?>
<module>
    <name>{}</name>
    <alias>{}</alias>
    <executable>./{}</executable>
    <option>
        <flag></flag>
        <command></command>
    </option>
</module>"#,
            name,
            aliases.join("</alias>\n    <alias>"),
            file_name
        );

        fs::write(module_dir.join("manifest.xml"), manifest)?;
        Ok(module_dir)
    }

    pub fn _write_aliases_to_file(modules_dir: &Path, shell_file: &Path) -> std::io::Result<()> {
        let modules = Self::scan_modules(modules_dir)?;
        let mut content = String::new();

        content.push_str("# aktools module aliases - auto-generated\n");
        content.push_str("# Do not edit manually\n\n");

        for (_, manifest) in &modules {
            if !manifest.executable.is_empty() && manifest.aliases.len() > 1 {
                continue;
            }
            for alias in &manifest.aliases {
                if !manifest.executable.is_empty() {
                    content.push_str(&format!("alias {}='aktools run {}'\n",
                        alias, manifest.name));
                } else if let Some(opt) = manifest.options.first() {
                    if let Some(flag) = opt.flags.first() {
                        let clean_flag = flag.trim_start_matches('*');
                        content.push_str(&format!("alias {}='aktools run {} {}'\n",
                            alias, manifest.name, clean_flag));
                    }
                }
            }
        }

        if let Some(parent) = shell_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(shell_file, content)
    }

    pub fn has_shell_operators(commands: &[String]) -> bool {
        commands.iter().any(|cmd| {
            cmd.contains("&&") || cmd.contains("||") || cmd.contains(";") || 
            cmd.starts_with("sudo") || cmd.contains(" &") || cmd.ends_with(" &") || 
            cmd.trim_end().ends_with("&")
        })
    }

    pub fn generate_shell_script(module_path: &Path, commands: &[String]) -> std::io::Result<()> {
        let script_path = module_path.join("commands.sh");
        let mut content = String::from("#!/bin/bash\nset -e\n\n");
        
        for cmd in commands {
            content.push_str(cmd);
            content.push('\n');
        }
        
        fs::write(&script_path, content)?;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }
}
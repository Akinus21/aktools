use std::io::Write;
use std::path::Path;
use crate::modules::ModuleManager;

pub fn execute(modules_dir: &Path, _registry_path: &Path, module_name: Option<String>) -> i32 {
    let modules = match ModuleManager::scan_modules(modules_dir) {
        Ok(m) => m,
        Err(e) => {
            println!("Error scanning modules: {}", e);
            return 1;
        }
    };

    let selected_name = if let Some(name) = module_name {
        if !modules.contains_key(&name) {
            println!("Error: module '{}' not found", name);
            return 1;
        }
        name
    } else {
        loop {
            println!("\nInstalled modules:");
            let mut names: Vec<_> = modules.keys().collect();
            names.sort();
            for (i, name) in names.iter().enumerate() {
                println!("  {} - {}", i + 1, name);
            }
            println!("  q - quit");

            print!("\nSelect module to edit: ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Error reading input");
                continue;
            }

            let input = input.trim();
            if input == "q" {
                return 0;
            }

            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= names.len() {
                    break names[idx - 1].clone();
                }
            }
            println!("Invalid selection");
        }
    };

    loop {
        let manifest = modules.get(&selected_name).expect("Module not found");

        println!("\nEditing module: {}", manifest.name);
        println!("1. Name: {}", manifest.name);
        println!("2. Aliases: {:?}", manifest.aliases);
        println!("3. Options: {} option(s)", manifest.options.len());
        println!("q - quit editing");

        print!("\nSelect field to edit (1-3) or 'q' to quit: ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input");
            continue;
        }

        let input = input.trim();
        if input == "q" {
            return 0;
        }

        match input {
            "1" => {
                print!("New name: ");
                std::io::stdout().flush().unwrap();
                let mut new_name = String::new();
                if std::io::stdin().read_line(&mut new_name).is_err() {
                    println!("Error reading input");
                    continue;
                }
                let new_name = new_name.trim().to_string();
                if !new_name.is_empty() {
                    let module_dir = modules_dir.join(&manifest.name);
                    let new_module_dir = modules_dir.join(&new_name);
                    if module_dir.exists() {
                        if let Err(e) = std::fs::rename(&module_dir, &new_module_dir) {
                            println!("Error renaming module: {}", e);
                            continue;
                        }
                    }
                    println!("Name updated to: {}", new_name);
                }
            }
            "2" => {
                print!("New aliases (comma-separated): ");
                std::io::stdout().flush().unwrap();
                let mut new_aliases = String::new();
                if std::io::stdin().read_line(&mut new_aliases).is_err() {
                    println!("Error reading input");
                    continue;
                }
                let new_aliases: Vec<String> = new_aliases
                    .trim()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                println!("Aliases updated to: {:?}", new_aliases);

                let module_dir = modules_dir.join(&manifest.name);
                let manifest_path = module_dir.join("manifest.xml");
                let content = std::fs::read_to_string(&manifest_path).ok();
                if let Some(mut xml) = content {
                    xml = xml.replace(&manifest.aliases.join("</alias>\n    <alias>"), &new_aliases.join("</alias>\n    <alias>"));
                    if let Err(e) = std::fs::write(&manifest_path, xml) {
                        println!("Error updating manifest: {}", e);
                    }
                }
            }
            "3" => {
                if manifest.options.is_empty() {
                    println!("No options to edit.");
                    continue;
                }
                for (i, opt) in manifest.options.iter().enumerate() {
                    println!("\nOption {}:", i + 1);
                    println!("  Flags: {:?}", opt.flags);
                    println!("  Commands: {} command(s)", opt.commands.len());
                    for (j, cmd) in opt.commands.iter().enumerate() {
                        println!("    {}. {}", j + 1, cmd);
                    }
                }

                print!("\nSelect option to edit (1-{}) or 'q' to quit: ", manifest.options.len());
                std::io::stdout().flush().unwrap();
                let mut input_opt = String::new();
                if std::io::stdin().read_line(&mut input_opt).is_err() {
                    println!("Error reading input");
                    continue;
                }

                let input_opt = input_opt.trim();
                if input_opt == "q" {
                    continue;
                }

                if let Ok(opt_idx) = input_opt.parse::<usize>() {
                    if opt_idx > 0 && opt_idx <= manifest.options.len() {
                        let option = &manifest.options[opt_idx - 1];
                        println!("\nCommands for option {}:", opt_idx);
                        for (i, cmd) in option.commands.iter().enumerate() {
                            println!("  {}: {}", i + 1, cmd);
                        }

                        print!("\nSelect command to edit (1-{}), 'a' to add, 'd' to delete, 'q' to quit: ", option.commands.len());
                        std::io::stdout().flush().unwrap();
                        let mut input_cmd = String::new();
                        if std::io::stdin().read_line(&mut input_cmd).is_err() {
                            println!("Error reading input");
                            continue;
                        }

                        let input_cmd = input_cmd.trim();
                        if input_cmd == "q" {
                            continue;
                        }

                        let module_dir = modules_dir.join(&manifest.name);
                        let manifest_path = module_dir.join("manifest.xml");

                        match input_cmd {
                            "a" => {
                                print!("Enter new command: ");
                                std::io::stdout().flush().unwrap();
                                let mut new_cmd = String::new();
                                if std::io::stdin().read_line(&mut new_cmd).is_err() {
                                    println!("Error reading input");
                                    continue;
                                }
                                let new_cmd = new_cmd.trim().to_string();
                                if !new_cmd.is_empty() {
                                    if let Some(content) = std::fs::read_to_string(&manifest_path).ok() {
                                        if let Some(new_content) = add_command_to_manifest(content, opt_idx - 1) {
                                            if let Err(e) = std::fs::write(&manifest_path, new_content) {
                                                println!("Error updating manifest: {}", e);
                                            } else {
                                                println!("Command added successfully.");
                                            }
                                        }
                                    }
                                }
                            }
                            "d" => {
                                print!("Select command to delete (1-{}): ", option.commands.len());
                                std::io::stdout().flush().unwrap();
                                let mut input_del = String::new();
                                if std::io::stdin().read_line(&mut input_del).is_err() {
                                    println!("Error reading input");
                                    continue;
                                }
                                if let Ok(idx) = input_del.trim().parse::<usize>() {
                                    if idx > 0 && idx <= option.commands.len() {
                                        if let Some(content) = std::fs::read_to_string(&manifest_path).ok() {
                                            if let Some(new_content) = delete_command_from_manifest(content, opt_idx - 1, idx - 1) {
                                                if let Err(e) = std::fs::write(&manifest_path, new_content) {
                                                    println!("Error updating manifest: {}", e);
                                                } else {
                                                    println!("Command deleted successfully.");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                if let Ok(cmd_idx) = input_cmd.parse::<usize>() {
                                    if cmd_idx > 0 && cmd_idx <= option.commands.len() {
                                        print!("Enter new command (old: {}): ", option.commands[cmd_idx - 1]);
                                        std::io::stdout().flush().unwrap();
                                        let mut new_cmd = String::new();
                                        if std::io::stdin().read_line(&mut new_cmd).is_err() {
                                            println!("Error reading input");
                                            continue;
                                        }
                                        let new_cmd = new_cmd.trim().to_string();
                                        if !new_cmd.is_empty() {
                                            if let Some(content) = std::fs::read_to_string(&manifest_path).ok() {
                                                if let Some(new_content) = update_command_in_manifest(content, opt_idx - 1, cmd_idx - 1, &new_cmd) {
                                                    if let Err(e) = std::fs::write(&manifest_path, new_content) {
                                                        println!("Error updating manifest: {}", e);
                                                    } else {
                                                        println!("Command updated successfully.");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                println!("Invalid selection");
            }
        }
    }
}

fn add_command_to_manifest(xml: String, opt_idx: usize) -> Option<String> {
    let mut in_option = false;
    let mut option_count = 0;
    let mut pos = 0;

    for line in xml.lines() {
        if line.trim().starts_with("<option") {
            option_count += 1;
            if option_count == opt_idx + 1 {
                in_option = true;
                pos = xml.lines().position(|l| l == line).unwrap_or(0);
                break;
            }
        } else if line.trim().starts_with("</option>") {
            if in_option {
                break;
            }
        }
    }

    let lines: Vec<&str> = xml.lines().collect();
    let insert_line = if opt_idx < lines.len() {
        lines.iter().position(|&l| l.trim().starts_with("</option>")).unwrap_or(lines.len() - 1)
    } else {
        xml.len() - 1
    };

    let command_tag = "        <command></command>";
    let lines: Vec<String> = xml.lines().map(|l| l.to_string()).collect();
    let mut new_lines: Vec<String> = Vec::new();
    let mut added = false;

    for (i, line) in lines.iter().enumerate() {
        new_lines.push(line.clone());
        if line.trim().starts_with("<flag>") && !added && i > 0 {
            for prev_line in (0..i).rev() {
                if lines[prev_line].trim().starts_with("<flag>") {
                    let command_tag = "        <command></command>";
                    new_lines.push(command_tag.to_string());
                    added = true;
                    break;
                }
            }
        }
    }

    if !added {
        let end_opt = "</option>".to_string();
        if let Some(pos) = new_lines.iter().position(|l| l == &end_opt) {
            new_lines.insert(pos, "        <command></command>".to_string());
        }
    }

    Some(new_lines.join("\n"))
}

fn delete_command_from_manifest(xml: String, opt_idx: usize, cmd_idx: usize) -> Option<String> {
    let lines: Vec<String> = xml.lines().map(|l| l.to_string()).collect();
    let mut in_option = false;
    let mut option_count = 0;
    let mut cmd_count = 0;
    let mut to_delete: Vec<usize> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("<option") {
            in_option = true;
            option_count += 1;
        } else if trimmed.starts_with("</option>") {
            if in_option && option_count == opt_idx + 1 {
                break;
            }
            in_option = false;
        } else if trimmed.starts_with("<command>") && in_option && option_count == opt_idx + 1 {
            if cmd_count == cmd_idx {
                to_delete.push(i);
                if trimmed.ends_with("</command>") {
                    cmd_count += 1;
                }
            } else {
                if trimmed.ends_with("</command>") {
                    cmd_count += 1;
                }
            }
        }
    }

    if to_delete.is_empty() {
        return None;
    }

    let mut new_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !to_delete.contains(&i) {
            new_lines.push(line.clone());
        }
    }

    Some(new_lines.join("\n"))
}

fn update_command_in_manifest(xml: String, opt_idx: usize, cmd_idx: usize, new_cmd: &str) -> Option<String> {
    let lines: Vec<String> = xml.lines().map(|l| l.to_string()).collect();
    let mut in_option = false;
    let mut option_count = 0;
    let mut cmd_count = 0;
    let mut to_update: Vec<usize> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("<option") {
            in_option = true;
            option_count += 1;
        } else if trimmed.starts_with("</option>") {
            if in_option && option_count == opt_idx + 1 {
                break;
            }
            in_option = false;
        } else if trimmed.starts_with("<command>") && in_option && option_count == opt_idx + 1 {
            if cmd_count == cmd_idx {
                to_update.push(i);
                if trimmed.ends_with("</command>") {
                    cmd_count += 1;
                }
            } else {
                if trimmed.ends_with("</command>") {
                    cmd_count += 1;
                }
            }
        }
    }

    if to_update.is_empty() {
        return None;
    }

    let mut new_lines: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if to_update.contains(&i) && !to_update.contains(&(i + 1)) {
            new_lines.push(format!("        <command>{}</command>", new_cmd));
        } else {
            new_lines.push(line.clone());
        }
    }

    Some(new_lines.join("\n"))
}

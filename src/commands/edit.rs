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
        println!("3. Options: {} options", manifest.options.len());
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
                println!("Aliases updated");
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

                print!("\nSelect option to edit (1-{}), 'a' to add, 'd' to delete, 'q' to quit: ", manifest.options.len());
                std::io::stdout().flush().unwrap();
                let mut input_opt = String::new();
                if std::io::stdin().read_line(&mut input_opt).is_err() {
                    println!("Error reading input");
                    continue;
                }

                let input_opt = input_opt.trim();
                if input_opt == "a" {
                    print!("Enter flag name: ");
                    std::io::stdout().flush().unwrap();
                    let mut flag = String::new();
                    if std::io::stdin().read_line(&mut flag).is_err() {
                        println!("Error reading input");
                        continue;
                    }
                    let flag = flag.trim().to_string();
                    if flag.is_empty() {
                        println!("Flag cannot be empty.");
                        continue;
                    }
                    print!("Enter command: ");
                    std::io::stdout().flush().unwrap();
                    let mut command = String::new();
                    if std::io::stdin().read_line(&mut command).is_err() {
                        println!("Error reading input");
                        continue;
                    }
                    let command = command.trim().to_string();
                    if command.is_empty() {
                        println!("Command cannot be empty.");
                        continue;
                    }
                    let module_dir = modules_dir.join(&manifest.name);
                    let manifest_path = module_dir.join("manifest.xml");
                    if let Some(content) = std::fs::read_to_string(&manifest_path).ok() {
                        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                        if let Some(end_idx) = lines.iter().position(|l| l.trim().starts_with("</option>")) {
                            let mut new_lines: Vec<String> = Vec::new();
                            for (i, line) in lines.iter().enumerate() {
                                new_lines.push(line.clone());
                                if i == end_idx {
                                    new_lines.push(format!("    <option>
        <flag>{}</flag>
        <command>{}</command>
    </option>", flag, command));
                                }
                            }
                            if let Err(e) = std::fs::write(&manifest_path, new_lines.join("\n")) {
                                println!("Error updating manifest: {}", e);
                            } else {
                                println!("Option added successfully.");
                            }
                        }
                    }
                    continue;
                } else if input_opt == "d" {
                    if manifest.options.len() > 1 {
                        print!("Select option to delete (1-{}): ", manifest.options.len());
                        std::io::stdout().flush().unwrap();
                        let mut input_del = String::new();
                        if std::io::stdin().read_line(&mut input_del).is_err() {
                            println!("Error reading input");
                            continue;
                        }
                        if let Ok(idx) = input_del.trim().parse::<usize>() {
                            if idx > 0 && idx <= manifest.options.len() {
                                let module_dir = modules_dir.join(&manifest.name);
                                let manifest_path = module_dir.join("manifest.xml");
                                if let Some(content) = std::fs::read_to_string(&manifest_path).ok() {
                                    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                                    let mut skip_start: Option<usize> = None;
                                    let mut skip_end: Option<usize> = None;
                                    let mut option_count = 0;
                                    for (i, line) in lines.iter().enumerate() {
                                        let trimmed = line.trim();
                                        if trimmed.starts_with("<option") {
                                            option_count += 1;
                                            if option_count == idx {
                                                skip_start = Some(i);
                                            }
                                        } else if trimmed.starts_with("</option>") {
                                            if option_count == idx {
                                                skip_end = Some(i);
                                                break;
                                            }
                                        }
                                    }
                                    if let (Some(start), Some(end)) = (skip_start, skip_end) {
                                        let new_lines: Vec<String> = lines.iter().enumerate()
                                            .filter(|(i, _)| *i < start || *i > end)
                                            .map(|(_, l)| l.clone())
                                            .collect();
                                        if let Err(e) = std::fs::write(&manifest_path, new_lines.join("\n")) {
                                            println!("Error updating manifest: {}", e);
                                        } else {
                                            println!("Option deleted successfully.");
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        println!("Cannot delete the last option.");
                    }
                    continue;
                } else if input_opt == "q" {
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
                                        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                                        let mut option_count = 0;
                                        let mut in_option = false;
                                        let mut found_option = false;
                                        for (i, line) in lines.iter().enumerate() {
                                            let trimmed = line.trim();
                                            if trimmed.starts_with("<option") {
                                                option_count += 1;
                                                if option_count == opt_idx + 1 {
                                                    in_option = true;
                                                }
                                            } else if trimmed.starts_with("</option>") {
                                                if in_option && option_count == opt_idx + 1 {
                                                    let mut new_lines: Vec<String> = Vec::new();
                                                    for (idx, l) in lines.iter().enumerate() {
                                                        new_lines.push(l.clone());
                                                        if idx == i {
                                                            new_lines.push(format!("        <command>{}</command>", new_cmd));
                                                        }
                                                    }
                                                    if let Err(e) = std::fs::write(&manifest_path, new_lines.join("\n")) {
                                                        println!("Error updating manifest: {}", e);
                                                    } else {
                                                        println!("Command added successfully.");
                                                    }
                                                    found_option = true;
                                                    break;
                                                }
                                                in_option = false;
                                            }
                                        }
                                        if !found_option {
                                            println!("Error: could not find option to add command.");
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
                                            let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                                            let mut option_count = 0;
                                            let mut in_option = false;
                                            let mut cmd_count = 0;
                                            let mut to_delete: Vec<usize> = Vec::new();
                                            for (i, line) in lines.iter().enumerate() {
                                                let trimmed = line.trim();
                                                if trimmed.starts_with("<option") {
                                                    option_count += 1;
                                                    if option_count == opt_idx + 1 {
                                                        in_option = true;
                                                    }
                                                } else if trimmed.starts_with("</option>") {
                                                    if in_option && option_count == opt_idx + 1 {
                                                        break;
                                                    }
                                                    in_option = false;
                                                } else if trimmed.starts_with("<command>") && in_option && option_count == opt_idx + 1 {
                                                    if cmd_count == idx - 1 {
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
                                            if !to_delete.is_empty() {
                                                let new_lines: Vec<String> = lines.iter().enumerate()
                                                    .filter(|(i, _)| !to_delete.contains(i))
                                                    .map(|(_, l)| l.clone())
                                                    .collect();
                                                if let Err(e) = std::fs::write(&manifest_path, new_lines.join("\n")) {
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
                                                let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                                                let mut option_count = 0;
                                                let mut in_option = false;
                                                let mut cmd_count = 0;
                                                let mut to_update: Vec<usize> = Vec::new();
                                                for (i, line) in lines.iter().enumerate() {
                                                    let trimmed = line.trim();
                                                    if trimmed.starts_with("<option") {
                                                        option_count += 1;
                                                        if option_count == opt_idx + 1 {
                                                            in_option = true;
                                                        }
                                                    } else if trimmed.starts_with("</option>") {
                                                        if in_option && option_count == opt_idx + 1 {
                                                            break;
                                                        }
                                                        in_option = false;
                                                    } else if trimmed.starts_with("<command>") && in_option && option_count == opt_idx + 1 {
                                                        if cmd_count == cmd_idx - 1 {
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
                                                if !to_update.is_empty() {
                                                    let mut new_lines: Vec<String> = Vec::new();
                                                    for (i, line) in lines.iter().enumerate() {
                                                        if to_update.contains(&i) && !to_update.contains(&(i + 1)) {
                                                            new_lines.push(format!("        <command>{}</command>", new_cmd));
                                                        } else {
                                                            new_lines.push(line.clone());
                                                        }
                                                    }
                                                    if let Err(e) = std::fs::write(&manifest_path, new_lines.join("\n")) {
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
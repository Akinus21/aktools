# AKTools

**Modular CLI Tool Runner** — Turn any script into a polished CLI command.

AKTools lets you package scripts as modules with custom aliases, multiple entry points, and centralized management. No more hunting for that one-off script buried in your dotfiles.

## Features

- **Module-based architecture** — Organize scripts as packages with metadata
- **Multiple commands per module** — Define different flags/entry points
- **Custom aliases** — Short names for your modules
- **Auto-fix doctor** — Diagnoses and repairs configuration issues automatically
- **Interactive module creation** — Build command modules with an interactive prompt
- **Homebrew install** — Easy installation via Homebrew
- **Shell completion** — Tab completion for bash, zsh, and fish
- **Community modules** — Install modules from GitHub repos

## Installation

```bash
brew tap Akinus21/homebrew-tap
brew install aktools
```

After installation, add to your shell config (`~/.bashrc` or `~/.zshrc`):

```bash
export AKTOOLS_HOME="$HOME/.aktools"
export PATH="$AKTOOLS_HOME/bin:$PATH"
```

Then run `aktools doctor` to set everything up.

## Quick Start

### Create a command module interactively

```bash
aktools build-command
# Follow the prompts to create a module with custom flags and commands
```

### Add a script as a module

```bash
aktools add myscript.sh
# Follow the prompts for name and aliases
```

### Run a module

```bash
aktools <module-name> [args...]
```

### List installed modules

```bash
aktools list
```

### Diagnose issues

```bash
aktools doctor        # Auto-fix issues
aktools doctor --no-fix  # Show issues without fixing
```

## Module Structure

Modules live in `~/.aktools/modules/`. Each module is a folder containing:

```
~/.aktools/modules/
└── mymodule/
    ├── manifest.xml
    └── script.sh
```

### manifest.xml

```xml
<?xml version="1.0"?>
<module>
    <name>mymodule</name>
    <alias>mm</alias>
    <executable>./script.sh</executable>
    <option>
        <flag>run</flag>
        <command>./script.sh</command>
    </option>
</module>
```

- `name` — Module identifier
- `alias` — Short command to invoke the module
- `executable` — Path to script (empty for command-only modules)
- `flag` — Command-line flag to match
- `command` — Command(s) to execute

### Command-Only Modules

Modules can be command-only without an executable:

```xml
<?xml version="1.0"?>
<module>
    <name>sys</name>
    <alias>sys</alias>
    <executable></executable>
    <option>
        <flag>upgrade</flag>
        <command>sudo bootc upgrade && reboot</command>
    </option>
</module>
```

Run with `aktools sys upgrade`.

## Commands

| Command | Description |
|---------|-------------|
| `aktools build-command` | Create a new command module interactively |
| `aktools add <file>` | Add a script as a new module |
| `aktools edit [name]` | Edit a module's manifest |
| `aktools edit-aliases` | Edit shell aliases interactively |
| `aktools list` | List all installed modules |
| `aktools rm <name>` | Remove a module |
| `aktools update` | Rebuild the module registry |
| `aktools doctor` | Diagnose and fix configuration issues |
| `aktools completion <shell>` | Generate shell completions (bash/zsh/fish) |
| `aktools add-repo <user/repo>` | Add a GitHub repo to track modules from |
| `aktools list-repos` | List configured repos |
| `aktools search-mods <term>` | Search for modules in repos |
| `aktools install-mods <mod> [<mod>...]` | Install one or more modules from repos |
| `aktools add-mod <module>` | Submit a module to the community repo |
| `aktools help` | Show this help message |

## Configuration

- **Config directory**: `~/.aktools/`
- **Modules directory**: `~/.aktools/modules/`
- **Registry file**: `~/.aktools/registry.json`
- **Aliases file**: `~/.aktools/aliases.sh`

## Updating

```bash
brew upgrade aktools
```

## Shell Completions

Enable tab completion for your shell:

```bash
# Bash
aktools completion bash --install

# Zsh
aktools completion zsh --install

# Fish
aktools completion fish --install
```

## Community Modules

Install modules from GitHub repos:

```bash
# Add a repo to track
aktools add-repo username/my-plugins

# List configured repos
aktools list-repos

# Search for modules
aktools search-mods mymodule

# Install one or more modules (space-separated)
aktools install-mods mymodule anothermod yetanothermod

# Submit your module to the community repo
aktools add-mod mymodule
```

The default community repo is `Akinus21/aktools-modules` which is always available.

## License

MIT
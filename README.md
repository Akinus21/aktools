# AKTools

**Modular CLI Tool Runner** — Turn any script into a polished CLI command.

AKTools lets you package scripts as modules with custom aliases, multiple entry points, and centralized management. No more hunting for that one-off script buried in your dotfiles.

## Features

- **Module-based architecture** — Organize scripts as packages with metadata
- **Multiple commands per module** — Define different flags/entry points
- **Custom aliases** — Short names for your modules
- **Auto-fix doctor** — Diagnoses and repairs configuration issues automatically
- **Homebrew install** — Easy installation via `brew install aktools`

## Installation

```bash
brew install aktools
```

After installation, add to your shell config (`~/.bashrc` or `~/.zshrc`):

```bash
export AKTOOLS_HOME="$HOME/.aktools"
export PATH="$AKTOOLS_HOME/bin:$PATH"
```

Then run `aktools doctor` to set everything up.

## Quick Start

### Add a script as a module

```bash
aktools add myscript.sh
# Follow the prompts for name and aliases
```

### Run a module

```bash
aktools <module-name>
aktools <module-name> <flag>
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
└── myscript/
    ├── manifest.xml
    └── run.sh
```

### manifest.xml

```xml
<?xml version="1.0"?>
<module>
    <name>myscript</name>
    <alias>ms</alias>
    <option>
        <flag>*run</flag>
        <command>./run.sh</command>
    </option>
    <option>
        <flag>list</flag>
        <command>./list.sh</command>
    </option>
</module>
```

- `name` — Module identifier
- `alias` — Short command to invoke the module
- `flag` — Prefix with `*` for the default command
- `command` — Script to execute

## Commands

| Command | Description |
|---------|-------------|
| `aktools add <file>` | Add a script as a new module |
| `aktools edit [name]` | Edit a module's manifest |
| `aktools list` | List all installed modules |
| `aktools rm <name>` | Remove a module |
| `aktools update` | Rebuild the module registry |
| `aktools doctor` | Diagnose and fix configuration issues |
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

## License

MIT
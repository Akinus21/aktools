# AKTools Rust Refactor - Agent Instructions

## Overview

This is the Rust rewrite of AKTools - a modular CLI tool runner.

## Building

```bash
cd /home/opencode/aktools-rust
cargo build --release
./target/release/aktools help
```

## Git Push Workflow

Since gh CLI is not authenticated, use SSH directly:

```bash
cd /home/opencode/projects/aktools
git add -A
git commit -m "<description>"
GIT_SSH_COMMAND="ssh -i /config/.ssh/github -o StrictHostKeyChecking=no" git push origin main
```

**IMPORTANT: Always push to GitHub after making and verifying changes.**

## Project Structure

```
aktools-rust/
├── Cargo.toml
└── src/
    ├── main.rs          # Entry point
    ├── registry.rs     # Module registry (JSON)
    ├── modules/
    │   └── mod.rs       # XML manifest parser
    └── commands/
        ├── mod.rs       # Command exports
        ├── add.rs       # Add module from file
        ├── edit.rs      # Edit module manifest
        ├── rm.rs        # Remove module
        ├── update.rs    # Rebuild registry
        ├── doctor.rs    # Diagnose issues
        └── help.rs      # Help text
```

## Module Structure

Modules are stored in `~/.aktools/modules/`:
- Each module is a folder
- Contains `manifest.xml` with metadata
- May contain scripts and resources

## Auto-Update (TODO)

- Check GitHub releases for updates
- Download and replace binary
- Verify HMAC signature before executing
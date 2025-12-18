# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Orbit is a Rust CLI/TUI application for workspace awareness and project organization. It scans directories to discover projects, tracks their metadata, identifies duplicates via fingerprinting, and provides export/snapshot functionality.

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run (defaults to TUI mode)
./target/release/orbit

# Run all tests
cargo test

# Run a single test
cargo test <test_name>

# Formatting check (CI enforced)
cargo fmt --check

# Linting (CI enforced with -D warnings)
cargo clippy --all-targets -- -D warnings
```

## CLI Commands

```bash
orbit                                    # Launch TUI (default)
orbit tui                                # Launch TUI explicitly
orbit census --depth 4 --since YYYY-MM-DD  # Scan workspace
orbit status                             # Show current index status
orbit focus --add <path>                 # Pin a project
orbit focus --remove <path>              # Unpin a project
orbit focus --list                       # List pinned projects
orbit snap --label <name>                # Create snapshot of pinned projects
orbit export                             # Export to .orbit/exports/
```

## Architecture

```
src/
├── cli.rs          # Clap-based CLI parser and command dispatch
├── main.rs         # Entry point (calls cli::run)
├── lib.rs          # Module exports
├── model/
│   └── project.rs  # ProjectEntry struct and ProjectKind enum
├── scan/
│   ├── census.rs   # Main scanning logic, runs walk and builds index
│   ├── discover.rs # Finds projects by marker files (Cargo.toml, package.json, etc.)
│   ├── fingerprint.rs  # Blake3-based duplicate detection
│   └── artifacts.rs    # Identifies artifact files
├── index/
│   ├── store.rs    # OrbitIndex load/save to .orbit/index.json
│   ├── focus.rs    # Focus (pinned projects) load/save to .orbit/focus.json
│   └── status.rs   # Print status command
├── export/
│   ├── all.rs      # Export to md/json/csv
│   └── md.rs       # Markdown rendering
├── snapshot/
│   └── quick.rs    # Snapshot pinned projects with artifacts
└── tui/
    ├── mod.rs      # TUI event loop (crossterm + ratatui)
    ├── state.rs    # TUI state machine (panels, search, filters)
    └── ui.rs       # TUI rendering
```

### Key Data Flows

1. **Census**: `discover_projects()` → `summarize_project()` → `fingerprint_project()` → `mark_duplicates_by_fingerprint()` → `store::save()`

2. **Duplicate Detection**: Projects with matching Blake3 fingerprints (computed from marker files + directory children) are grouped and marked as `BackupDuplicate` unless pinned or experimental.

3. **TUI State**: Three panels (Home/Projects/Duplicates) with checkbox filters, live search, and keyboard navigation.

### Data Files

All state stored under `.orbit/` in the workspace root:
- `index.json` - Scanned project metadata (OrbitIndex)
- `focus.json` - Pinned project paths (Focus)
- `exports/` - Exported reports (summary.md, index.json, index.csv)
- `snapshots/` - Timestamped snapshot directories

## Testing

Integration tests in `tests/cli_smoke.rs` exercise the full CLI against a temp directory:
- census with `--since` filtering
- focus add/list
- export output verification
- snapshot artifact copying
- status output

Unit tests are inline in modules (`#[cfg(test)]`):
- `scan/census.rs`: relpath handling, duplicate demotion logic
- `snapshot/quick.rs`: sanitize function

## CI

GitHub Actions (`.github/workflows/ci.yml`) runs on push to main/master and PRs:
1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

## Key Dependencies

- `ratatui` + `crossterm`: TUI rendering and terminal handling
- `clap`: CLI argument parsing with derive macros
- `serde` + `serde_json`: JSON serialization for index/focus
- `walkdir`: Directory traversal
- `blake3`: Fast hashing for fingerprints
- `chrono`: Date/time handling with `--since` filtering
- `globset`: Pattern matching for project marker files
- `csv`: CSV export

# Orbit (Rust) — merged UX+ + Census/Snapshot build

This version merges:
- Checkbox cursor (SPACE toggles selected checkbox)
- Live search on indexed projects (`/` to search, type, Enter to apply, Esc to cancel)
- Duplicate similarity panel (groups by fingerprint; shows why it's flagged)

## Build & Run
```bash
cargo build --release
./target/release/orbit
```

## Keys (TUI)
- `Tab` / `Shift+Tab`: switch panels (Home / Projects / Duplicates)
- `↑/↓`: navigate lists or move checkbox cursor (Home)
- `Space`: toggle selected checkbox (Home)
- `Enter`: run primary action (Home = Census refresh, Search = apply)
- `/`: open search prompt (Projects)
- `Esc`: cancel search
- `Backspace`: edit search text
- `f`: pin/unpin selected project (Projects)
- `s`: snapshot pinned projects (quick manifest + artifacts + exports)
- `e`: export md/json/csv to `.orbit/exports`
- `q`: quit

## CLI
- `orbit census --depth 4 --since YYYY-MM-DD`
- `orbit status`
- `orbit focus --add path | --remove path | --list`
- `orbit snap --label mylabel`
- `orbit export`

## Quick guide (non-technical)
Orbit is a small terminal app that helps you understand and organize a folder full of projects.

What it does:
- Scans a folder and builds an index (`.orbit/index.json`)
- Lets you browse projects, search, and “pin” important ones
- Flags likely duplicates (projects with similar fingerprints)
- Exports simple reports to `.orbit/exports`

Typical flow:
- Start Orbit in the folder you care about
- Press `Enter` on Home to run a Census (scan)
- Go to Projects (Tab), use `↑/↓` to move, `/` to search, `f` to pin
- Review duplicates (Tab again)
- Export (`e`) or snapshot pinned projects (`s`)

Where files go:
- `.orbit/index.json`: the latest scan results
- `.orbit/focus.json`: your pinned list
- `.orbit/exports/`: exported summary files (md/json/csv)
- `.orbit/snapshots/`: timestamped snapshots

## Next steps (engineering)
- Add CI so every change runs: `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets`
- Expand test coverage around:
  - Census filtering and `--since` behavior
  - Duplicate fingerprint grouping (and pinned behavior)
  - Snapshot/export outputs
- Improve resilience by removing `unwrap()` in user-input paths (e.g. bad dates)
- Performance pass for large workspaces (optional): skip heavy folders like `node_modules`, `target`, and `.git` during deep walks

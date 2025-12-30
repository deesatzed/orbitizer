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

## Keys (Orbit TUI)
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
- `--dry-run`: preview snapshot/export (and future cleanup) without writing
- Feature flags: `ORBIT_FEATURE_PROGRESS=1` to show progress logs, `ORBIT_FEATURE_DRY_RUN=1` to force dry-run
- `q`: quit

## Integration (Orbit + Mole)
Orbit (Rust) and Mole (Go) now share a unified data layer for a seamless project management experience:
- **Shared Storage (`~/.orbit/`)**:
  - **Pins/Focus**: `~/.orbit/focus.json` (shared project favorites)
  - **Session**: `~/.orbit/session.json` (restores panel, selection, and search state)
  - **Index**: `~/.orbit/index.json` (authoritative project metadata)
- **Instant Startup**: Mole hydrates its overview mode directly from the Orbit index, enabling instant-on performance for large project roots.
- **Headless CI**: `orbit ci --root <path>` generates census reports and shared index/session state for automated environments.
- **Unified UI**: Accessibility (High Contrast) and TUI state are synchronized between tools.
- See `docs/integration_schemas.md` for the shared JSON contracts.

## Mole analyze (TUI) quick keys
When working in the Mole analyzer (`mo analyze`):
- Navigation: `↑/↓` or `j/k`, `Enter` to dive, `b` to go back to overview.
- Search: `/` to search as you type; results counter updates live.
- Large files: `t` toggles large-files view.
- Delete: `⌫`/`Delete` enters confirmation; deletions move to `~/.mole/trash/<timestamp>` and can be undone.
- Undo: `Ctrl+Z` restores the last delete from trash.
- Multi-select: `Space` toggles selection; `x` opens batch palette (Export, Delete, Pin, Unpin).
- Export: `e` opens export modal (JSON/CSV); respects batch selection when present.
- Pins: `p` toggles pin on the current item (requires `MO_FEATURE_PROJECTS=1`).
- High contrast: press `C` to toggle; or start with `MO_HIGH_CONTRAST=1`.
- Quit: `q`.

### Session persistence (Mole analyze)
- When `MO_FEATURE_PROJECTS=1`, the analyzer saves session state to `~/.orbit/session.json` (shared with Orbit).
- Session auto-restores on the next run (expires after 24h). Session is saved on quit and when navigating directories.
- Legacy support for `~/.mole/session.json` is maintained for existing users.

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
- [x] Unify Orbit and Mole data models (Pins, Session, Index).
- [x] Implement Orbit headless `ci` command.
- [x] Optimize Mole startup via Orbit index ingestion.
- [ ] Add automated E2E integration tests between Rust and Go binaries.
- [ ] Implement binary release pipeline (GitHub Actions).
- [ ] Consolidate all legacy `~/.mole` config paths to `~/.orbit`.

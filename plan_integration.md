

Here’s a detailed build plan for integrating Orbit ↔ Mole capabilities.

## 0) Assumptions
- Target OS: macOS (primary), terminal-first UX.
- Languages: Orbit (Rust 2021), Mole (Bash + Go Bubble Tea).
- No breaking changes to existing CLI entry points; additions are opt-in flags/panels.

## 1) Foundation & Discovery
1.1 Align data contracts  
- Define shared schema for “project” metadata (path, size, mtime, language flags, fingerprint, pinned) and “system metrics” snapshot.  
- Decide interchange format (JSON) and file locations (e.g., `.orbit/metrics.json`, `.mole/projects.json`).

1.2 Module boundaries  
- Orbit: add `system` module for metrics intake (Rust).  
- Mole: add `projects` module for project discovery metadata (Go/Bash).

1.3 Safety/whitelist policy  
- Define global protected paths and dry-run semantics for any destructive action.

Deliverables: RFC doc, schemas, module skeletons, feature flags.

## 2) Mole → Orbit enhancements
2.1 Progress UX  
- Add spinner/progress to census, fingerprinting, exports.  
- Use non-blocking tick in TUI; expose CLI `--progress` flag.  
Tests: golden snapshots for progress off, manual for on.

2.2 System health panel  
- Port Mole collectors (CPU/mem/disk/net/battery) to Rust (via `sysinfo`/`gopsutil`-equivalent).  
- New TUI panel “System” with thresholds coloring; optional `--no-metrics`.  
Tests: mocked metrics provider; UI snapshot tests.

2.3 Artifact cleanup upgrade  
- Import Mole’s target list and age rules; add whitelist/dry-run.  
- Integrate into census output (flag risky/large artifacts).  
Tests: tempdir-based cleanup scenarios; whitelist respected.

2.4 Safety + dry-run  
- Add `--dry-run` to snapshot/export/cleanup; print planned actions.  
- Persist whitelist in `.orbit/whitelist.json`; surface in TUI.

2.5 CJK-aware text + adaptive layout  
- Port width calculation and terminal sizing; improve list truncation.  
Tests: unit tests for width calc; TUI snapshots with wide chars.

## 3) Orbit → Mole enhancements
3.1 Project discovery & categorization  
- Port Orbit’s discovery (language heuristics, depth) into Go helper, write JSON cache.  
- Expose `mo analyze --projects` to show categorized projects.  
Tests: fixture trees for detection; depth/skip rules.

3.2 Search & filter in TUI  
- Add search buffer to Mole menus (Bubble Tea); live filtering for analyzer/uninstaller.  
Tests: model update tests for search, pagination + filter interplay.

3.3 Duplicate/fingerprint detection  
- Port Orbit fingerprinting to Go (blake3); group similar dirs in analyzer.  
- Add “show duplicates” action with confidence scoring.  
Tests: fixture dirs with near-duplicates; perf guard rails.

3.4 Rich exports  
- Add JSON/CSV export for analyzer/status results (align with Orbit export schema).  
- CLI flags: `--export json|csv`, default path `~/.cache/mole/exports`.  
Tests: golden exports.

3.5 Pin/focus concepts  
- Allow bookmarking frequently touched dirs/apps; persist in `~/.config/mole/focus.json`.  
- Surface “Pinned” filter in analyzer and purge.  
Tests: focus CRUD; purge respects pins.

## 4) Cross-cutting UX & Config
4.1 Feature flags  
- Env/CLI toggles: `ORBIT_FEATURE_METRICS`, `MO_FEATURE_PROJECTS`, `--experimental`.  
- Default off for new panels/features until hardened.

4.2 Error handling & logging  
- Remove `unwrap()` on user paths in Orbit; structured errors.  
- Mole: ensure bash paths are quoted; add `--debug` parity.

4.3 Accessibility & keymaps  
- Keep existing keybindings; add help overlay updates in both TUIs.  
- Ensure tab order and search escape paths consistent.

## 5) Testing & Quality
- Unit: metrics provider mocks, width calc, discovery rules, fingerprinting.
- Integration: tempdir project trees; cleanup dry-run vs apply; export content.
- TUI snapshots: golden renders for key states (search on/off, progress on/off).
- Performance: benchmark census and duplicate detection on large trees; guard timeouts.
- Safety: whitelist tests, dry-run idempotence, no destructive defaults.

## 6) Rollout & Docs
- Staged flags (experimental → default).  
- Update READMEs and keybindings help.  
- Add “What’s new” sections and migration notes (paths/config).

## 7) Risks & Mitigations
- Perf regression from fingerprinting/metrics: add timeouts and sampling; make opt-in.  
- Destructive ops: whitelists + dry-run + explicit confirms.  
- UX clutter: new panels gated behind flags until validated.  
- Cross-language parity drift: document schemas; add contract tests for JSON interchange.

## 8) Milestone Sequencing (suggested)
- M1: Foundation (schemas, feature flags, module stubs).
- M2: Mole → Orbit UX (progress, safety, CJK); metrics panel optional.
- M3: Orbit → Mole project intelligence (discovery, search, exports).
- M4: Duplicate detection + artifact cleanup parity.
- M5: Pins/bookmarks + docs/QA hardening.
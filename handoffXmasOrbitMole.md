
# Orbitizer (Orbit + Mole) — Engineering Handoff Packet (as of 2025-12-25)

## 1. Executive Snapshot
- **Phase:** Integration mid-flight; Orbit side largely done, Mole analyzer enhancements partially started (search/filter/duplicates/exports/pins not finished).
- **Last stable commit:** TODO: source? (no git metadata captured in-session).
- **Working**
  - Orbit: TUI progress, dry-run, whitelist; feature flags; serde/dry-run/progress tests.
  - Mole: `mo analyze --projects` writes `.mole/projects.json` (fingerprints, pin awareness, gated by `MO_FEATURE_PROJECTS`).
- **Incomplete / in-progress**
  - Mole analyzer Bubble Tea: search/filter UI, duplicate grouping, exports, pin persistence not implemented.
  - Tests for Mole analyzer features pending.
- **Resume in 1 day?**
  - [ ] Confirm latest commit SHA/date.
  - [ ] Wire search state into model/update/view; apply filtering to entries/large files.
  - [ ] Add fingerprint-based duplicate grouping + exports + pins UI.
  - [ ] Add tests (feature flag, search/filter, duplicates/export, pins).
  - [ ] Run full test suite (Rust + Go) and document results.
- **Active owners:** TODO: owner? (TL/PM/DevOps/QA/Security not specified).

## 2. Project Specification (What)
- **Scope/goals:** Integrate Orbit (Rust TUI) and Mole (Go/Bash) for project discovery/export, dry-run safety, whitelist, progress UI, shared schemas, and Mole analyzer enhancements (search/filter, duplicates, exports, pins).
- **Non-goals:** Not specified; assume no production deploy/infra changes.
- **Stakeholders/requirements:** TODO: owner?, source?
- **Success metrics:** Not specified; implied functional parity and tests.

## 3. Implementation State (How)
- **Modules**
  - Orbit (Rust): feature flags, scan progress buffer, snapshot/export dry-run + whitelist, TUI footer status, tests (feature flags, progress, dry-run, serde).
  - Mole (Go): project discovery writer (`--projects`), fingerprints, pin awareness from `~/.config/mole/focus.json`, feature-flag gate.
- **Boundaries/strategy**
  - Shared JSON schema for projects/metrics/whitelist (doc exists in Orbit repo).
  - Feature flags: Orbit `ORBIT_FEATURE_*`; Mole `MO_FEATURE_PROJECTS`.
  - Mole analyzer UI still lacks search/filter + duplicates/export/pins.
- **Dependency graph (high-level)**
  - Mole [main.go](cci:7://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/main.go:0:0-0:0) → Bubble Tea [model](cci:2://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/main.go:81:0-116:1) (entries/largeFiles) → [scanner.go](cci:7://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/scanner.go:0:0-0:0) (concurrent scan) → cache/write; [projects.go](cci:7://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/projects.go:0:0-0:0) for discovery; [view.go](cci:7://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/view.go:0:0-0:0) for rendering.

## 4. Architecture & Environments
- **High-level:** CLI TUIs (no client/server). Orbit Rust binary; Mole Go binary.
- **Environments:** Local only (no staging/prod noted). TODO: clarify.
- **URLs/IaC:** None referenced. TODO.

## 5. Versions & Compatibility
- **Languages:** Rust (version not captured), Go 1.24.6 toolchain (Go module). @ Mole/go.mod:L1–L13 @ main#TODO-SHA
- **Frameworks:** Bubble Tea v1.3.10; Lipgloss v1.1.0. @ Mole/go.mod:L7–L13
- **Containers/Datastores:** None.
- **Compatibility matrix:** Not defined; CLI-only.

## 6. Build & Run Guides
- **Mole**
  - Build: `cd Mole && go build ./cmd/analyze` (Go 1.24.6). TODO: verify module path; no Makefile referenced.
  - Run project discovery: `MO_FEATURE_PROJECTS=1 mo analyze --projects [root]` → writes `[root]/.mole/projects.json`. @ Mole/cmd/analyze/main.go:L123–L163 @ main#TODO-SHA; projects.go:L39–L176
- **Orbit:** Not exercised this session; refer to Cargo build in repo. TODO: record exact commands.
- **Env vars:** `MO_FEATURE_PROJECTS=1` gate for discovery; existing Mole envs (MO_DEBUG, etc.) documented.

## 7. Deployment & Release
- No CI/CD or release flow referenced. TODO: owner?, source?

## 8. Configuration, Secrets & Compliance
- **Config surfaces:** Env vars; focus pins read from `~/.config/mole/focus.json` (no secrets). @ Mole/cmd/analyze/projects.go:L185–L203
- **Secrets:** None handled here; CLI local only. TODO if other components exist.
- **Licenses/SBOM:** Not referenced. TODO.

## 9. Testing, QA & Quality Gates
- **Orbit tests added previously** (feature flags, progress buffer, dry-run, serde). Not rerun.
- **Mole tests present:** [projects_test.go](cci:7://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/projects_test.go:0:0-0:0) for discovery output and feature flag gating. @ Mole/cmd/analyze/projects_test.go:L1–L53
- **Missing:** Tests for search/filter, duplicates, exports, pins; overall test run not executed this session.
- **Instructions:** `cd Mole && go test ./...` (not run). TODO: capture results.

## 10. Observability & Operations
- None (local CLI). TODO.

## 11. Work Stream & Next Steps
Top next steps:
1) Implement Mole analyzer search/filter UI.
   - Owner: TODO
   - Risk: med (UI correctness).
   - How: add `searchMode/searchInput/searchQuery` to model (already added fields); handle key events to edit/apply; filter entries/large files in view using [filterEntries](cci:1://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/search.go:4:0-17:1)/[filterLargeFiles](cci:1://file:///Volumes/WS4TB/messymarvin/orbit_merged_full/Mole/cmd/analyze/search.go:19:0-32:1) (new file).
2) Implement duplicates grouping (fingerprint) + exports.
   - Owner: TODO
   - Risk: med.
   - How: use `Fingerprint` from `.mole/projects.json` or compute per-entry; add export (JSON/CSV) actions; render grouped view.
3) Implement pin persistence in analyzer.
   - Owner: TODO
   - Risk: med.
   - How: load/save focus pins file; toggle pin in UI; reflect in project index/export.
4) Add tests for new Mole analyzer behaviors (search/filter, duplicates, exports, pins; feature flag).
   - Owner: TODO
   - Risk: low/med.
   - How: go test with temp dirs; ensure gating on `MO_FEATURE_PROJECTS` where applicable.
5) Run full test suites (Orbit Rust + Mole Go); document results.
   - Owner: TODO
   - Risk: low.

Blocked items:
- Need design for Bubble Tea keybindings and UI for search/duplicates/pins (not specified). TODO: decisions.

Risk matrix:
- Missing UI features → medium likelihood/impact; mitigated by focused Bubble Tea changes.
- No automated tests for new features → high likelihood of regressions; mitigated by adding go tests.

Decision log/ADR: None referenced. TODO.

## 12. Appendices
- **API surfaces:** CLI only; key command: `mo analyze --projects` (flag gated).
- **Data model:** Project index fields: path, kind, pinned, latest_mtime, size_bytes, artifact_count, has_git/rust/node/python, fingerprint. @ Mole/cmd/analyze/projects.go:L14–L165
- **Migrations:** None.
- **Glossary:** Orbit (Rust TUI), Mole (Go CLI/TUI); focus pins (`~/.config/mole/focus.json`).

---

### Embedded Checklists

**1-Day Resume**
- [ ] Confirm last stable commit SHA/date.
- [ ] Wire search/filter to model/update/view; apply filtering in view.
- [ ] Implement duplicates grouping/export and pin persistence.
- [ ] Add go tests for analyzer features; run `go test ./...`.
- [ ] Run Orbit tests (`cargo test`); record results.

**Pre-Deploy Gate**
- [ ] All tests green (Rust + Go).
- [ ] Feature flags default safe (projects discovery gated).
- [ ] Docs updated (Mole README already notes `--projects`/flag).
- [ ] No secrets committed.

**Local Dev Sanity**
- [ ] Go toolchain 1.24.6 installed.
- [ ] Build `go build ./cmd/analyze` succeeds.
- [ ] `MO_FEATURE_PROJECTS=1 mo analyze --projects ~` generates `.mole/projects.json`.
- [ ] Orbit: `cargo test` passes. TODO: verify.

**Secrets & Compliance**
- [ ] No secrets required for local analyzer.
- [ ] Focus pins file path: `~/.config/mole/focus.json` (non-secret).
- [ ] If adding remote services later, define vault paths and rotation policy. TODO.

---

## Machine-Readable Summary
```json
{
  "project_name": "Orbitizer (Orbit + Mole)",
  "pause_date_iso": "2025-12-25",
  "repo_roots": ["/Volumes/WS4TB/messymarvin/orbit_merged_full"],
  "default_branch": "main",
  "last_stable_commit": {"sha": "TODO", "date": "TODO", "message": "TODO"},
  "modules": [
    {"name": "Orbit", "path": "src/", "owner": "TODO", "status": "green", "notes": "TUI progress/dry-run/whitelist with tests"},
    {"name": "Mole analyzer", "path": "Mole/cmd/analyze", "owner": "TODO", "status": "yellow", "notes": "Project discovery done; search/duplicates/export/pins not finished"},
    {"name": "Mole project discovery", "path": "Mole/cmd/analyze/projects.go", "owner": "TODO", "status": "green", "notes": "Writes .mole/projects.json; fingerprint/pin aware; gated by MO_FEATURE_PROJECTS"}
  ],
  "versions": {
    "languages": [{"name": "Go", "version": "1.24.6"}],
    "frameworks": [
      {"name": "Bubble Tea", "version": "1.3.10"},
      {"name": "Lipgloss", "version": "1.1.0"}
    ],
    "containers": [],
    "datastores": []
  },
  "ci_cd": {
    "pipelines": [],
    "release_flow": "TODO"
  },
  "environments": [],
  "observability": {
    "dashboards": [],
    "alerts": []
  },
  "backlog_next_steps": [
    {"title": "Implement analyzer search/filter", "owner": "TODO", "eta_days": 1, "risk": "med"},
    {"title": "Duplicates grouping + export", "owner": "TODO", "eta_days": 2, "risk": "med"},
    {"title": "Pin persistence in analyzer", "owner": "TODO", "eta_days": 2, "risk": "med"},
    {"title": "Add analyzer tests", "owner": "TODO", "eta_days": 1, "risk": "low"}
  ],
  "risks": [
    {"risk": "Analyzer UI features incomplete", "likelihood": "med", "impact": "med", "mitigation": "Implement search/duplicates/export/pins with tests"},
    {"risk": "Lack of tests for new UI", "likelihood": "high", "impact": "med", "mitigation": "Add go tests for search/filter, duplicates, export, pins"}
  ],
  "todos": [
    {"item": "Record last stable commit SHA/date", "owner": "TODO", "due": null},
    {"item": "Define owners (TL/PM/DevOps/QA/Security)", "owner": "TODO", "due": null},
    {"item": "Run go test ./... and cargo test", "owner": "TODO", "due": null},
    {"item": "Implement analyzer search/filter + duplicates/export/pins", "owner": "TODO", "due": null}
  ]
}
```
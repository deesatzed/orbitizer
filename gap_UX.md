# Harder UX Review: User Experience & Interface Deep Dive

## 1. User Experience & Interface Deep Dive

### Current Interaction Model
- **Two separate CLIs** with different keybindings, mental models, and terminology. Users must remember which tool does what and how to drive each.
- **Feature flags as environment variables** forces users out of the app and into shell docs—high cognitive load.
- **No progressive disclosure**: All power-user features are either invisible or hidden behind flags.

### Core UX Frictions

#### Discoverability & Learnability
- **No in-app help**: Users see a blank TUI. No hints about ‘?’ for help, no status line explaining available actions.
- **Hidden features**: `mo analyze --projects` and search/duplicates/exports/pins are invisible unless you read docs.
- **Inconsistent entry points**: Orbit and Mole have overlapping purposes (disk/project analysis) but no unified launcher or cross-reference.

#### Feedback & Affordances
- **Silent failures**: Missing feature flag prints nothing or a generic status; users don’t know why nothing happens.
- **No visual cues for state**: No indicator when search mode is active, no badge for “pinned,” no highlight for duplicates.
- **Progress opacity**: Spinners rotate but no ETA or completion percentage; users can’t decide whether to wait or abort.

#### Efficiency & Flow
- **Mode fragmentation**: Users must remember separate modes for large files, search, duplicates, exports—each with different keys.
- **No multi-action workflows**: Can’t search → select duplicates → export in one sequence; must exit/re-enter modes.
- **Repetitive navigation**: No bookmarks or jump-to-project; deep directory re-entry is tedious.

#### Accessibility & Inclusivity
- **Color-only encoding**: Status and size use color without symbols/texture; inaccessible for colorblind users.
- **Small fonts and dense lists**: No adjustable text size; long paths truncate without tooltip or ellipsis policy.
- **Keyboard-only reliance**: No mouse support even where it would speed workflows (e.g., clicking to pin/export).

### Interface Design Gaps

#### Visual Hierarchy
- **Flat lists**: No grouping or nesting; duplicates and pins are visually identical to regular items.
- **No summary dashboard**: Users can’t see at a glance how many duplicates, total pinned, or export readiness.
- **Inconsistent icons**: Mix of emoji and text; no legend.

#### Interaction Patterns
- **No context menus**: Advanced actions require memorizing global keys instead of contextual menus.
- **No preview pane**: Selecting an item doesn’t show metadata (fingerprint, size breakdown, pin status) in a sidebar.
- **No undo**: Destructive actions (delete, unpin) are irreversible.

#### Workflow Support
- **No session persistence**: Restarting loses selections, search state, and view position.
- **No batch operations UI**: Multi-select exists but no bulk action palette.
- **No export customization**: Users can’t choose columns or sort order for CSV/JSON.

## 2. Refined UX Gap Plan

### Immediate (1–2 days): Reduce Friction, Add Signposts
- **Help bar**: Persistent footer line: “Press ? for help | Projects mode: MO_FEATURE_PROJECTS=1 | Search: /”
- **Mode indicators**: Badge in header showing current mode (BROWSE/SEARCH/DUPLICATES/PINS) with active filter count.
- **Feature flag banner**: If flag missing, show a one-time banner: “Enable projects mode: export MO_FEATURE_PROJECTS=1”.
- **Keyboard hints**: Show keybinding hints next to actions when idle for 2 seconds (e.g., “/ to search, e to export”).

### Short term (1 week): Unified, Contextual UI
- **Unified launcher**: `mo` command presents a menu: “(o) Orbit census | (a) Analyze disk | (p) Project discovery”.
- **Contextual sidebar**: When an item is selected, show a panel with path, fingerprint, size, pin toggle, and actions (export, delete, open).
- **Search-as-you-type**: Activate with ‘/’; show live result count and highlight matches in list.
- **Duplicate groups**: Collapse duplicates under a header showing count and total size; expand to show members with pin/export per-item.
- **Export dialog**: Modal to pick format (JSON/CSV), columns, and destination; show preview before write.

### Medium term (2–4 weeks): Workflow & Polish
- **Session persistence**: Save/restore search, selection, and mode in ~/.mole/session.json.
- **Batch action palette**: After multi-select, press ‘x’ to open a small palette with options: export, delete, pin, unpin.
- **Undo stack**: Ctrl+z to undo last destructive action; show notification in status bar.
- **Customizable keybindings**: ~/.mole/keys.yaml to remap actions; include ‘?’ to show current map.
- **Accessibility mode**: Toggle high-contrast, add symbols before colors, adjustable font size.

### Long term (1–2 months): Dashboard & Integration
- **Summary dashboard**: First screen shows cards: total projects, duplicates, pinned, recent exports, quick actions.
- **Plugin architecture**: Allow users to add custom duplicate detectors or export formats.
- **Integration bridge**: Orbit can read Mole’s .mole/projects.json and vice versa; cross-link actions between tools.

## 3. Interface Design Sketches (Textual)

### Header Layout
```
Orbitizer  [BROWSE]  Filter: 12/145  Projects: ON
-------------------------------------------------
? Help  q Quit  / Search  d Duplicates  e Export  p Pins
```

### Item Line with Indicators
```
● myproject/  [Rust]  12.3 MB  Pinned  Duplicate(3)  fp:a1b2...
```

### Context Sidebar (right 30 cols)
```
> myproject/
  Path: ~/dev/myproject
  Fingerprint: a1b2...
  Size: 12.3 MB (842 files)
  Languages: Rust
  [★] Pinned
  Actions:
    [E]xport  [O]pen  [D]elete  [U]npin
```

### Search Mode
```
/ mypr
Results: 3/145
● myproject/  [Rust]  12.3 MB
● myproject2/ [Node]  5.1 MB
● oldproject/ [Go]   8.4 MB
```

### Duplicates Grouped
```
▼ Duplicates (3 groups, 45.2 MB)
  ▶ Group a1b2... (2 items, 22.1 MB)
    ● projA/  12.3 MB  [Pinned]
    ● projB/   9.8 MB
  ▶ Group c3d4... (3 items, 23.1 MB)
    ...
```

## 4. Success Metrics (UX)
- **Time to first success**: New user runs `mo`, sees help, enables projects mode, and exports within 2 minutes.
- **Error recovery**: Users can recover from missing flag or empty search without reading docs.
- **Feature usage**: At least 60% of users engage with search/duplicates/pins within first week.
- **Accessibility**: Colorblind users can distinguish states via symbols/text alone.

## 5. Required Owner Roles
- **Product UX Designer**: To wireframe the sidebar, dashboard, and help system.
- **Frontend/TUI Engineer**: To implement Bubble Tea components for sidebar, modals, and session persistence.
- **Accessibility Specialist**: To review color/contrast and keyboard-only flows.

## 6. Risk Mitigations (UX)
- **Complexity creep**: Keep the default BROWSE mode minimal; expose power features only upon request.
- **Mode confusion**: Use persistent badges and clear transitions; avoid hidden modes.
- **Performance lag**: Defer heavy operations (duplicate detection) until explicit user action; show cancellable progress.
use ratatui::{prelude::*, widgets::*};

use super::state::{Checkbox, Panel, State};

fn cb_label(cb: Checkbox) -> &'static str {
    match cb {
        Checkbox::Active => "Active projects",
        Checkbox::Focus => "Focus (pinned)",
        Checkbox::Backups => "Backups / duplicates",
        Checkbox::Artifacts => "Artifacts (.md)",
    }
}

fn draw_status(f: &mut Frame, st: &mut State, area: Rect) {
    let mut status = Vec::new();
    if let Some(last) = st.progress_log.last() {
        status.push(format!("Progress: {}", last));
    } else {
        status.push("Progress: idle".into());
    }
    status.push(format!("Dry-run: {}", if st.dry_run { "on" } else { "off" }));

    let b = Block::default().borders(Borders::ALL);
    let p = Paragraph::new(status.join("   ")).block(b);
    f.render_widget(p, area);
}

pub fn draw(f: &mut Frame, st: &mut State) {
    let area = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(9), // header
                Constraint::Min(0),    // main
                Constraint::Length(2), // footer/status
            ]
            .as_ref(),
        )
        .split(area);

    let header = Block::default()
        .title("🪐 Orbit — SPACE toggle checkbox, / search, f pin, ENTER refresh census, s snapshot, e export, TAB panels, q quit")
        .borders(Borders::ALL);
    let hi = header.inner(layout[0]);
    f.render_widget(header, layout[0]);

    let mut lines: Vec<String> = vec![];
    lines.push(format!("Root: {}", st.root.display()));
    lines.push(format!(
        "Indexed: {}",
        st.index
            .generated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "N/A".into())
    ));
    lines.push(format!(
        "Panel: {:?}   Search: {}",
        st.panel,
        if st.search_mode {
            format!("/{}", st.search_buf)
        } else {
            st.search_query.clone()
        }
    ));
    if st.dry_run {
        lines.push("Mode: DRY-RUN (no writes)".into());
    }
    if !st.progress_log.is_empty() {
        lines.push("Recent progress:".into());
        for line in st.progress_log.iter().rev().take(3).rev() {
            lines.push(format!(" • {line}"));
        }
    }
    lines.push("".into());
    for (i, cb) in st.checkboxes.iter().enumerate() {
        let mark = if st.checked.contains(cb) { "x" } else { " " };
        let cur = if st.panel == Panel::Home && i == st.checkbox_cursor {
            ">"
        } else {
            " "
        };
        lines.push(format!("{} [{}] {}", cur, mark, cb_label(*cb)));
    }

    f.render_widget(Paragraph::new(lines.join("\n")), hi);

    match st.panel {
        Panel::Home => draw_home(f, st, layout[1]),
        Panel::Projects => draw_projects(f, st, layout[1]),
        Panel::Duplicates => draw_dupes(f, st, layout[1]),
    }

    draw_status(f, st, layout[2]);
}

fn draw_home(f: &mut Frame, st: &mut State, area: Rect) {
    let mut text = String::from(
        "ENTER: run Census (depth=4) and refresh index.\nTAB: Projects and Duplicates panels.\n\nTip: Pin your current work with `f` (Projects panel).\n",
    );
    if st.dry_run {
        text.push_str("\nDRY-RUN is ON (no writes).\n");
    }
    if !st.progress_log.is_empty() {
        text.push_str("\nRecent progress:\n");
        for line in st.progress_log.iter().rev().take(5).rev() {
            text.push_str(&format!(" - {line}\n"));
        }
    }
    let b = Block::default().title("Home").borders(Borders::ALL);
    let p = Paragraph::new(text).block(b);
    f.render_widget(p, area);
}

fn draw_projects(f: &mut Frame, st: &mut State, area: Rect) {
    let b = Block::default()
        .title("Projects (filtered) — ↑/↓ select, f pin, / search")
        .borders(Borders::ALL);
    let ps = st.projects_filtered();
    let items: Vec<ListItem> = ps
        .iter()
        .map(|p| {
            let star = if p.pinned { "★" } else { " " };
            let lm = p
                .latest_mtime
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "N/A".into());
            let sz = p
                .size_bytes
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into());
            let line = format!(
                "{} {:<46} {:?}  latest:{}  size:{}  artifacts:{}",
                star, p.path, p.kind, lm, sz, p.artifact_count
            );
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !items.is_empty() {
        let sel = st.selected_project.min(items.len() - 1);
        state.select(Some(sel));
    }
    let list = List::new(items)
        .block(b)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_dupes(f: &mut Frame, st: &mut State, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)].as_ref())
        .split(area);

    // Copy selection indices before borrowing
    let sel_group = st.selected_dupe_group;
    let sel_item = st.selected_dupe_item;

    // Clone the groups data to avoid borrow conflicts
    let groups: Vec<(String, Vec<_>)> = st.duplicate_groups().to_vec();

    let left_block = Block::default()
        .title("Duplicate groups (by fingerprint)")
        .borders(Borders::ALL);
    let right_block = Block::default().title("Why flagged").borders(Borders::ALL);

    let left_items: Vec<ListItem> = groups
        .iter()
        .map(|(fp, v)| {
            let newest = v
                .iter()
                .filter_map(|p| p.latest_mtime)
                .max()
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "N/A".into());
            ListItem::new(format!(
                "{}…  copies:{}  newest:{}",
                &fp[..8.min(fp.len())],
                v.len(),
                newest
            ))
        })
        .collect();

    let mut lstate = ListState::default();
    if !left_items.is_empty() {
        lstate.select(Some(sel_group.min(left_items.len() - 1)));
    }
    let left = List::new(left_items)
        .block(left_block)
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(left, layout[0], &mut lstate);

    // Right panel explanation
    let mut expl = String::new();
    if groups.is_empty() {
        expl.push_str(
            "No duplicate fingerprint groups found yet.\n\nRun Census (ENTER) to refresh index.",
        );
    } else {
        let gidx = sel_group.min(groups.len() - 1);
        let (fp, items) = &groups[gidx];
        let iidx = sel_item.min(items.len() - 1);
        let item = &items[iidx];

        expl.push_str(&format!("Fingerprint: {}\n", fp));
        expl.push_str("\nOrbit marks duplicates when multiple projects share the same lightweight fingerprint.\n");
        expl.push_str("Basis currently includes:\n");
        expl.push_str(" • Blake3 hash of sampled marker files (README/CLAUDE/AGENT/build files)\n");
        expl.push_str(" • Structural hint (immediate child names)\n\n");
        expl.push_str("Selected copy:\n");
        expl.push_str(&format!(
            " • Path: {}\n • Kind: {:?}\n • Pinned: {}\n",
            item.path, item.kind, item.pinned
        ));
        if let Some(lm) = item.latest_mtime {
            expl.push_str(&format!(" • Latest: {}\n", lm.to_rfc3339()));
        }
        expl.push_str("\nCopies in this group:\n");
        for p in items.iter().take(12) {
            expl.push_str(&format!(
                " - {}{}\n",
                if p.pinned { "★ " } else { "" },
                p.path
            ));
        }
        if items.len() > 12 {
            expl.push_str(" - …\n");
        }
        expl.push_str("\nTip: Pin the intended 'real' one with `f` so Orbit won't auto-demote it.");
    }

    let right = Paragraph::new(expl)
        .block(right_block)
        .wrap(Wrap { trim: false });
    f.render_widget(right, layout[1]);
}

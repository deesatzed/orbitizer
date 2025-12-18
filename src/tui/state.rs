use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::index::{focus, store};
use crate::model::project::{sync_pinned_flags, ProjectEntry, ProjectKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Checkbox {
    Active,
    Focus,
    Backups,
    Artifacts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Home,
    Projects,
    Duplicates,
}

pub struct State {
    pub root: PathBuf,
    pub panel: Panel,

    pub checkboxes: Vec<Checkbox>,
    pub checked: HashSet<Checkbox>,
    pub checkbox_cursor: usize,

    pub index: store::OrbitIndex,
    pub focus: focus::Focus,

    pub selected_project: usize,
    pub selected_dupe_group: usize,
    pub selected_dupe_item: usize,

    // search
    pub search_mode: bool,
    pub search_buf: String,
    pub search_query: String,

    // cached duplicate groups (invalidated on index change)
    cached_dupe_groups: Option<Vec<(String, Vec<ProjectEntry>)>>,
    // cached filtered projects (invalidated on index/filter/search change)
    cached_filtered_projects: Option<Vec<ProjectEntry>>,
}

impl State {
    pub fn new(root: PathBuf) -> Result<Self> {
        let index = store::load(&root).unwrap_or_default();
        let focus = focus::load_focus(&root).unwrap_or_default();

        let mut checked = HashSet::new();
        checked.insert(Checkbox::Active);
        checked.insert(Checkbox::Focus);
        checked.insert(Checkbox::Artifacts);

        Ok(Self {
            root,
            panel: Panel::Home,
            checkboxes: vec![
                Checkbox::Active,
                Checkbox::Focus,
                Checkbox::Backups,
                Checkbox::Artifacts,
            ],
            checked,
            checkbox_cursor: 0,
            index,
            focus,
            selected_project: 0,
            selected_dupe_group: 0,
            selected_dupe_item: 0,
            search_mode: false,
            search_buf: String::new(),
            search_query: String::new(),
            cached_dupe_groups: None,
            cached_filtered_projects: None,
        })
    }

    pub fn next_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Home => Panel::Projects,
            Panel::Projects => Panel::Duplicates,
            Panel::Duplicates => Panel::Home,
        };
    }
    pub fn prev_panel(&mut self) {
        self.next_panel();
    }

    pub fn up(&mut self) {
        match self.panel {
            Panel::Home => {
                if self.checkbox_cursor > 0 {
                    self.checkbox_cursor -= 1;
                }
            }
            Panel::Projects => {
                if self.selected_project > 0 {
                    self.selected_project -= 1;
                }
            }
            Panel::Duplicates => {
                if self.selected_dupe_item > 0 {
                    self.selected_dupe_item -= 1;
                } else if self.selected_dupe_group > 0 {
                    self.selected_dupe_group -= 1;
                    self.selected_dupe_item = 0;
                }
            }
        }
    }

    pub fn down(&mut self) {
        match self.panel {
            Panel::Home => {
                if self.checkbox_cursor + 1 < self.checkboxes.len() {
                    self.checkbox_cursor += 1;
                }
            }
            Panel::Projects => {
                let cur = self.selected_project;
                let ps = self.projects_filtered();
                if !ps.is_empty() && cur + 1 < ps.len() {
                    self.selected_project += 1;
                }
            }
            Panel::Duplicates => {
                // Get current selection before borrowing
                let cur_group = self.selected_dupe_group;
                let cur_item = self.selected_dupe_item;
                // Get counts from cached groups
                let (group_count, item_count) = {
                    let groups = self.duplicate_groups();
                    if groups.is_empty() {
                        return;
                    }
                    let g = cur_group.min(groups.len() - 1);
                    (groups.len(), groups[g].1.len())
                };
                if cur_item + 1 < item_count {
                    self.selected_dupe_item += 1;
                } else if cur_group + 1 < group_count {
                    self.selected_dupe_group += 1;
                    self.selected_dupe_item = 0;
                }
            }
        }
    }

    pub fn toggle_checkbox(&mut self) {
        if self.panel != Panel::Home {
            return;
        }
        let cb = self.checkboxes[self.checkbox_cursor];
        if self.checked.contains(&cb) {
            self.checked.remove(&cb);
        } else {
            self.checked.insert(cb);
        }
        self.invalidate_filter_cache();
    }

    pub fn primary_action(&mut self) -> Result<()> {
        // Home ENTER: refresh census
        crate::scan::census::run_census(self.root.to_string_lossy().as_ref(), 4, None, false)?;
        self.index = store::load(&self.root)?;
        self.focus = focus::load_focus(&self.root)?;
        // re-sync pinned flags from focus (single source of truth)
        sync_pinned_flags(&mut self.index.projects, &self.focus.pinned);
        store::save(&self.root, &self.index)?;
        // Invalidate cache after index change
        self.invalidate_cache();
        Ok(())
    }

    /// Invalidate all cached computations (call after index changes)
    fn invalidate_cache(&mut self) {
        self.cached_dupe_groups = None;
        self.cached_filtered_projects = None;
    }

    /// Invalidate only filtered projects cache (call after filter/search changes)
    fn invalidate_filter_cache(&mut self) {
        self.cached_filtered_projects = None;
    }

    pub fn start_search(&mut self) {
        if self.panel != Panel::Projects {
            return;
        }
        self.search_mode = true;
        self.search_buf.clear();
    }
    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_buf.clear();
    }
    pub fn apply_search(&mut self) {
        self.search_mode = false;
        self.search_query = self.search_buf.clone();
        self.search_buf.clear();
        self.selected_project = 0;
        self.invalidate_filter_cache();
    }
    pub fn push_search(&mut self, c: char) {
        // keep it simple; later add cursor position
        if self.search_buf.len() < 120 {
            self.search_buf.push(c);
        }
    }
    pub fn backspace_search(&mut self) {
        self.search_buf.pop();
    }

    pub fn toggle_pin_selected(&mut self) -> Result<()> {
        if self.panel != Panel::Projects {
            return Ok(());
        }
        // Copy selection index before borrowing
        let idx = self.selected_project;
        let sel = {
            let ps = self.projects_filtered();
            if ps.is_empty() {
                return Ok(());
            }
            ps[idx.min(ps.len() - 1)].path.clone()
        };

        if self.focus.pinned.iter().any(|p| p == &sel) {
            self.focus.pinned.retain(|p| p != &sel);
        } else {
            self.focus.pinned.push(sel);
            self.focus.pinned.sort();
        }
        focus::save_focus(&self.root, &self.focus)?;

        // Sync pinned flags from focus (single source of truth)
        sync_pinned_flags(&mut self.index.projects, &self.focus.pinned);
        store::save(&self.root, &self.index)?;
        // Invalidate filter cache since pinned status changed
        self.invalidate_cache();
        Ok(())
    }

    pub fn snapshot(&mut self) -> Result<()> {
        crate::snapshot::quick::snapshot_pinned(self.root.to_string_lossy().as_ref(), Some("tui"))?;
        Ok(())
    }
    pub fn export(&mut self) -> Result<()> {
        crate::export::all::export_all(self.root.to_string_lossy().as_ref())?;
        Ok(())
    }

    /// Returns filtered and sorted projects (cached for performance)
    pub fn projects_filtered(&mut self) -> &[ProjectEntry] {
        if self.cached_filtered_projects.is_none() {
            self.cached_filtered_projects = Some(self.compute_filtered_projects());
        }
        self.cached_filtered_projects.as_ref().unwrap()
    }

    fn compute_filtered_projects(&self) -> Vec<ProjectEntry> {
        let mut ps = self.index.projects.clone();
        // Sync pinned flags from focus (single source of truth)
        sync_pinned_flags(&mut ps, &self.focus.pinned);

        // checkbox filters
        ps.retain(|p| {
            if self.checked.contains(&Checkbox::Focus) && p.pinned {
                return true;
            }
            if self.checked.contains(&Checkbox::Active)
                && matches!(p.kind, ProjectKind::ActiveStandalone)
            {
                return true;
            }
            if self.checked.contains(&Checkbox::Backups)
                && matches!(p.kind, ProjectKind::BackupDuplicate)
            {
                return true;
            }
            if self.checked.contains(&Checkbox::Artifacts) && p.artifact_count > 0 {
                return true;
            }
            false
        });

        // live search filter
        if !self.search_query.trim().is_empty() {
            let q = self.search_query.to_lowercase();
            ps.retain(|p| p.path.to_lowercase().contains(&q));
        }

        ps.sort_by_key(|p| p.latest_mtime);
        ps.reverse();
        ps
    }

    /// Returns groups: (fingerprint, [projects])
    /// Results are cached and invalidated on index change
    pub fn duplicate_groups(&mut self) -> &[(String, Vec<ProjectEntry>)] {
        if self.cached_dupe_groups.is_none() {
            self.cached_dupe_groups = Some(self.compute_duplicate_groups());
        }
        self.cached_dupe_groups.as_ref().unwrap()
    }

    fn compute_duplicate_groups(&self) -> Vec<(String, Vec<ProjectEntry>)> {
        let mut map: HashMap<String, Vec<ProjectEntry>> = HashMap::new();
        for p in &self.index.projects {
            if let Some(fp) = &p.fingerprint {
                map.entry(fp.clone()).or_default().push(p.clone());
            }
        }
        let mut groups: Vec<(String, Vec<ProjectEntry>)> =
            map.into_iter().filter(|(_, v)| v.len() >= 2).collect();
        // sort groups by most recent member
        groups.sort_by_key(|(_, v)| v.iter().filter_map(|p| p.latest_mtime).max());
        groups.reverse();
        for (_, v) in groups.iter_mut() {
            v.sort_by_key(|p| p.latest_mtime);
            v.reverse();
        }
        groups
    }
}

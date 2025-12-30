package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// duplicateGroup represents a set of entries sharing the same fingerprint
type duplicateGroup struct {
	Fingerprint string
	Entries     []dirEntry
	TotalSize   int64
}

// loadProjectIndex loads the .mole/projects.json if present, for fingerprint/duplicate info
func loadProjectIndex(root string) (projectIndex, error) {
	var idx projectIndex
	p := filepath.Join(root, ".mole", "projects.json")
	data, err := os.ReadFile(p)
	if err != nil {
		return idx, err
	}
	err = json.Unmarshal(data, &idx)
	return idx, err
}

// groupDuplicatesByFingerprint groups entries by fingerprint using the project index if available
func (m model) groupDuplicatesByFingerprint() []duplicateGroup {
	if !featureProjectsEnabled() {
		return nil
	}
	// Load project index to get fingerprint mapping
	idx, err := loadProjectIndex(m.path)
	if err != nil {
		return nil
	}
	// Map fingerprint to entries
	fpMap := make(map[string][]dirEntry)
	for _, proj := range idx.Projects {
		// Find matching dirEntry by path (relative)
		for _, entry := range m.entries {
			// Convert entry.Path to relative if needed
			rel, errRel := filepath.Rel(m.path, entry.Path)
			if errRel != nil {
				continue
			}
			if rel == proj.Path && proj.Fingerprint != "" {
				fpMap[proj.Fingerprint] = append(fpMap[proj.Fingerprint], entry)
			}
		}
	}
	var groups []duplicateGroup
	for fp, entries := range fpMap {
		if len(entries) > 1 {
			var total int64
			for _, e := range entries {
				total += e.Size
			}
			groups = append(groups, duplicateGroup{
				Fingerprint: fp,
				Entries:     entries,
				TotalSize:   total,
			})
		}
	}
	return groups
}

// renderDuplicateGroups renders the duplicate groups view
func (m model) renderDuplicateGroups(b *strings.Builder, groups []duplicateGroup) {
	if len(groups) == 0 {
		fmt.Fprintf(b, "%sNo duplicates found.%s\n", colorGray, colorReset)
		return
	}
	fmt.Fprintf(b, "%s▼ Duplicates (%d groups, %s)%s\n", colorCyan, len(groups), humanizeBytes(m.totalSize), colorReset)
	for _, g := range groups {
		// Simple collapsible: always expanded for now
		fmt.Fprintf(b, "%s  ▶ Group %s... (%d items, %s)%s\n", colorGray, g.Fingerprint[:8], len(g.Entries), humanizeBytes(g.TotalSize), colorReset)
		for _, entry := range g.Entries {
			icon := "📁"
			if !entry.IsDir {
				icon = "📄"
			}
			name := entry.Name
			size := humanizeBytes(entry.Size)
			fmt.Fprintf(b, "    %s %s  %s%s\n", icon, name, colorGray, size)
			// TODO: add per-item actions (pin/export) here later
		}
	}
}

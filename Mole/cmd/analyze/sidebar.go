package main

import (
	"fmt"
	"path/filepath"
	"strings"
)

// renderSidebar renders a contextual sidebar for the selected item
func (m model) renderSidebar(b *strings.Builder) {
	if len(m.entries) == 0 {
		return
	}
	entry := m.entries[m.selected]
	// Simple 30-column sidebar on the right
	fmt.Fprintf(b, "%s|", colorGray)
	// Header
	fmt.Fprintf(b, "\n%s> %s%s\n", colorPurpleBold, entry.Name, colorReset)
	fmt.Fprintf(b, "%sPath: %s%s\n", colorGray, displayPath(entry.Path), colorReset)
	// Size and file count
	fmt.Fprintf(b, "%sSize: %s%s\n", colorGray, humanizeBytes(entry.Size), colorReset)
	if entry.IsDir {
		fmt.Fprintf(b, "%sType: Directory%s\n", colorGray, colorReset)
	} else {
		fmt.Fprintf(b, "%sType: File%s\n", colorGray, colorReset)
	}
	// Last access time
	if !entry.LastAccess.IsZero() {
		fmt.Fprintf(b, "%sLast Access: %s%s\n", colorGray, entry.LastAccess.Format("2006-01-02 15:04"), colorReset)
	}
	// Projects mode metadata
	if featureProjectsEnabled() {
		rel, err := filepath.Rel(m.path, entry.Path)
		if err == nil {
			focus := loadFocus()
			if focus.isPinned(rel) {
				fmt.Fprintf(b, "%s[★] Pinned%s\n", colorGreen, colorReset)
			}
			// Try to get fingerprint from project index if available
			if idx, err := loadProjectIndex(m.path); err == nil {
				for _, proj := range idx.Projects {
					if proj.Path == rel && proj.Fingerprint != "" {
						fmt.Fprintf(b, "%sFingerprint: %s...%s\n", colorGray, proj.Fingerprint[:8], colorReset)
						break
					}
				}
			}
		}
	}
	// Actions
	fmt.Fprintf(b, "%s--- Actions ---%s\n", colorGray, colorReset)
	fmt.Fprintf(b, "%s[E]xport  [O]pen%s\n", colorGray, colorReset)
	if entry.IsDir {
		fmt.Fprintf(b, "%s[Enter] Enter%s\n", colorGray, colorReset)
	}
	if featureProjectsEnabled() {
		fmt.Fprintf(b, "%s[P] Pin/Unpin%s\n", colorGray, colorReset)
	}
	fmt.Fprintf(b, "%s[D]elete%s\n", colorGray, colorReset)
}

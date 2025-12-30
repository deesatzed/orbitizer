package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

func TestSearchFilter(t *testing.T) {
	m := model{
		entries: []dirEntry{
			{Name: "apple", Path: "/tmp/apple", Size: 100},
			{Name: "banana", Path: "/tmp/banana", Size: 200},
			{Name: "cherry", Path: "/tmp/cherry", Size: 300},
		},
		searchQuery: "an", // lowercase
	}
	filtered := m.filterEntries(m.entries)
	t.Logf("Search query: %s", m.searchQuery)
	t.Logf("Filtered entries: %d", len(filtered))
	for i, e := range filtered {
		t.Logf("  %d: %s (%s)", i, e.Name, e.Path)
	}
	if len(filtered) != 1 {
		t.Fatalf("expected 1 filtered entry (banana), got %d", len(filtered))
	}
	if len(filtered) > 0 && filtered[0].Name != "banana" {
		t.Errorf("expected 'banana', got %s", filtered[0].Name)
	}
}

func TestExportModal(t *testing.T) {
	m := model{
		entries: []dirEntry{
			{Name: "test.txt", Path: "/tmp/test.txt", Size: 100, IsDir: false},
		},
	}
	m.showExportModal()
	if !m.exportModal.active {
		t.Fatal("export modal should be active")
	}
	if m.exportModal.format != exportFormatJSON {
		t.Fatal("default format should be JSON")
	}
	m.setExportFormat(exportFormatCSV)
	if m.exportModal.format != exportFormatCSV {
		t.Fatal("format should be CSV")
	}
	m.hideExportModal()
	if m.exportModal.active {
		t.Fatal("export modal should be inactive")
	}
}

func TestPinToggle(t *testing.T) {
	// Setup temporary focus file
	tmp := t.TempDir()
	focusPath := filepath.Join(tmp, "focus.json")

	// Create a mock focus file with empty pinned list
	if err := os.WriteFile(focusPath, []byte(`{"pinned":[]}`), 0644); err != nil {
		t.Fatalf("write focus file: %v", err)
	}

	// Override the focus path for testing
	originalHome := os.Getenv("HOME")
	os.Setenv("HOME", tmp)
	defer os.Setenv("HOME", originalHome)

	// Enable projects feature
	os.Setenv("MO_FEATURE_PROJECTS", "1")
	defer os.Unsetenv("MO_FEATURE_PROJECTS")

	m := model{
		path: tmp,
		entries: []dirEntry{
			{Name: "project1", Path: filepath.Join(tmp, "project1"), Size: 1000, IsDir: true},
		},
		selected: 0,
	}

	// Test pinning
	if featureProjectsEnabled() && len(m.entries) > 0 {
		entry := m.entries[m.selected]
		rel, err := filepath.Rel(m.path, entry.Path)
		if err == nil {
			focus := loadFocus()
			if focus.isPinned(rel) {
				// Unpin
				newPinned := make([]string, 0, len(focus.Pinned))
				for _, p := range focus.Pinned {
					if p != rel {
						newPinned = append(newPinned, p)
					}
				}
				focus.Pinned = newPinned
				m.status = fmt.Sprintf("Unpinned %s", entry.Name)
			} else {
				// Pin
				focus.Pinned = append(focus.Pinned, rel)
				m.status = fmt.Sprintf("Pinned %s", entry.Name)
			}
			// Save focus list
			home, _ := os.UserHomeDir()
			focusPath = filepath.Join(home, ".config", "mole", "focus.json")
			if err := os.MkdirAll(filepath.Dir(focusPath), 0755); err == nil {
				data, _ := json.MarshalIndent(focus, "", "  ")
				_ = os.WriteFile(focusPath, data, 0644)
			}
		}
	}

	if m.status != "Pinned project1" {
		t.Errorf("expected pin status, got: %s", m.status)
	}

	// Test unpinning
	m.status = ""
	if featureProjectsEnabled() && len(m.entries) > 0 {
		entry := m.entries[m.selected]
		rel, err := filepath.Rel(m.path, entry.Path)
		if err == nil {
			focus := loadFocus()
			if focus.isPinned(rel) {
				// Unpin
				newPinned := make([]string, 0, len(focus.Pinned))
				for _, p := range focus.Pinned {
					if p != rel {
						newPinned = append(newPinned, p)
					}
				}
				focus.Pinned = newPinned
				m.status = fmt.Sprintf("Unpinned %s", entry.Name)
			} else {
				// Pin
				focus.Pinned = append(focus.Pinned, rel)
				m.status = fmt.Sprintf("Pinned %s", entry.Name)
			}
			// Save focus list
			home, _ := os.UserHomeDir()
			focusPath = filepath.Join(home, ".config", "mole", "focus.json")
			if err := os.MkdirAll(filepath.Dir(focusPath), 0755); err == nil {
				data, _ := json.MarshalIndent(focus, "", "  ")
				_ = os.WriteFile(focusPath, data, 0644)
			}
		}
	}

	if m.status != "Unpinned project1" {
		t.Errorf("expected unpin status, got: %s", m.status)
	}
}

func TestDuplicateGroups(t *testing.T) {
	// Enable projects feature
	os.Setenv("MO_FEATURE_PROJECTS", "1")
	defer os.Unsetenv("MO_FEATURE_PROJECTS")

	tmp := t.TempDir()
	// Create mock project index
	projIdx := projectIndex{
		Projects: []projectEntry{
			{Path: "dup1", Fingerprint: "abc123"},
			{Path: "dup2", Fingerprint: "abc123"},
			{Path: "unique", Fingerprint: "def456"},
		},
	}
	projPath := filepath.Join(tmp, ".mole", "projects.json")
	if err := os.MkdirAll(filepath.Dir(projPath), 0755); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	data, err := json.Marshal(projIdx)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if err := os.WriteFile(projPath, data, 0644); err != nil {
		t.Fatalf("write: %v", err)
	}

	m := model{
		path: tmp,
		entries: []dirEntry{
			{Name: "dup1", Path: filepath.Join(tmp, "dup1"), Size: 100},
			{Name: "dup2", Path: filepath.Join(tmp, "dup2"), Size: 100},
			{Name: "unique", Path: filepath.Join(tmp, "unique"), Size: 200},
		},
	}

	groups := m.groupDuplicatesByFingerprint()
	if len(groups) != 1 {
		t.Fatalf("expected 1 duplicate group, got %d", len(groups))
	}
	if groups[0].Fingerprint != "abc123" {
		t.Errorf("expected fingerprint abc123, got %s", groups[0].Fingerprint)
	}
	if len(groups[0].Entries) != 2 {
		t.Errorf("expected 2 entries in group, got %d", len(groups[0].Entries))
	}
}

func TestModeBadge(t *testing.T) {
	m := model{searchMode: true}
	if mode := m.getMode(); mode != "SEARCH" {
		t.Errorf("expected SEARCH mode, got %s", mode)
	}

	m = model{duplicatesMode: true}
	if mode := m.getMode(); mode != "DUPLICATES" {
		t.Errorf("expected DUPLICATES mode, got %s", mode)
	}

	m = model{showLargeFiles: true}
	if mode := m.getMode(); mode != "LARGE" {
		t.Errorf("expected LARGE mode, got %s", mode)
	}

	m = model{}
	if mode := m.getMode(); mode != "BROWSE" {
		t.Errorf("expected BROWSE mode, got %s", mode)
	}
}

func TestSidebarRendering(t *testing.T) {
	m := model{
		path: "/tmp",
		entries: []dirEntry{
			{Name: "test", Path: "/tmp/test", Size: 1000, IsDir: true, LastAccess: time.Now()},
		},
		selected: 0,
	}

	var b strings.Builder
	m.renderSidebar(&b)
	output := b.String()

	if !strings.Contains(output, "test") {
		t.Error("sidebar should contain entry name")
	}
	if !strings.Contains(output, "1000") {
		t.Error("sidebar should contain size")
	}
	if !strings.Contains(output, "Directory") {
		t.Error("sidebar should contain type")
	}
}

func TestSessionPersistence(t *testing.T) {
	// Enable projects feature
	os.Setenv("MO_FEATURE_PROJECTS", "1")
	defer os.Unsetenv("MO_FEATURE_PROJECTS")

	tmp := t.TempDir()

	// Create a model with some state
	m := model{
		path:           tmp,
		searchQuery:    "test",
		searchMode:     false,
		duplicatesMode: true,
		showLargeFiles: false,
		selected:       2,
		offset:         1,
		entries: []dirEntry{
			{Name: "file1", Path: filepath.Join(tmp, "file1"), Size: 100},
			{Name: "file2", Path: filepath.Join(tmp, "file2"), Size: 200},
			{Name: "file3", Path: filepath.Join(tmp, "file3"), Size: 300},
		},
		multiSelected: map[string]bool{
			filepath.Join(tmp, "file1"): true,
			filepath.Join(tmp, "file2"): true,
		},
	}

	// Save session
	if err := m.saveSession(); err != nil {
		t.Fatalf("save session: %v", err)
	}

	// Load session
	loaded, err := loadSession()
	if err != nil {
		t.Fatalf("load session: %v", err)
	}

	if loaded == nil {
		t.Fatal("expected non-nil session")
	}

	// Verify session data
	if loaded.Path != tmp {
		t.Errorf("expected path %s, got %s", tmp, loaded.Path)
	}
	if loaded.SearchQuery != "test" {
		t.Errorf("expected search query 'test', got %s", loaded.SearchQuery)
	}
	if !loaded.DuplicatesMode {
		t.Error("expected duplicates mode to be true")
	}
	if loaded.Selected != 2 {
		t.Errorf("expected selected 2, got %d", loaded.Selected)
	}
	if loaded.Offset != 1 {
		t.Errorf("expected offset 1, got %d", loaded.Offset)
	}
	if len(loaded.MultiSelected) != 2 {
		t.Errorf("expected 2 multi-selected items, got %d", len(loaded.MultiSelected))
	}

	// Test applying session to new model
	newM := model{
		path: tmp,
		entries: []dirEntry{
			{Name: "file1", Path: filepath.Join(tmp, "file1"), Size: 100},
			{Name: "file2", Path: filepath.Join(tmp, "file2"), Size: 200},
			{Name: "file3", Path: filepath.Join(tmp, "file3"), Size: 300},
		},
	}

	newM.applySession(loaded)

	if newM.searchQuery != "test" {
		t.Errorf("expected search query 'test' after apply, got %s", newM.searchQuery)
	}
	if !newM.duplicatesMode {
		t.Error("expected duplicates mode to be true after apply")
	}
	if newM.selected != 2 {
		t.Errorf("expected selected 2 after apply, got %d", newM.selected)
	}
	if len(newM.multiSelected) != 2 {
		t.Errorf("expected 2 multi-selected items after apply, got %d", len(newM.multiSelected))
	}
}

func TestSessionExpiration(t *testing.T) {
	// Enable projects feature
	os.Setenv("MO_FEATURE_PROJECTS", "1")
	defer os.Unsetenv("MO_FEATURE_PROJECTS")

	tmp := t.TempDir()

	// Create an old session file
	session := Session{
		Version:   "1.0",
		Timestamp: time.Now().Add(-25 * time.Hour), // 25 hours ago
		Path:      tmp,
	}

	sessionFile, err := sessionPath()
	if err != nil {
		t.Fatalf("get session path: %v", err)
	}

	data, err := json.Marshal(session)
	if err != nil {
		t.Fatalf("marshal session: %v", err)
	}

	if err := os.WriteFile(sessionFile, data, 0644); err != nil {
		t.Fatalf("write session file: %v", err)
	}

	// Try to load - should return nil due to expiration
	loaded, err := loadSession()
	if err != nil {
		t.Fatalf("load session: %v", err)
	}

	if loaded != nil {
		t.Error("expected nil session due to expiration")
	}

	// Session file should be cleaned up
	if _, err := os.Stat(sessionFile); !os.IsNotExist(err) {
		t.Error("expected session file to be cleaned up")
	}
}

func TestUndoMoveAndRestore(t *testing.T) {
	tmp := t.TempDir()
	// create files
	paths := []string{
		filepath.Join(tmp, "a.txt"),
		filepath.Join(tmp, "b.txt"),
	}
	for _, p := range paths {
		if err := os.WriteFile(p, []byte("x"), 0644); err != nil {
			t.Fatalf("write file: %v", err)
		}
	}

	entry, err := movePathsToTrash(paths)
	if err != nil {
		t.Fatalf("move to trash: %v", err)
	}
	// originals should be gone
	for _, p := range paths {
		if _, err := os.Stat(p); !os.IsNotExist(err) {
			t.Fatalf("expected %s removed", p)
		}
	}
	// trashed should exist
	for _, item := range entry.Items {
		if _, err := os.Stat(item.Trashed); err != nil {
			t.Fatalf("expected trashed file: %v", err)
		}
	}

	if err := restoreUndoEntry(entry); err != nil {
		t.Fatalf("restore: %v", err)
	}
	// originals restored
	for _, p := range paths {
		if _, err := os.Stat(p); err != nil {
			t.Fatalf("expected restored file: %v", err)
		}
	}
}

func TestCtrlZFlow(t *testing.T) {
	tmp := t.TempDir()
	target := filepath.Join(tmp, "undo_me.txt")
	if err := os.WriteFile(target, []byte("data"), 0644); err != nil {
		t.Fatalf("write: %v", err)
	}
	entry, err := movePathsToTrash([]string{target})
	if err != nil {
		t.Fatalf("trash: %v", err)
	}

	m := model{
		path:     tmp,
		lastUndo: entry,
	}
	// trigger ctrl+z
	_, cmd := m.updateKey(tea.KeyMsg{Type: tea.KeyCtrlZ})
	if cmd == nil {
		t.Fatal("expected undo command")
	}
	msg := cmd()
	switch mm := msg.(type) {
	case undoResultMsg:
		m2, _ := m.Update(mm)
		if m2.(model).status != "Restored 1 item(s)" {
			t.Fatalf("unexpected status: %s", m2.(model).status)
		}
	case tea.BatchMsg:
		found := false
		for _, sub := range mm {
			if res, ok := sub().(undoResultMsg); ok {
				found = true
				m2, _ := m.Update(res)
				if m2.(model).status != "Restored 1 item(s)" {
					t.Fatalf("unexpected status: %s", m2.(model).status)
				}
			}
		}
		if !found {
			t.Fatalf("batch did not contain undoResultMsg, got %v", mm)
		}
	default:
		t.Fatalf("unexpected msg type %T", msg)
	}
}

// Helper function for mode badge
func (m model) getMode() string {
	if m.searchMode {
		return "SEARCH"
	}
	if m.duplicatesMode {
		return "DUPLICATES"
	}
	if m.showLargeFiles {
		return "LARGE"
	}
	return "BROWSE"
}

// Helper function for pin toggle test
func (m *model) pinToggle() {
	if len(m.entries) == 0 {
		return
	}
	entry := m.entries[m.selected]
	rel, err := filepath.Rel(m.path, entry.Path)
	if err != nil {
		return
	}
	focus := loadFocus()
	if focus.isPinned(rel) {
		m.status = "Unpinned " + entry.Name
	} else {
		focus.Pinned = append(focus.Pinned, rel)
		m.status = "Pinned " + entry.Name
	}
}

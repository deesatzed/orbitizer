package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"
)

func orbitDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".orbit"), nil
}

func moleDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".mole"), nil
}

// Session represents the persisted state of a Mole analyzer session
type Session struct {
	Version        string          `json:"version"`
	Timestamp      time.Time       `json:"timestamp"`
	Path           string          `json:"path"`
	SearchQuery    string          `json:"search_query,omitempty"`
	SearchMode     bool            `json:"search_mode,omitempty"`
	DuplicatesMode bool            `json:"duplicates_mode,omitempty"`
	ShowLargeFiles bool            `json:"show_large_files,omitempty"`
	Selected       int             `json:"selected"`
	Offset         int             `json:"offset"`
	MultiSelected  map[string]bool `json:"multi_selected,omitempty"`
}

// sessionPath returns the path to the session file
func sessionPath() (string, error) {
	base, err := orbitDir()
	if err != nil {
		return "", err
	}
	if err := os.MkdirAll(base, 0755); err != nil {
		return "", err
	}
	return filepath.Join(base, "session.json"), nil
}

// saveSession persists the current model state
func (m model) saveSession() error {
	if !featureProjectsEnabled() {
		return nil // Only save session if projects feature is enabled
	}

	sessionFile, err := sessionPath()
	if err != nil {
		return err
	}
	legacyFile, _ := moleDir()

	// Create session state
	session := Session{
		Version:        "1.0",
		Timestamp:      time.Now(),
		Path:           m.path,
		SearchQuery:    m.searchQuery,
		SearchMode:     m.searchMode,
		DuplicatesMode: m.duplicatesMode,
		ShowLargeFiles: m.showLargeFiles,
		Selected:       m.selected,
		Offset:         m.offset,
	}

	// Only include multi-selected if there are any
	if len(m.multiSelected) > 0 {
		session.MultiSelected = make(map[string]bool)
		for k, v := range m.multiSelected {
			session.MultiSelected[k] = v
		}
	}

	data, err := json.MarshalIndent(session, "", "  ")
	if err != nil {
		return err
	}

	if err := os.WriteFile(sessionFile, data, 0644); err != nil {
		return err
	}

	// Best-effort legacy write for backward compatibility
	if legacyFile != "" {
		_ = os.MkdirAll(legacyFile, 0755)
		_ = os.WriteFile(filepath.Join(legacyFile, "session.json"), data, 0644)
	}
	return nil
}

// loadSession restores a previously saved session
func loadSession() (*Session, error) {
	if !featureProjectsEnabled() {
		return nil, nil
	}

	var candidates []string
	if p, err := sessionPath(); err == nil {
		candidates = append(candidates, p)
	}
	if legacyDir, err := moleDir(); err == nil {
		candidates = append(candidates, filepath.Join(legacyDir, "session.json"))
	}

	for _, sessionFile := range candidates {
		data, err := os.ReadFile(sessionFile)
		if err != nil {
			if os.IsNotExist(err) {
				continue // try next candidate
			}
			return nil, err
		}

		var session Session
		if err := json.Unmarshal(data, &session); err != nil {
			return nil, err
		}

		// Check if session is too old (older than 24 hours)
		if time.Since(session.Timestamp) > 24*time.Hour {
			_ = os.Remove(sessionFile) // Clean up old session
			return nil, nil
		}

		return &session, nil
	}

	return nil, nil
}

// applySession applies a loaded session to the current model
func (m *model) applySession(session *Session) {
	if session == nil {
		return
	}

	// Only apply if we're in the same directory
	if session.Path != m.path {
		return
	}

	// Restore search state
	m.searchQuery = session.SearchQuery
	m.searchMode = session.SearchMode

	// Restore view modes
	m.duplicatesMode = session.DuplicatesMode
	m.showLargeFiles = session.ShowLargeFiles

	// Restore selection state
	if session.Selected >= 0 && session.Selected < len(m.entries) {
		m.selected = session.Selected
	}
	if session.Offset >= 0 {
		m.offset = session.Offset
	}

	// Restore multi-selection
	if session.MultiSelected != nil {
		m.multiSelected = make(map[string]bool)
		for k, v := range session.MultiSelected {
			m.multiSelected[k] = v
		}
	}

	// Show session restored message
	m.status = "Session restored"
}

// clearSession removes the saved session file
func clearSession() error {
	var errs []error
	if sessionFile, err := sessionPath(); err == nil {
		if err := os.Remove(sessionFile); err != nil && !os.IsNotExist(err) {
			errs = append(errs, err)
		}
	}
	if legacyDir, err := moleDir(); err == nil {
		legacyFile := filepath.Join(legacyDir, "session.json")
		if err := os.Remove(legacyFile); err != nil && !os.IsNotExist(err) {
			errs = append(errs, err)
		}
	}
	if len(errs) > 0 {
		return errs[0]
	}
	return nil
}

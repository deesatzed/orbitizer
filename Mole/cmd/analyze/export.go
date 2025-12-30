package main

import (
	"encoding/csv"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

// exportFormat represents the available export formats
type exportFormat int

const (
	exportFormatJSON exportFormat = iota
	exportFormatCSV
)

// exportModal holds state for the export dialog
type exportModal struct {
	active      bool
	format      exportFormat
	destination string
	preview     string
}

// showExportModal activates the export modal
func (m *model) showExportModal() {
	m.exportModal = exportModal{
		active:      true,
		format:      exportFormatJSON,
		destination: "",
		preview:     "",
	}
}

// hideExportModal deactivates the export modal
func (m *model) hideExportModal() {
	m.exportModal = exportModal{active: false}
}

// setExportFormat sets the export format and updates preview
func (m *model) setExportFormat(f exportFormat) {
	m.exportModal.format = f
	m.updateExportPreview()
}

// setExportDestination sets the destination path and updates preview
func (m *model) setExportDestination(dest string) {
	m.exportModal.destination = dest
	m.updateExportPreview()
}

// updateExportPreview generates a preview of the export
func (m *model) updateExportPreview() {
	if m.exportModal.destination == "" {
		m.exportModal.preview = "No destination set"
		return
	}
	// For now, show a simple preview
	switch m.exportModal.format {
	case exportFormatJSON:
		m.exportModal.preview = fmt.Sprintf("Will write JSON to %s", m.exportModal.destination)
	case exportFormatCSV:
		m.exportModal.preview = fmt.Sprintf("Will write CSV to %s", m.exportModal.destination)
	}
}

// performExport executes the export with current settings
func (m model) performExport() error {
	if m.exportModal.destination == "" {
		return fmt.Errorf("no destination")
	}
	// Prefer explicit selection if present
	entries := m.exportSelection
	if len(entries) == 0 {
		entries = m.entries
		if m.searchQuery != "" {
			entries = m.filterEntries(m.entries)
		}
	}
	switch m.exportModal.format {
	case exportFormatJSON:
		return m.exportJSON(entries, m.exportModal.destination)
	case exportFormatCSV:
		return m.exportCSV(entries, m.exportModal.destination)
	default:
		return fmt.Errorf("unsupported format")
	}
}

// exportJSON writes entries as JSON
func (m model) exportJSON(entries []dirEntry, dest string) error {
	type exportEntry struct {
		Name  string `json:"name"`
		Path  string `json:"path"`
		Size  int64  `json:"size"`
		IsDir bool   `json:"is_dir"`
	}
	var out []exportEntry
	for _, e := range entries {
		out = append(out, exportEntry{
			Name:  e.Name,
			Path:  e.Path,
			Size:  e.Size,
			IsDir: e.IsDir,
		})
	}
	data, err := json.MarshalIndent(out, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(dest, data, 0644)
}

// exportCSV writes entries as CSV
func (m model) exportCSV(entries []dirEntry, dest string) error {
	file, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer file.Close()
	writer := csv.NewWriter(file)
	defer writer.Flush()
	if err := writer.Write([]string{"name", "path", "size", "is_dir"}); err != nil {
		return err
	}
	for _, e := range entries {
		if err := writer.Write([]string{e.Name, e.Path, fmt.Sprintf("%d", e.Size), fmt.Sprintf("%t", e.IsDir)}); err != nil {
			return err
		}
	}
	return nil
}

// renderExportModal renders the export dialog
func (m model) renderExportModal(b *strings.Builder) {
	fmt.Fprintf(b, "%s=== Export ===%s\n", colorPurpleBold, colorReset)
	fmt.Fprintf(b, "%sFormat:%s %s%s\n", colorGray, colorReset,
		map[exportFormat]string{exportFormatJSON: "JSON", exportFormatCSV: "CSV"}[m.exportModal.format],
		map[bool]string{true: " (active)", false: ""}[m.exportModal.format == exportFormatJSON])
	fmt.Fprintf(b, "%sDestination:%s %s%s\n", colorGray, colorReset, m.exportModal.destination, colorReset)
	fmt.Fprintf(b, "%sPreview:%s %s%s\n", colorGray, colorReset, m.exportModal.preview, colorReset)
	fmt.Fprintf(b, "%sKeys: j/k JSON/CSV, d to set destination, Enter to export, Esc to cancel%s\n", colorGray, colorReset)
}

// handleExportModalKey processes key events when export modal is active
func (m *model) handleExportModalKey(msg string) (tea.Model, tea.Cmd) {
	switch msg {
	case "j":
		m.setExportFormat(exportFormatJSON)
	case "k":
		m.setExportFormat(exportFormatCSV)
	case "d":
		// For simplicity, set destination to a default path
		ext := "json"
		if m.exportModal.format == exportFormatCSV {
			ext = "csv"
		}
		defaultDest := filepath.Join(os.Getenv("HOME"), "Desktop", "mole_export."+ext)
		m.setExportDestination(defaultDest)
	case "enter":
		if err := m.performExport(); err != nil {
			m.status = fmt.Sprintf("Export failed: %v", err)
		} else {
			m.status = fmt.Sprintf("Exported to %s", m.exportModal.destination)
		}
		m.hideExportModal()
	case "esc":
		m.hideExportModal()
	}
	return m, nil
}

package main

import "strings"

// filterEntries applies the current search query to directory entries.
func (m model) filterEntries(entries []dirEntry) []dirEntry {
	if m.searchQuery == "" {
		return entries
	}
	q := strings.ToLower(m.searchQuery)
	out := make([]dirEntry, 0, len(entries))
	for _, e := range entries {
		if strings.Contains(strings.ToLower(e.Name), q) || strings.Contains(strings.ToLower(e.Path), q) {
			out = append(out, e)
		}
	}
	return out
}

// filterLargeFiles applies the current search query to large files.
func (m model) filterLargeFiles(files []fileEntry) []fileEntry {
	if m.searchQuery == "" {
		return files
	}
	q := strings.ToLower(m.searchQuery)
	out := make([]fileEntry, 0, len(files))
	for _, f := range files {
		if strings.Contains(strings.ToLower(f.Name), q) || strings.Contains(strings.ToLower(f.Path), q) {
			out = append(out, f)
		}
	}
	return out
}

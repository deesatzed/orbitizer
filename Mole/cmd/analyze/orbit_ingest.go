package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"time"
)

type orbitProject struct {
	Path      string  `json:"path"`
	SizeBytes *int64  `json:"size_bytes"`
	Mtime     *string `json:"mtime,omitempty"`
}

type orbitIndex struct {
	Root     string         `json:"root"`
	Projects []orbitProject `json:"projects"`
}

// loadOrbitIndexForPath attempts to load ~/.orbit/index.json and returns entries if the root matches targetPath.
func loadOrbitIndexForPath(targetPath string) ([]dirEntry, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}
	idxPath := filepath.Join(home, ".orbit", "index.json")
	data, err := os.ReadFile(idxPath)
	if err != nil {
		return nil, err
	}

	var idx orbitIndex
	if err := json.Unmarshal(data, &idx); err != nil {
		return nil, err
	}

	// Resolve index root to absolute
	root := idx.Root
	if !filepath.IsAbs(root) {
		if cwd, err := os.Getwd(); err == nil {
			root = filepath.Join(cwd, root)
		} else {
			root = filepath.Clean(root)
		}
	}
	rootAbs, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}

	targetAbs, err := filepath.Abs(targetPath)
	if err != nil {
		return nil, err
	}

	if filepath.Clean(rootAbs) != filepath.Clean(targetAbs) {
		return nil, nil
	}

	var entries []dirEntry
	now := time.Now()
	for _, p := range idx.Projects {
		abs := p.Path
		if !filepath.IsAbs(abs) {
			abs = filepath.Join(rootAbs, abs)
		}
		size := int64(-1)
		if p.SizeBytes != nil {
			size = *p.SizeBytes
		}
		entry := dirEntry{
			Name:       filepath.Base(abs),
			Path:       abs,
			Size:       size,
			IsDir:      true,
			LastAccess: now,
		}
		entries = append(entries, entry)
	}

	return entries, nil
}

package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

type projectEntry struct {
	Path          string `json:"path"`
	Kind          string `json:"kind"`
	Pinned        bool   `json:"pinned"`
	LatestMtime   int64  `json:"latest_mtime,omitempty"`
	SizeBytes     int64  `json:"size_bytes,omitempty"`
	ArtifactCount int    `json:"artifact_count,omitempty"`
	HasGit        bool   `json:"has_git"`
	HasRust       bool   `json:"has_rust"`
	HasNode       bool   `json:"has_node"`
	HasPython     bool   `json:"has_python"`
	Fingerprint   string `json:"fingerprint,omitempty"`
}

type projectIndex struct {
	Version     string         `json:"version"`
	Root        string         `json:"root"`
	GeneratedAt time.Time      `json:"generated_at"`
	Projects    []projectEntry `json:"projects"`
}

type focusList struct {
	Pinned []string `json:"pinned"`
}

func saveProjectIndex(root string, projects []projectEntry) error {
	idx := projectIndex{
		Version:     "0.1",
		Root:        root,
		GeneratedAt: time.Now(),
		Projects:    projects,
	}
	data, err := json.MarshalIndent(idx, "", "  ")
	if err != nil {
		return err
	}
	outDir := filepath.Join(root, ".mole")
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		return err
	}
	outPath := filepath.Join(outDir, "projects.json")
	return os.WriteFile(outPath, data, 0o644)
}

// runProjectDiscovery scans for projects up to a modest depth and writes .mole/projects.json.
func runProjectDiscovery(root string) error {
	maxDepth := 4
	var projects []projectEntry

	rootAbs, err := filepath.Abs(root)
	if err != nil {
		return err
	}

	focus := loadFocus()

	err = filepath.WalkDir(rootAbs, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return nil // best-effort
		}
		if !d.IsDir() {
			return nil
		}
		rel, _ := filepath.Rel(rootAbs, path)
		if rel == "." {
			return nil
		}
		depth := strings.Count(rel, string(filepath.Separator))
		if depth > maxDepth {
			return filepath.SkipDir
		}

		// Skip known heavy directories
		base := d.Name()
		skip := map[string]bool{
			".git":         true,
			"node_modules": true,
			"target":       true,
			".venv":        true,
			"venv":         true,
			".next":        true,
			"build":        true,
			"dist":         true,
			"Library":      true,
			"Applications": true,
			".Trash":       true,
		}
		if skip[base] {
			return filepath.SkipDir
		}

		// Detect markers for languages
		var hasGit, hasRust, hasNode, hasPython bool
		entries, _ := os.ReadDir(path)
		for _, e := range entries {
			if e.IsDir() && e.Name() == ".git" {
				hasGit = true
			}
			if !e.IsDir() {
				switch e.Name() {
				case "Cargo.toml":
					hasRust = true
				case "package.json":
					hasNode = true
				case "pyproject.toml", "pytest.ini":
					hasPython = true
				}
			}
		}

		// Heuristic: treat any directory with a language marker as a project root
		if hasRust || hasNode || hasPython {
			var latest int64
			var total int64
			var artifacts int

			filepath.WalkDir(path, func(p string, de os.DirEntry, err error) error {
				if err != nil {
					return nil
				}
				info, ierr := de.Info()
				if ierr == nil && !info.IsDir() {
					mt := info.ModTime().Unix()
					if mt > latest {
						latest = mt
					}
					total += info.Size()
					name := strings.ToLower(info.Name())
					if strings.Contains(name, "readme") || strings.Contains(name, "export") || strings.Contains(name, "plan") {
						artifacts++
					}
				}
				return nil
			})

			fp := hashFingerprint(path, total, latest)
			relPath := rel
			pinned := focus.isPinned(relPath)
			projects = append(projects, projectEntry{
				Path:          relPath,
				Kind:          "standalone",
				Pinned:        pinned,
				LatestMtime:   latest,
				SizeBytes:     total,
				ArtifactCount: artifacts,
				HasGit:        hasGit,
				HasRust:       hasRust,
				HasNode:       hasNode,
				HasPython:     hasPython,
				Fingerprint:   fp,
			})

			// Do not descend further inside this project root
			return filepath.SkipDir
		}
		return nil
	})
	if err != nil {
		return err
	}

	return saveProjectIndex(rootAbs, projects)
}

func hashFingerprint(path string, size int64, latest int64) string {
	h := sha256.New()
	_, _ = h.Write([]byte(path))
	_, _ = h.Write([]byte(fmt.Sprintf(":%d:%d", size, latest)))
	return hex.EncodeToString(h.Sum(nil))
}

func loadFocus() focusList {
	// Try shared Orbit focus first, then Mole legacy
	home := os.Getenv("HOME")
	if home == "" {
		return focusList{}
	}
	candidates := []string{
		filepath.Join(home, ".orbit", "focus.json"),
		filepath.Join(home, ".config", "mole", "focus.json"),
	}
	for _, p := range candidates {
		data, err := os.ReadFile(p)
		if err != nil {
			continue
		}
		var f focusList
		if err := json.Unmarshal(data, &f); err != nil {
			continue
		}
		return f
	}
	return focusList{}
}

func (f focusList) isPinned(rel string) bool {
	for _, p := range f.Pinned {
		if p == rel {
			return true
		}
	}
	return false
}

func saveFocus(f focusList) error {
	home := os.Getenv("HOME")
	if home == "" {
		return fmt.Errorf("HOME not set")
	}
	primary := filepath.Join(home, ".orbit", "focus.json")
	legacy := filepath.Join(home, ".config", "mole", "focus.json")

	data, err := json.MarshalIndent(f, "", "  ")
	if err != nil {
		return err
	}

	if err := os.MkdirAll(filepath.Dir(primary), 0755); err == nil {
		if err := os.WriteFile(primary, data, 0644); err != nil {
			return err
		}
	}

	// Best-effort legacy write
	if err := os.MkdirAll(filepath.Dir(legacy), 0755); err == nil {
		_ = os.WriteFile(legacy, data, 0644)
	}
	return nil
}

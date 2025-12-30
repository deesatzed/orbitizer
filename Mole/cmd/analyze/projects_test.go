package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestRunProjectDiscoveryWritesIndex(t *testing.T) {
	dir := t.TempDir()
	app := filepath.Join(dir, "app1")
	if err := os.MkdirAll(app, 0o755); err != nil {
		t.Fatalf("mkdir app: %v", err)
	}
	// Language markers + content
	if err := os.WriteFile(filepath.Join(app, "Cargo.toml"), []byte("[package]\nname=\"app1\""), 0o644); err != nil {
		t.Fatalf("write Cargo.toml: %v", err)
	}
	if err := os.WriteFile(filepath.Join(app, "README.md"), []byte("hello"), 0o644); err != nil {
		t.Fatalf("write README: %v", err)
	}

	if err := runProjectDiscovery(dir); err != nil {
		t.Fatalf("runProjectDiscovery: %v", err)
	}

	outPath := filepath.Join(dir, ".mole", "projects.json")
	data, err := os.ReadFile(outPath)
	if err != nil {
		t.Fatalf("read projects.json: %v", err)
	}

	var idx projectIndex
	if err := json.Unmarshal(data, &idx); err != nil {
		t.Fatalf("unmarshal projects.json: %v", err)
	}
	if len(idx.Projects) != 1 {
		t.Fatalf("expected 1 project, got %d", len(idx.Projects))
	}
	p := idx.Projects[0]
	if p.Path != "app1" {
		t.Fatalf("expected path app1, got %s", p.Path)
	}
	if !p.HasRust {
		t.Fatalf("expected HasRust=true")
	}
	if p.ArtifactCount == 0 {
		t.Fatalf("expected artifacts counted from README")
	}
	if p.SizeBytes == 0 {
		t.Fatalf("expected size > 0")
	}
}

func TestFeatureProjectsEnabled(t *testing.T) {
	t.Setenv("MO_FEATURE_PROJECTS", "true")
	if !featureProjectsEnabled() {
		t.Fatalf("expected featureProjectsEnabled to be true")
	}
	t.Setenv("MO_FEATURE_PROJECTS", "0")
	if featureProjectsEnabled() {
		t.Fatalf("expected featureProjectsEnabled to be false")
	}
}

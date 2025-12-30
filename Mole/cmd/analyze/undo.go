package main

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

type undoItem struct {
	Original string
	Trashed  string
}

type undoEntry struct {
	TrashRoot string
	Items     []undoItem
}

// getTrashRoot creates a unique trash directory under ~/.mole/trash
func getTrashRoot() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	root := filepath.Join(home, ".mole", "trash", time.Now().Format("20060102T150405.000"))
	if err := os.MkdirAll(root, 0755); err != nil {
		return "", err
	}
	return root, nil
}

// movePathsToTrash renames the given paths into a trash root for undo.
func movePathsToTrash(paths []string) (*undoEntry, error) {
	if len(paths) == 0 {
		return nil, fmt.Errorf("no paths to trash")
	}
	trashRoot, err := getTrashRoot()
	if err != nil {
		return nil, err
	}
	entry := &undoEntry{TrashRoot: trashRoot}

	for i, p := range paths {
		base := filepath.Base(p)
		target := filepath.Join(trashRoot, fmt.Sprintf("%d_%s", i, base))
		// Ensure parent exists
		if err := os.MkdirAll(filepath.Dir(target), 0755); err != nil {
			return nil, err
		}
		if err := os.Rename(p, target); err != nil {
			// If rename fails (cross-device), fall back to remove all to avoid partial state
			_ = os.RemoveAll(target)
			return nil, err
		}
		entry.Items = append(entry.Items, undoItem{
			Original: p,
			Trashed:  target,
		})
	}

	return entry, nil
}

// restoreUndoEntry moves trashed items back to their original locations.
func restoreUndoEntry(entry *undoEntry) error {
	if entry == nil {
		return fmt.Errorf("nil undo entry")
	}
	for _, item := range entry.Items {
		// Ensure parent exists
		if err := os.MkdirAll(filepath.Dir(item.Original), 0755); err != nil {
			return err
		}
		if err := os.Rename(item.Trashed, item.Original); err != nil {
			return err
		}
	}
	_ = os.RemoveAll(entry.TrashRoot)
	return nil
}

// moveToTrashCmd wraps movePathsToTrash in a Tea command.
func moveToTrashCmd(paths []string, counter *int64) tea.Cmd {
	return func() tea.Msg {
		entry, err := movePathsToTrash(paths)
		count := int64(len(paths))
		if counter != nil {
			*counter = count
		}
		return deleteProgressMsg{
			done:  true,
			err:   err,
			count: count,
			path:  "", // signals refresh
			undo:  entry,
		}
	}
}

// undoCmd restores the latest undo entry.
func undoCmd(entry *undoEntry) tea.Cmd {
	return func() tea.Msg {
		err := restoreUndoEntry(entry)
		restored := 0
		if entry != nil {
			restored = len(entry.Items)
		}
		return undoResultMsg{err: err, restored: restored}
	}
}

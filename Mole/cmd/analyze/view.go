//go:build darwin

package main

import (
	"fmt"
	"path/filepath"
	"strings"
	"sync/atomic"
)

// renderHelpBar draws a persistent help line and feature flag banner.
func (m model) renderHelpBar(b *strings.Builder) {
	// Feature flag banner if missing
	if !featureProjectsEnabled() {
		fmt.Fprintf(b, "%sEnable projects mode: export MO_FEATURE_PROJECTS=1%s\n", colorYellow, colorReset)
	}
	// Mode badge
	mode := "BROWSE"
	if m.searchMode {
		mode = "SEARCH"
	}
	if m.duplicatesMode {
		mode = "DUPLICATES"
	}
	if m.showLargeFiles {
		mode = "LARGE"
	}
	// Filter count
	filterCount := len(m.entries)
	if m.searchQuery != "" {
		filterCount = len(m.filterEntries(m.entries))
	}
	fmt.Fprintf(b, "%sOrbitizer  [%s]%s  Filter: %d/%d  Projects: %s%s\n",
		colorPurpleBold, mode, colorReset, filterCount, len(m.entries),
		map[bool]string{true: "ON", false: "OFF"}[featureProjectsEnabled()], colorReset)
	// Key hints
	fmt.Fprintf(b, "%s? Help  q Quit  / Search  d Duplicates  e Export  p Pins%s\n", colorGray, colorReset)
	fmt.Fprintf(b, "%s------------------------------------------------------------------------%s\n", colorGray, colorReset)
}

// View renders the TUI display.
func (m model) View() string {
	var b strings.Builder
	fmt.Fprintln(&b)

	// === Help bar ===
	m.renderHelpBar(&b)

	// === Export modal ===
	if m.exportModal.active {
		m.renderExportModal(&b)
		return b.String()
	}
	// === Search prompt ===
	if m.searchMode {
		fmt.Fprintf(&b, "%s/%s%s\n", colorCyan, m.searchInput, colorReset)
		filtered := m.filterEntries(m.entries)
		fmt.Fprintf(&b, "%sResults: %d/%d%s\n", colorGray, len(filtered), len(m.entries), colorReset)
		fmt.Fprintf(&b, "%s------------------------------------------------------------------------%s\n", colorGray, colorReset)
	}
	// === Duplicates view ===
	if m.duplicatesMode {
		groups := m.groupDuplicatesByFingerprint()
		m.renderDuplicateGroups(&b, groups)
		return b.String()
	}

	// === Pin indicator helper ===
	isPinned := func(entry dirEntry) bool {
		if !featureProjectsEnabled() {
			return false
		}
		rel, err := filepath.Rel(m.path, entry.Path)
		if err != nil {
			return false
		}
		focus := loadFocus()
		return focus.isPinned(rel)
	}

	// === Main content ===
	var mainBuilder strings.Builder

	if m.inOverviewMode() {
		fmt.Fprintf(&mainBuilder, "%sAnalyze Disk%s\n", colorPurpleBold, colorReset)
		if m.overviewScanning {
			// Check if we're in initial scan (all entries are pending)
			allPending := true
			for _, entry := range m.entries {
				if entry.Size >= 0 {
					allPending = false
					break
				}
			}

			if allPending {
				// Show prominent loading screen for initial scan
				fmt.Fprintf(&mainBuilder, "%s%s%s%s Analyzing disk usage, please wait...%s\n",
					colorCyan, colorBold,
					spinnerFrames[m.spinner],
					colorReset, colorReset)
				fmt.Fprintf(&b, "%s", mainBuilder.String())
				return b.String()
			} else {
				// Progressive scanning - show subtle indicator
				fmt.Fprintf(&mainBuilder, "%sSelect a location to explore:%s  ", colorGray, colorReset)
				fmt.Fprintf(&mainBuilder, "%s%s%s%s Scanning...\n\n", colorCyan, colorBold, spinnerFrames[m.spinner], colorReset)
			}
		} else {
			// Check if there are still pending items
			hasPending := false
			for _, entry := range m.entries {
				if entry.Size < 0 {
					hasPending = true
					break
				}
			}
			if hasPending {
				fmt.Fprintf(&mainBuilder, "%sSelect a location to explore:%s  ", colorGray, colorReset)
				fmt.Fprintf(&mainBuilder, "%s%s%s%s Scanning...\n\n", colorCyan, colorBold, spinnerFrames[m.spinner], colorReset)
			} else {
				fmt.Fprintf(&mainBuilder, "%sSelect a location to explore:%s\n\n", colorGray, colorReset)
			}
		}
	} else {
		fmt.Fprintf(&mainBuilder, "%sAnalyze Disk%s  %s%s%s", colorPurpleBold, colorReset, colorGray, displayPath(m.path), colorReset)
		if !m.scanning {
			fmt.Fprintf(&mainBuilder, "  |  Total: %s", humanizeBytes(m.totalSize))
		}
		fmt.Fprintf(&mainBuilder, "\n\n")
	}

	if m.deleting {
		// Show delete progress
		count := int64(0)
		if m.deleteCount != nil {
			count = atomic.LoadInt64(m.deleteCount)
		}

		fmt.Fprintf(&mainBuilder, "%s%s%s%s Deleting: %s%s items%s removed, please wait...\n",
			colorCyan, colorBold,
			spinnerFrames[m.spinner],
			colorReset,
			colorYellow, formatNumber(count), colorReset)

	}

	if m.scanning {
		// Show scanning progress
		files := int64(0)
		dirs := int64(0)
		bytes := int64(0)
		if m.filesScanned != nil {
			files = atomic.LoadInt64(m.filesScanned)
		}
		if m.dirsScanned != nil {
			dirs = atomic.LoadInt64(m.dirsScanned)
		}
		if m.bytesScanned != nil {
			bytes = atomic.LoadInt64(m.bytesScanned)
		}

		fmt.Fprintf(&mainBuilder, "%s%s%s%s Scanning: %s files, %s dirs, %s%s\n",
			colorCyan, colorBold,
			spinnerFrames[m.spinner],
			colorReset,
			colorYellow, formatNumber(files), formatNumber(dirs), humanizeBytes(bytes))

	}

	// Render main content (directory entries or large files)
	if m.showLargeFiles {
		if len(m.largeFiles) == 0 {
			fmt.Fprintln(&b, "  No large files found")
		} else {
			maxLargeSize := int64(1)
			for _, file := range m.largeFiles {
				if file.Size > maxLargeSize {
					maxLargeSize = file.Size
				}
			}
			nameWidth := calculateNameWidth(m.width)
			viewport := calculateViewport(m.height, true)
			start := m.largeOffset
			if start < 0 {
				start = 0
			}
			end := start + viewport
			if end > len(m.largeFiles) {
				end = len(m.largeFiles)
			}

			for idx := start; idx < end; idx++ {
				file := m.largeFiles[idx]
				shortPath := displayPath(file.Path)
				shortPath = truncateMiddle(shortPath, nameWidth)
				paddedPath := padName(shortPath, nameWidth)
				entryPrefix := "   "
				nameColor := ""
				sizeColor := colorGray
				numColor := ""

				// Check if this item is multi-selected (by path, not index)
				isMultiSelected := m.largeMultiSelected != nil && m.largeMultiSelected[file.Path]
				selectIcon := "○"
				if isMultiSelected {
					selectIcon = fmt.Sprintf("%s●%s", colorGreen, colorReset)
					nameColor = colorGreen
				}

				if idx == m.largeSelected {
					entryPrefix = fmt.Sprintf(" %s%s▶%s ", colorCyan, colorBold, colorReset)
					if !isMultiSelected {
						nameColor = colorCyan
					}
					sizeColor = colorCyan
					numColor = colorCyan
				}
				size := humanizeBytes(file.Size)
				bar := coloredProgressBar(file.Size, maxLargeSize, 0)
				fmt.Fprintf(&mainBuilder, "%s%s %s%2d.%s %s  |  📄 %s%s%s  %s%10s%s\n",
					entryPrefix, selectIcon, numColor, idx+1, colorReset, bar, nameColor, paddedPath, colorReset, sizeColor, size, colorReset)
			}
		}
	} else {
		if len(m.entries) == 0 {
			fmt.Fprintln(&b, "  Empty directory")
		} else {
			// Determine which list to render (filtered or full)
			entriesToRender := m.entries
			if m.searchQuery != "" {
				entriesToRender = m.filterEntries(m.entries)
			}
			if m.inOverviewMode() {
				maxSize := int64(1)
				for _, entry := range entriesToRender {
					if entry.Size > maxSize {
						maxSize = entry.Size
					}
				}
				totalSize := m.totalSize
				// For overview mode, use a fixed small width since path names are short
				// (~/Downloads, ~/Library, etc. - max ~15 chars)
				nameWidth := 20
				for idx, entry := range entriesToRender {
					icon := "📁"
					sizeVal := entry.Size
					barValue := sizeVal
					if barValue < 0 {
						barValue = 0
					}
					var percent float64
					if totalSize > 0 && sizeVal >= 0 {
						percent = float64(sizeVal) / float64(totalSize) * 100
					} else {
						percent = 0
					}
					percentStr := fmt.Sprintf("%5.1f%%", percent)
					if totalSize == 0 || sizeVal < 0 {
						percentStr = "  --  "
					}
					bar := coloredProgressBar(barValue, maxSize, percent)
					sizeText := "pending.."
					if sizeVal >= 0 {
						sizeText = humanizeBytes(sizeVal)
					}
					sizeColor := colorGray
					if sizeVal >= 0 && totalSize > 0 {
						switch {
						case percent >= 50:
							sizeColor = colorRed
						case percent >= 20:
							sizeColor = colorYellow
						case percent >= 5:
							sizeColor = colorBlue
						default:
							sizeColor = colorGray
						}
					}
					entryPrefix := "   "
					name := trimNameWithWidth(entry.Name, nameWidth)
					paddedName := padName(name, nameWidth)
					nameSegment := fmt.Sprintf("%s %s", icon, paddedName)
					numColor := ""
					percentColor := ""
					if idx == m.selected {
						entryPrefix = fmt.Sprintf(" %s%s▶%s ", colorCyan, colorBold, colorReset)
						nameSegment = fmt.Sprintf("%s%s %s%s", colorCyan, icon, paddedName, colorReset)
						numColor = colorCyan
						percentColor = colorCyan
						sizeColor = colorCyan
					}
					displayIndex := idx + 1

					// Priority: cleanable > unused time
					var hintLabel string
					if entry.IsDir && isCleanableDir(entry.Path) {
						hintLabel = fmt.Sprintf("%s🧹%s", colorYellow, colorReset)
					} else {
						// For overview mode, get access time on-demand if not set
						lastAccess := entry.LastAccess
						if lastAccess.IsZero() && entry.Path != "" {
							lastAccess = getLastAccessTime(entry.Path)
						}
						if unusedTime := formatUnusedTime(lastAccess); unusedTime != "" {
							hintLabel = fmt.Sprintf("%s%s%s", colorGray, unusedTime, colorReset)
						}
					}

					if hintLabel == "" {
						fmt.Fprintf(&mainBuilder, "%s%s%2d.%s %s %s%s%s  |  %s %s%10s%s\n",
							entryPrefix, numColor, displayIndex, colorReset, bar, percentColor, percentStr, colorReset,
							nameSegment, sizeColor, sizeText, colorReset)
					} else {
						fmt.Fprintf(&mainBuilder, "%s%s%2d.%s %s %s%s%s  |  %s %s%10s%s  %s\n",
							entryPrefix, numColor, displayIndex, colorReset, bar, percentColor, percentStr, colorReset,
							nameSegment, sizeColor, sizeText, colorReset, hintLabel)
					}
				}
			} else {
				// Normal mode with sizes and progress bars
				maxSize := int64(1)
				for _, entry := range entriesToRender {
					if entry.Size > maxSize {
						maxSize = entry.Size
					}
				}

				viewport := calculateViewport(m.height, false)
				nameWidth := calculateNameWidth(m.width)
				start := m.offset
				if start < 0 {
					start = 0
				}
				end := start + viewport
				if end > len(entriesToRender) {
					end = len(entriesToRender)
				}

				for idx := start; idx < end; idx++ {
					entry := entriesToRender[idx]
					icon := "📄"
					if entry.IsDir {
						icon = "📁"
					}
					size := humanizeBytes(entry.Size)
					name := trimNameWithWidth(entry.Name, nameWidth)
					paddedName := padName(name, nameWidth)

					// Calculate percentage
					percent := float64(entry.Size) / float64(m.totalSize) * 100
					percentStr := fmt.Sprintf("%5.1f%%", percent)

					// Get colored progress bar
					bar := coloredProgressBar(entry.Size, maxSize, percent)

					// Color the size based on magnitude
					var sizeColor string
					if percent >= 50 {
						sizeColor = colorRed
					} else if percent >= 20 {
						sizeColor = colorYellow
					} else if percent >= 5 {
						sizeColor = colorBlue
					} else {
						sizeColor = colorGray
					}

					// Check if this item is multi-selected (by path, not index)
					isMultiSelected := m.multiSelected != nil && m.multiSelected[entry.Path]
					selectIcon := "○"
					nameColor := ""
					if isMultiSelected {
						selectIcon = fmt.Sprintf("%s●%s", colorGreen, colorReset)
						nameColor = colorGreen
					}

					// Keep chart columns aligned even when arrow is shown
					entryPrefix := "   "
					nameSegment := fmt.Sprintf("%s %s", icon, paddedName)
					if nameColor != "" {
						nameSegment = fmt.Sprintf("%s%s %s%s", nameColor, icon, paddedName, colorReset)
					}
					numColor := ""
					percentColor := ""
					if idx == m.selected {
						entryPrefix = fmt.Sprintf(" %s%s▶%s ", colorCyan, colorBold, colorReset)
						if !isMultiSelected {
							nameSegment = fmt.Sprintf("%s%s %s%s", colorCyan, icon, paddedName, colorReset)
						}
						numColor = colorCyan
						percentColor = colorCyan
						sizeColor = colorCyan
					}

					displayIndex := idx + 1

					// Priority: cleanable > unused time > pin
					var hintLabel string
					if entry.IsDir && isCleanableDir(entry.Path) {
						hintLabel = fmt.Sprintf("%s🧹%s", colorYellow, colorReset)
					} else if isPinned(entry) {
						hintLabel = fmt.Sprintf("%s★%s", colorGreen, colorReset)
					} else {
						// Get access time on-demand if not set
						lastAccess := entry.LastAccess
						if lastAccess.IsZero() && entry.Path != "" {
							lastAccess = getLastAccessTime(entry.Path)
						}
						if unusedTime := formatUnusedTime(lastAccess); unusedTime != "" {
							hintLabel = fmt.Sprintf("%s%s%s", colorGray, unusedTime, colorReset)
						}
					}

					if hintLabel == "" {
						fmt.Fprintf(&mainBuilder, "%s%s %s%2d.%s %s %s%s%s  |  %s %s%10s%s\n",
							entryPrefix, selectIcon, numColor, displayIndex, colorReset, bar, percentColor, percentStr, colorReset,
							nameSegment, sizeColor, size, colorReset)
					} else {
						fmt.Fprintf(&mainBuilder, "%s%s %s%2d.%s %s %s%s%s  |  %s %s%10s%s  %s\n",
							entryPrefix, selectIcon, numColor, displayIndex, colorReset, bar, percentColor, percentStr, colorReset,
							nameSegment, sizeColor, size, colorReset, hintLabel)
					}
				}
			}
		}
	}

	fmt.Fprintln(&b)
	if m.inOverviewMode() {
		fmt.Fprintf(&b, "%s%s%s%s Navigation: up/down to move, enter to explore, 'b' to return to overview, 'q' to quit%s", colorGray, colorBold, colorCyan, colorReset, colorReset)
	} else {
		fmt.Fprintf(&b, "%s%s%s%s Navigation: up/down to move, enter to explore, back to return, 'q' to quit%s", colorGray, colorBold, colorCyan, colorReset, colorReset)
	}

	// === Simple sidebar (below main content) ===
	if len(m.entries) > 0 {
		fmt.Fprintf(&b, "\n\n")
		m.renderSidebar(&b)
	}

	return b.String()
}

func calculateViewport(termHeight int, isLargeFiles bool) int {
	if termHeight <= 0 {
		// Terminal height unknown, use default
		return defaultViewport
	}

	// Calculate reserved space for UI elements
	reserved := 6 // header (3-4 lines) + footer (2 lines)
	if isLargeFiles {
		reserved = 5 // Large files view has less overhead
	}
	viewport := termHeight - reserved
	if viewport < 1 {
		viewport = 1
	}
	return viewport
}

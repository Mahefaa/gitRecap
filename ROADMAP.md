# gitRecap Roadmap & Future Features

This document outlines the planned features, enhancements, and UI/UX overhauls for gitRecap. 
It ensures we do not lose track of the ambitious goals set for this project.

## ⚡ Performance First (In Progress)
- [x] Limit recursive directory scanning depth to prevent I/O blocking.
- [x] Batch process terminal events to eliminate UI lag/stutter on rapid key presses.
- [ ] **Multi-threading (Rayon):** Parallelize git parsing (`git2` `revwalk`) across all CPU cores when scanning multiple repositories.
- [ ] Optimize `revwalk` to strictly halt traversal once it passes the time filter threshold.
- [ ] Only render the visible viewport (`ratatui` does this generally, but ensure we don't build 10,000 ListItems if only 50 are visible).

## 🖥️ UI / UX Refactor & Graphical Overhaul
- [ ] Restructure the UI components for clarity and lightweight performance on low-spec PCs.
- [ ] **Optional GUI Backend:** Evaluate migrating the terminal backend to a GPU-accelerated graphics layer (like `alacritty` recommendations, or a hybrid Rust GUI like `egui` / `iced`) allowing ultra-fast rendering for powerful GPUs, while retaining a fallback mode for pure TUI environments.

## 🛠️ Vim-Style Commands (`:`)
- [ ] `:w <filename>` / `:export <filename>`: Export recap to a custom file (e.g., `:w report.md`).
- [ ] `:q!`: Force quit instantly without confirmation.
- [ ] `:sort <criteria>`: Sort projects dynamically by activity, name, or commit count.
- [ ] `:since <time>` / `:until <time>`: Natural language date filtering (e.g., `:since 1 week ago`).
- [ ] `:fetch all`: Run a background job to sync all known repositories.

## 🌟 Game-Changing Features
1. **Commit Diff Viewer (Split Screen)**
   - Pressing `Enter` on a commit opens a syntax-highlighted diff (via `git show`) in a bottom split pane.
2. **Interactive Commit Editing**
   - Press `E` on an unpushed commit to open `$EDITOR` and seamlessly run `git commit --amend` to rewrite history.
3. **Global Fuzzy Finder**
   - Press `/` to trigger a global fuzzy search window (similar to `fzf`) across all projects, branches, and commits simultaneously.
4. **Activity Dashboard**
   - A dedicated `AppMode::Dashboard` featuring `ratatui` charts (heatmaps, bar charts) visualizing commit frequency and productivity over the selected timeline.

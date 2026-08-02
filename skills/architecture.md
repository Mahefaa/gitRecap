---
name: GitRecap Architecture
description: Understanding the inner workings of GitRecap
---

# GitRecap Architecture

## Overview
GitRecap is a TUI application built with Ratatui and Crossterm. It tracks git commits across multiple local repositories and aggregates them.

## Core Components
- **`app.rs` (App & AppMode):** The central state machine. `AppMode` enum defines the current view (Normal, Details, Input paths, File Explorer, etc.). The `App` struct holds the state (filters, projects list, flash messages).
- **`ui.rs`:** Handles all rendering. The layout is split into Top Bar, Main Content (which switches depending on `AppMode`), and Footer (contextual help).
- **`git_utils.rs`:** Interfaces with `git2` to discover repositories (`find_git_repos`), parse commits based on date ranges and authors (`get_commits`), and checks if commits are pushed.
- **`config.rs`:** Persists state to `~/.config/git-recap/config.json`. Manages multiple `AppProfile`s (which store sources, disabled, and removed projects).
- **`file_explorer.rs`:** Handles the custom file explorer with Vim motions (`gg`, `G`, `m<c>`) to traverse the file system and manually add sources.

## Data Flow
1. User presses a key (`main.rs`).
2. The key updates `App` state or `AppMode`.
3. If an action requires git computation (like changing dates or profiles), `app.reload_data()` or `app.scan_sources()` is called.
4. `ui.rs` renders the updated state on the next loop iteration.

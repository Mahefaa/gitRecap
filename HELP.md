# gitRecap Help Guide

Welcome to **gitRecap**! This guide will help you understand how to navigate, filter, and manage your Git repositories effectively.

## 🚀 Basic Navigation
- `j` / `Down Arrow`: Move down.
- `k` / `Up Arrow`: Move up.
- `g`: Jump to the top of the list.
- `G`: Jump to the bottom of the list.
- `Space`: Toggle a project on/off (excludes it from the right-hand view).
- `s`: Toggle all projects on/off.
- `c`: Expand/Collapse the currently selected project or commit node.
- `Enter` / `l` / `Right Arrow`: Enter the Details view (right-side panel).
- `Esc` / `h` / `Left Arrow`: Leave the Details view.
- `q`: Quit the application.

## 🔍 Filters
Filters allow you to narrow down exactly what commits you see. **All filters are now saved persistently to your current Profile!**
- `a`: Set Author filter (e.g. "John Doe").
- `b`: Set Branch filter (e.g. "main", "feat-auth").
- `d`: Set Date filter (Supports natural language!).
  - *Examples:* "today", "yesterday", "1 week", "2023-01-01..2023-01-31".
- `/`: Search/Filter commits dynamically.

## 🗂️ Profiles & Project Management
- `P`: Switch profiles (e.g. "work", "personal"). Profiles remember your filters and tracked directories.
- `A`: Add a specific folder to the tracker manually.
- `.` : Add your current working directory to the tracker.
- `p`: Open the visual File Explorer to find and add parent folders.
- `r` / `Delete`: Remove the selected project from the tracker.
- `D`: Remove all projects from the tracker.
- `R`: Force reload data (triggers parallel multi-threading scan).
- `e`: Export the current view to `summary.txt`.

## 📤 Git Integration
- `u`: Soft push `git push` on the selected repository.
- `U`: Force push `git push --force` on the selected repository.
- **Unpushed Commits:** Any commit labeled with a red `*` has not yet been pushed to your remote origin.

## ⌨️ Command Mode
Press `:` to enter command mode, just like Vim!
- `:cp`: Collapse all projects.
- `:ep`: Expand all projects.
- `:cd`: Collapse all dates.
- `:ed`: Expand all dates.
- `:cb`: Collapse all branches.
- `:eb`: Expand all branches.
- *(More commands coming soon! See ROADMAP.md)*

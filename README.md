# GitRecap 🧠

**GitRecap**: *Because "What on earth did I do yesterday?" is a question you ask way too often.*

GitRecap is a blazingly fast, heavily-optimized Terminal User Interface (TUI) that scans your local directories to find Git repositories, analyzes your commits, and groups them beautifully by branch, date, and author. Finally, a tool that proves you actually wrote code this week instead of just staring at StackOverflow.

## Features
- 🚀 **Blazingly Fast**: Built in Rust using `ratatui` and `git2`.
- 📁 **File Explorer**: Built-in file explorer with fuzzy search to easily add new workspaces as sources.
- 🌿 **Branch-Aware**: Groups your commits by the exact local branch they were made on.
- 👤 **Author Autocomplete**: Automatically extracts known authors from recent commits to help you filter efficiently.
- 💾 **Persistent Profiles**: Save your enabled projects, sources, and removed paths into separate profiles (e.g., `work`, `personal`).
- ⌨️ **True Vim Keybindings**: Full support for `h`, `j`, `k`, `l`, `/` to search, and `Esc` to cancel. Navigate entirely without leaving the home row.
- 📝 **Export Summary**: Export a clean, text-based summary of your commits for daily standups or invoices.

## Quick Start
```bash
# Clone the repository
git clone git@github.com:Mahefaa/gitRecap.git
cd gitRecap

# Run the app
cargo run --release
```

## Usage & Keybindings
When you launch the app, you will start in `Normal` mode.
- **`p`**: Open File Explorer to add a new directory to scan for Git projects.
- **`P`**: Switch or Create a Profile.
- **`Space`**: Toggle (enable/disable) querying for a specific project.
- **`r` / `Delete`**: Permanently remove a project from your profile.
- **`a`**: Filter commits by Author (with autocomplete).
- **`d`**: Filter commits by Date (YYYY-MM-DD).
- **`Enter` / `l`**: View commits for the currently selected project.
- **`e`**: Export a clean summary to `summary.txt`.
- **`q`**: Quit

## Contributing
See [CONTRIBUTING_AI.md](CONTRIBUTING_AI.md) for architecture details and AI-agent guidelines.

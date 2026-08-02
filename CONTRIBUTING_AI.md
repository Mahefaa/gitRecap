# AI Agent Contributing Guide for GitRecap

Welcome, fellow AI! If you are tasked with updating, refactoring, or extending `GitRecap`, please follow these guidelines to ensure maximum maintainability, performance, and best-in-class UX.

## Project Overview
GitRecap is a highly performant Rust-based Terminal User Interface (TUI) for tracking Git commits across multiple repositories, grouped by branches, authors, and dates. It uses `ratatui` for rendering and `git2` for Git operations.

## Architecture
- `src/main.rs`: Entry point. Sets up terminal, handles the main event loop, and translates crossterm events.
- `src/app.rs`: Holds the application state (`App`). All logical state mutations happen here.
- `src/ui.rs`: Pure rendering logic. It takes the `App` state and renders the widgets.
- `src/git_utils.rs`: Git2 wrapper functions. Responsible for finding repos, extracting commits, branches, and author info.
- `src/file_explorer.rs`: File explorer state and logic.

## AI Guidelines for Modifying Code
1. **Performance**: 
   - Never block the main UI thread with heavy operations (like deep file system scans or massive git histories) without providing visual feedback or using async/background threads. For simple local tasks, keep scopes minimal.
   - Cache results. If a user filters by author, do not re-scan the file system, just re-filter the already fetched commit list.
2. **Best UX Ever**:
   - Use vibrant and distinct colors (e.g., `Color::Cyan`, `Color::Magenta`, `Color::LightGreen`) to make the interface pop.
   - Provide clear, contextual help menus at the bottom of the screen.
   - Always support **Vim keybindings** (`j` for down, `k` for up, `h` for left/back, `l` for right/enter, `/` for search, `Esc` to cancel).
3. **Robustness**:
   - Do not use `.unwrap()` in production logic unless you are 100% sure it will not panic. Handle errors gracefully and display them in the TUI or log them.
4. **Testing**:
   - If you write new logic, write a standard `#[test]` module at the bottom of the file or in a `tests/` folder. Run `cargo test` to verify.

## How to Test Your Changes
To verify the app builds and runs without error:
1. `cargo check` to ensure types align.
2. `cargo build` for standard compilation.
3. Because it is a TUI, you can't easily capture its output in a non-interactive shell. To test UI flows programmatically, rely on unit tests for `App` state transitions.

Good luck! Build the best UX ever.

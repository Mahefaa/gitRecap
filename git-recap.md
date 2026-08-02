# GitRecap

## Purpose
A Terminal User Interface (TUI) application designed to track and summarize Git activity across multiple projects. It helps users quickly answer the question: "What have I gitted on a specific date?"

## Key Features
- **Filter by Date & Author**: View commits made on a specific date by a specific author.
- **Multiple Sources**: 
  - Add local folders (to find WIPs).
  - Add online sources (to find PRs from preprod to prod).
- **Summary per Project**: Displays a summary of commits grouped by project.
- **Detailed View**: Ability to drill down into specific commits to see the changes made.
- **Push Status**: Indicates whether a local commit has been pushed to a remote repository or not.
- **Export Summary**: Can output a file summarizing the activity in the format:
  ```
  - project1 (commit1 [pushed], commit2 [unpushed])
  - project2 (commit1 [pushed])
  ```

## Tech Stack
- **Language**: Rust
- **UI Framework**: Ratatui (or similar Rust TUI library)

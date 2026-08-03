# gitRecap Complete Documentation

Welcome to **gitRecap**! This is your ultimate guide to using the application, completely updated to cover all the new performance upgrades, persistent filters, and vim-style navigation keys.

## 🚀 Basic Navigation
- `j` / `Down Arrow`: Move cursor down.
- `k` / `Up Arrow`: Move cursor up.
- `g`: Jump to the top of the list.
- `G`: Jump to the bottom of the list.
- `Space`: Toggle a project on/off (excludes it from the right-hand view).
- `s`: Toggle all projects on/off simultaneously.
- `c`: Expand/Collapse the currently selected node (works on projects, dates, or branches).
- `Enter` / `l` / `Right Arrow`: Enter the Details view (jump your cursor to the right-side panel).
- `Esc` / `h` / `Left Arrow`: Leave the Details view and return to the Projects list.
- `v`: Open the **Activity Dashboard** (visualize your commit history in a BarChart).
- `q`: Quit the application.

---

## 🔍 Filters & Persistent State
Filters allow you to slice your Git history effortlessly. 
**All filters are saved persistently to your current Profile.** When you close gitRecap and reopen it, your exact configuration will be preserved.

- `a`: Set **Author filter** (e.g., "John Doe"). Leave empty or type "Any" to clear.
- `b`: Set **Branch filter** (e.g., "main", "feat-auth").
- `d`: Set **Date filter**.
  - This allows you to filter commits strictly within a certain timeframe. 
  - **Natural Language Parsing**: gitRecap understands human language to dynamically resolve dates. Try typing:
    - `"today"`: Commits from midnight today to now.
    - `"yesterday"`: Commits strictly from yesterday.
    - `"3 days"` or `"3d"`: Commits from exactly 3 days ago.
    - `"1 week"` or `"1w"`: Commits from exactly 1 week ago.
    - `"startofmonth"` or `"endofmonth"`: Commits relating to the current month boundaries.
    - `"january"` or `"jan"`: Filter by specific months of the current year.
  - **Absolute date ranges**: Specify precise bounding ranges using the `..` operator:
    - `"2023-01-01..2023-01-31"`: Commits between Jan 1st and Jan 31st.
    - `"since 2023-01-01"`: Commits from Jan 1st to right now.
  - *Magic behavior:* All natural language tags like "today" are saved as string tokens in your config, meaning if you open your profile tomorrow, "today" will be evaluated against tomorrow's date! No manual updating required!
- `/`: **Fuzzy Search / Filter commits dynamically**. Press `/` and start typing to instantly filter the visible commits based on their message.

---

## 🗂️ Profiles & Project Management
Profiles allow you to maintain entirely separate, parallel workspaces for different contexts (e.g., "work", "open-source", "personal"). 
Each profile operates completely independently. When you switch to a profile, gitRecap instantly loads that profile's tracked repositories, its active author filter, its branch filter, its date filter, and even its specific UI toggles.

- `P`: **Switch or Create Profile**. 
  - Type the name of a profile you want to switch to. 
  - If the profile exists, gitRecap will instantly load it without needing a restart.
  - If the profile doesn't exist, gitRecap creates a fresh blank workspace for you.
  - The `default` profile is used if you haven't created one.
- `A`: Add a specific folder to the tracker manually.
- `.` : Add your current working directory to the tracker.
- `p`: Open the visual **File Explorer** to navigate your filesystem and add root project folders.
- `r` / `Delete`: Remove the selected project from the current profile.
- `D`: Remove all projects from the current profile.
- `R`: **Force reload data**. Triggers the multi-threaded Rayon engine to rescan the disk and parse git objects in parallel.
- `e`: Export the current view to `summary.txt` in your working directory.

---

## 📤 Git Integration
- `u`: Run `git push` on the currently selected repository.
- `U`: Run `git push --force` on the currently selected repository (requires confirmation).
- `E`: **Interactive Rewrite**. Press `E` while hovering over any unpushed commit to suspend the app, open your terminal `$EDITOR`, and seamlessly run `git commit --amend`. The app will automatically redraw and reload your updated Git tree when you're done!
- **Unpushed Commits Marker:** Any commit labeled with a red `*` next to it exists on your local machine but has **not yet been pushed** to your remote origin. This makes it incredibly easy to see which repositories need syncing!

---

## 🔍 Commits View
- `j/k`, `Up/Down`: Navigate through commits.
- `l` or `Enter`: Toggle collapse on headers, or **Open the Split-Screen Diff Viewer** when selecting a commit!
- `c`: Collapse/expand all headers in the Commits view globally.
- `/`: Trigger the global Fuzzy Finder to search across all commits instantly.

## 📝 Diff Viewer Mode
- When the split-screen diff viewer is open (triggered by `Enter`), use `PageUp` and `PageDown` to instantly scroll through the syntax-highlighted Git Diff!
- Press `Esc` or `h` to close the Diff Viewer and regain the full commit view screen space.

## ⌨️ Command Mode
Press `:` to enter command mode (like Vim). Commands let you perform sweeping actions instantly:
- `:cp`: **Collapse all projects** (closes all project nodes).
- `:ep`: **Expand all projects**.
- `:cd`: **Collapse all dates**.
- `:ed`: **Expand all dates**.
- `:cb`: **Collapse all branches**.
- `:eb`: **Expand all branches**.
- `:w <filename>` or `:export <filename>`: Export the current view to a specific file.
- `:q!`: Force quit instantly without the confirmation dialog.
- `:sort name` or `:sort activity`: Dynamically sort your project list.
- `:since <time>` or `:until <time>`: Dynamically set a date filter using natural language (e.g. `:since 1 week`).
- `:fetch all`: Spawns a background worker to run `git fetch --all` across every tracked repository concurrently!

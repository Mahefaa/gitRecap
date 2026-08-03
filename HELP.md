# gitRecap Complete Documentation

Welcome to **gitRecap**! This is your ultimate guide to using the application, completely updated to cover all the new performance upgrades, persistent filters, and vim-style navigation keys.

## 🚀 Basic Navigation
- `j` / `Down Arrow`: Move cursor down.
- `k` / `Up Arrow`: Move cursor up.
- `g`: Jump to the top of the list.
- `G`: Jump to the bottom of the list.
- `Space`: Toggle a project on/off (excludes it from the right-hand commit view without removing it).
- `s`: Toggle all projects on/off simultaneously.
- `c`: Expand/Collapse the currently selected node (works on projects, dates, or branches).
- `Enter` / `l` / `Right Arrow`: Enter the Details view (jump your cursor to the right-side panel).
- `Esc` / `h` / `Left Arrow`: Leave the Details view and return to the Projects list.
- `q`: Quit the application.

---

## 🔍 Filters & Persistent State
Filters allow you to slice your Git history effortlessly. 
**All filters are saved persistently to your current Profile.** When you close gitRecap and reopen it, your exact configuration will be preserved.

- `a`: Set **Author filter** (e.g., "John Doe"). Leave empty or type "Any" to clear.
- `b`: Set **Branch filter** (e.g., "main", "feat-auth").
- `d`: Set **Date filter**.
  - gitRecap supports **Natural Language**! You can type:
    - `"today"`
    - `"yesterday"`
    - `"3 days"` or `"3d"`
    - `"1 week"` or `"1w"`
    - `"startofmonth"` or `"endofmonth"`
    - Absolute ranges: `"2023-01-01..2023-01-31"`
  - *Note:* Natural language tags like "today" are evaluated dynamically *every time you open the app*, ensuring your profile is always up to date with real time!
- `/`: **Fuzzy Search / Filter commits dynamically**. Press `/` and start typing to instantly filter the visible commits based on their message.

---

## 🗂️ Profiles & Project Management
Profiles allow you to maintain entirely separate workspaces (e.g., "work" vs. "personal"). Each profile tracks its own repositories, author names, branch filters, and date filters!

- `P`: **Switch or Create Profile**. Type a name. If it exists, gitRecap will instantly load its configuration and repositories. If it doesn't exist, it will create a new blank workspace for you.
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

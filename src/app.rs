use crate::git_utils::{self, BranchCommits};
use chrono::{Local, NaiveDate, TimeZone, Datelike};
use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use tui_input::Input;
use crate::config::{AppConfig, AppProfile};
use crate::file_explorer::FileExplorerState;

pub enum AppMode {
    Normal,
    Details,
    InputAuthor,
    InputDate,
    InputProfile,
    FileExplorer,
    ConfirmPush { force: bool },
    InputAddSource,
    ExplorerJumpPath,
    InputBranch,
    CommitsView,
    InputCommitSearch,
}

pub enum LoadingResult {
    ProjectScanned(ProjectData),
    ScanComplete(Vec<String>),
    ReloadComplete(Vec<ProjectData>),
}

#[derive(Clone)]
pub struct ProjectData {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub branches: Vec<BranchCommits>,
    pub is_expanded: bool,
}

pub struct App {
    pub author_filter: String,
    pub date_start_filter: chrono::DateTime<Local>,
    pub date_end_filter: chrono::DateTime<Local>,
    pub branch_filter: String,
    pub commit_search: String,
    pub sources: Vec<PathBuf>,
    pub projects: Vec<ProjectData>,
    pub project_list_state: ListState,
    pub commit_list_state: ListState,
    pub mode: AppMode,
    pub should_quit: bool,
    pub selected_project_idx: Option<usize>,
    pub input: Input,
    pub file_explorer: FileExplorerState,
    pub known_authors: Vec<String>,
    pub author_list_state: ListState,
    pub config: AppConfig,
    pub current_profile: AppProfile,
    pub flash_message: Option<String>,
    pub is_loading: bool,
    pub loading_rx: Option<std::sync::mpsc::Receiver<LoadingResult>>,
}

impl App {
    pub fn new() -> App {
        let today_start = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
        let today_end = Local::now().date_naive().and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Local).unwrap();
        let mut app = App {
            author_filter: "Any".to_string(),
            date_start_filter: today_start,
            date_end_filter: today_end,
            branch_filter: String::new(),
            commit_search: String::new(),
            sources: vec![PathBuf::from(".")],
            projects: Vec::new(),
            project_list_state: ListState::default(),
            commit_list_state: ListState::default(),
            mode: AppMode::Normal,
            should_quit: false,
            selected_project_idx: None,
            input: Input::default(),
            file_explorer: FileExplorerState::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            known_authors: Vec::new(),
            author_list_state: ListState::default(),
            config: AppConfig::load(),
            current_profile: AppProfile::default(),
            flash_message: None,
            is_loading: false,
            loading_rx: None,
        };
        app.current_profile = app.config.get_active_profile();
        app.sources = app.current_profile.sources.clone();
        
        app.scan_sources();
        app
    }

    pub fn check_background_tasks(&mut self) {
        if let Some(rx) = &self.loading_rx {
            while let Ok(result) = rx.try_recv() {
                match result {
                    LoadingResult::ProjectScanned(proj) => {
                        self.projects.push(proj);
                        if self.projects.len() == 1 && self.selected_project_idx.is_none() {
                            self.project_list_state.select(Some(0));
                            self.selected_project_idx = Some(0);
                        }
                    }
                    LoadingResult::ScanComplete(authors) => {
                        self.known_authors = authors;
                        self.projects.sort_by(|a, b| a.name.cmp(&b.name));
                        self.is_loading = false;
                        self.loading_rx = None;
                        break;
                    }
                    LoadingResult::ReloadComplete(updated_projects) => {
                        self.projects = updated_projects;
                        self.is_loading = false;
                        self.loading_rx = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn add_source(&mut self, path: PathBuf) {
        let mut updated = false;
        if !self.sources.contains(&path) {
            self.sources.push(path.clone());
            self.current_profile.sources = self.sources.clone();
            updated = true;
        }

        let original_len = self.current_profile.removed_projects.len();
        self.current_profile.removed_projects.retain(|p| !p.starts_with(&path));
        
        if self.current_profile.removed_projects.len() < original_len {
            updated = true;
        }

        if updated {
            self.config.update_active_profile(self.current_profile.clone());
            self.scan_sources();
        }
    }

    pub fn scan_sources(&mut self) {
        let sources = self.sources.clone();
        let disabled_projects = self.current_profile.disabled_projects.clone();
        let removed_projects = self.current_profile.removed_projects.clone();
        let author_filter = self.author_filter.clone();
        let branch_filter = self.branch_filter.clone();
        let start_date = self.date_start_filter;
        let end_date = self.date_end_filter;
        let existing_paths: std::collections::HashSet<_> = self.projects.iter().map(|p| p.path.clone()).collect();

        let (tx, rx) = std::sync::mpsc::channel();
        self.is_loading = true;
        self.loading_rx = Some(rx);

        std::thread::spawn(move || {
            let mut authors_set = std::collections::HashSet::new();

            for base_path in &sources {
                if !base_path.exists() { continue; }
                let repos = crate::git_utils::find_git_repos(base_path);
                for repo_path in repos {
                    if existing_paths.contains(&repo_path) { continue; }
                    let is_removed = removed_projects.contains(&repo_path);
                    if !is_removed {
                        let name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        let enabled = !disabled_projects.contains(&repo_path);
                        
                        let mut branches = Vec::new();
                        if enabled {
                            if let Ok(mut b) = crate::git_utils::get_commits(&repo_path, &author_filter, &branch_filter, start_date, end_date) {
                                for br in &mut b {
                                    br.commits.sort_by_key(|c| std::cmp::Reverse(c.date));
                                }
                                branches = b;
                            }
                        }

                        let _ = tx.send(LoadingResult::ProjectScanned(ProjectData {
                            name,
                            path: repo_path.clone(),
                            enabled,
                            branches,
                            is_expanded: true,
                        }));
                        
                        for author in crate::git_utils::get_recent_authors(&repo_path) {
                            authors_set.insert(author);
                        }
                    }
                }
            }
            
            let mut sorted_authors: Vec<String> = authors_set.into_iter().collect();
            sorted_authors.sort();
            let _ = tx.send(LoadingResult::ScanComplete(sorted_authors));
        });
    }

    pub fn reload_data(&mut self) {
        let mut projects = self.projects.clone();
        let author_filter = self.author_filter.clone();
        let branch_filter = self.branch_filter.clone();
        let start_date = self.date_start_filter;
        let end_date = self.date_end_filter;

        let (tx, rx) = std::sync::mpsc::channel();
        self.is_loading = true;
        self.loading_rx = Some(rx);

        std::thread::spawn(move || {
            for proj in &mut projects {
                proj.branches.clear();
                if proj.enabled {
                    if let Ok(mut branches) = crate::git_utils::get_commits(&proj.path, &author_filter, &branch_filter, start_date, end_date) {
                        for b in &mut branches {
                            b.commits.sort_by_key(|c| std::cmp::Reverse(c.date));
                        }
                        proj.branches = branches;
                    }
                }
            }
            let _ = tx.send(LoadingResult::ReloadComplete(projects));
        });
    }

    pub fn toggle_project(&mut self) {
        if let Some(idx) = self.selected_project_idx
            && idx < self.projects.len() {
                self.projects[idx].enabled = !self.projects[idx].enabled;
                
                // Update profile
                let path = self.projects[idx].path.clone();
                if self.projects[idx].enabled {
                    self.current_profile.disabled_projects.retain(|p| p != &path);
                } else {
                    if !self.current_profile.disabled_projects.contains(&path) {
                        self.current_profile.disabled_projects.push(path);
                    }
                }
                self.config.update_active_profile(self.current_profile.clone());
                
                self.reload_data();
            }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(idx) = self.selected_project_idx {
            if idx < self.projects.len() {
                self.projects[idx].is_expanded = !self.projects[idx].is_expanded;
            }
        }
    }

    pub fn toggle_all_projects(&mut self) {
        if self.projects.is_empty() { return; }
        let all_enabled = self.projects.iter().all(|p| p.enabled);
        let new_state = !all_enabled;
        for proj in &mut self.projects {
            proj.enabled = new_state;
            let path = proj.path.clone();
            if new_state {
                self.current_profile.disabled_projects.retain(|p| p != &path);
            } else {
                if !self.current_profile.disabled_projects.contains(&path) {
                    self.current_profile.disabled_projects.push(path);
                }
            }
        }
        self.config.update_active_profile(self.current_profile.clone());
        self.reload_data();
    }

    pub fn remove_project(&mut self) {
        if let Some(idx) = self.selected_project_idx
            && idx < self.projects.len() {
                let removed_path = self.projects[idx].path.clone();
                self.current_profile.removed_projects.push(removed_path);
                self.config.update_active_profile(self.current_profile.clone());
                
                self.projects.remove(idx);
                if self.projects.is_empty() {
                    self.project_list_state.select(None);
                    self.selected_project_idx = None;
                } else {
                    let new_idx = idx.min(self.projects.len() - 1);
                    self.project_list_state.select(Some(new_idx));
                    self.selected_project_idx = Some(new_idx);
                }
                self.commit_list_state.select(None);
            }
    }

    pub fn remove_all_projects(&mut self) {
        if self.projects.is_empty() { return; }
        for proj in &self.projects {
            let path = proj.path.clone();
            if !self.current_profile.removed_projects.contains(&path) {
                self.current_profile.removed_projects.push(path);
            }
        }
        self.config.update_active_profile(self.current_profile.clone());
        self.projects.clear();
        self.project_list_state.select(None);
        self.selected_project_idx = None;
        self.commit_list_state.select(None);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_item(&mut self) {
        match self.mode {
            AppMode::Normal => {
                let i = match self.project_list_state.selected() {
                    Some(i) => if i >= self.projects.len().saturating_sub(1) { 0 } else { i + 1 },
                    None => 0,
                };
                if !self.projects.is_empty() {
                    self.project_list_state.select(Some(i));
                    self.selected_project_idx = Some(i);
                    self.commit_list_state.select(None);
                }
            }
            AppMode::CommitsView => {
                let i = match self.commit_list_state.selected() {
                    Some(i) => i.saturating_add(1),
                    None => 0,
                };
                self.commit_list_state.select(Some(i));
            }
            AppMode::Details => {
                if let Some(proj_idx) = self.selected_project_idx {
                    if proj_idx < self.projects.len() {
                        let mut total_rendered = 0;
                        const LIMIT: usize = 1000;
                        if !self.projects[proj_idx].branches.is_empty() {
                            total_rendered += 1;
                        }
                        for b in &self.projects[proj_idx].branches {
                            if total_rendered >= LIMIT { break; }
                            total_rendered += 1;
                            total_rendered += b.commits.len();
                        }
                        if total_rendered > LIMIT {
                            total_rendered = LIMIT + 1;
                        }
                        if total_rendered > 0 {
                            let i = match self.commit_list_state.selected() {
                                Some(i) => if i >= total_rendered.saturating_sub(1) { 0 } else { i + 1 },
                                None => 0,
                            };
                            self.commit_list_state.select(Some(i));
                        }
                    }
                }
            }
            AppMode::InputAuthor => {
                let filtered = self.get_filtered_authors();
                if !filtered.is_empty() {
                    let i = match self.author_list_state.selected() {
                        Some(i) => if i >= filtered.len().saturating_sub(1) { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.author_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    pub fn previous_item(&mut self) {
        match self.mode {
            AppMode::Normal => {
                let i = match self.project_list_state.selected() {
                    Some(i) => if i == 0 { self.projects.len().saturating_sub(1) } else { i - 1 },
                    None => 0,
                };
                if !self.projects.is_empty() {
                    self.project_list_state.select(Some(i));
                    self.selected_project_idx = Some(i);
                    self.commit_list_state.select(None);
                }
            }
            AppMode::Details => {
                if let Some(proj_idx) = self.selected_project_idx {
                    if proj_idx < self.projects.len() {
                        let mut total_rendered = 0;
                        const LIMIT: usize = 1000;
                        if !self.projects[proj_idx].branches.is_empty() {
                            total_rendered += 1;
                        }
                        for b in &self.projects[proj_idx].branches {
                            if total_rendered >= LIMIT { break; }
                            total_rendered += 1;
                            total_rendered += b.commits.len();
                        }
                        if total_rendered > LIMIT {
                            total_rendered = LIMIT + 1;
                        }
                        if total_rendered > 0 {
                            let i = match self.commit_list_state.selected() {
                                Some(i) => if i == 0 { total_rendered.saturating_sub(1) } else { i - 1 },
                                None => 0,
                            };
                            self.commit_list_state.select(Some(i));
                        }
                    }
                }
            }
            AppMode::InputAuthor => {
                let filtered = self.get_filtered_authors();
                if !filtered.is_empty() {
                    let i = match self.author_list_state.selected() {
                        Some(i) => if i == 0 { filtered.len().saturating_sub(1) } else { i - 1 },
                        None => 0,
                    };
                    self.author_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    pub fn enter_details(&mut self) {
        if let Some(proj_idx) = self.selected_project_idx {
            if proj_idx < self.projects.len() {
                let total_commits: usize = self.projects[proj_idx].branches.iter().map(|b| b.commits.len()).sum();
                if total_commits > 0 {
                    self.mode = AppMode::Details;
                    self.commit_list_state.select(Some(0));
                }
            }
        }
    }

    pub fn leave_details(&mut self) {
        self.mode = AppMode::Normal;
        self.commit_list_state.select(None);
    }

    pub fn enter_input_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        self.input.reset();
        match self.mode {
            AppMode::InputAuthor => {
                self.input = self.input.clone().with_value(if self.author_filter == "Any" { String::new() } else { self.author_filter.clone() });
                self.author_list_state.select(Some(0));
            },
            AppMode::InputDate => {
                let display = format!("{}..{}", self.date_start_filter.format("%Y-%m-%d"), self.date_end_filter.format("%Y-%m-%d"));
                self.input = self.input.clone().with_value(display);
            },
            AppMode::InputBranch => {
                self.input = self.input.clone().with_value(self.branch_filter.clone());
            },
            AppMode::InputCommitSearch => {
                self.input = self.input.clone().with_value(self.commit_search.clone());
            },
            AppMode::InputProfile => {
                self.input = self.input.clone().with_value(self.current_profile.name.clone());
            },
            AppMode::FileExplorer => {
                self.file_explorer.load_directory();
            }
            AppMode::InputAddSource => {
                self.input = self.input.clone().with_value("".to_string());
            }
            AppMode::ExplorerJumpPath => {
                self.input = self.input.clone().with_value(self.file_explorer.current_path.to_string_lossy().to_string());
            }
            _ => {}
        }
    }

    pub fn get_filtered_authors(&self) -> Vec<String> {
        let input_val = self.input.value().to_lowercase();
        let mut list = vec!["Any".to_string()];
        for a in &self.known_authors {
            if a.to_lowercase().contains(&input_val) && !list.contains(a) {
                list.push(a.clone());
            }
        }
        list
    }

    pub fn submit_input(&mut self) {
        match self.mode {
            AppMode::InputAuthor => {
                let filtered = self.get_filtered_authors();
                if let Some(idx) = self.author_list_state.selected() {
                    if idx < filtered.len() {
                        self.author_filter = filtered[idx].clone();
                    }
                } else {
                    self.author_filter = if self.input.value().is_empty() { "Any".to_string() } else { self.input.value().to_string() };
                }
                self.reload_data();
            }
            AppMode::InputDate => {
                if let Some((start, end)) = parse_date_input(self.input.value()) {
                    self.date_start_filter = start;
                    self.date_end_filter = end;
                    self.reload_data();
                }
            }
            AppMode::InputBranch => {
                self.branch_filter = self.input.value().to_string();
                self.reload_data();
            }
            AppMode::InputCommitSearch => {
                self.commit_search = self.input.value().to_lowercase();
                self.mode = AppMode::CommitsView;
                return;
            }
            AppMode::InputProfile => {
                let name = self.input.value().to_string();
                if !name.is_empty() {
                    self.current_profile = self.config.switch_profile(&name);
                    self.sources = self.current_profile.sources.clone();
                    self.projects.clear();
                    self.scan_sources();
                    self.reload_data();
                }
            }
            AppMode::InputAddSource => {
                let raw_path = self.input.value().trim().replace("\\", "/");
                let p = PathBuf::from(raw_path);
                if p.exists() && p.is_dir() {
                    self.add_source(p);
                    self.flash_message = Some("Source path added!".to_string());
                } else {
                    self.flash_message = Some(format!("Invalid directory: {}", self.input.value()));
                }
            }
            AppMode::ExplorerJumpPath => {
                let raw_path = self.input.value().trim().replace("\\", "/");
                let p = PathBuf::from(raw_path);
                if p.exists() && p.is_dir() {
                    self.file_explorer.current_path = p;
                    self.file_explorer.load_directory();
                    self.mode = AppMode::FileExplorer;
                    return; // Don't go back to normal
                } else {
                    self.flash_message = Some(format!("Invalid directory: {}", self.input.value()));
                    self.mode = AppMode::FileExplorer;
                    return;
                }
            }
            _ => {}
        }
        self.mode = AppMode::Normal;
    }

    pub fn cancel_input(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn export_summary(&self, path: &str) -> Result<(), Box<dyn Error>> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        for proj in &self.projects {
            if !proj.enabled || proj.branches.is_empty() {
                continue;
            }
            
            let mut has_commits = false;
            let mut proj_output = format!("- {}(", proj.name);
            
            let mut branch_strings = Vec::new();
            for branch in &proj.branches {
                if branch.commits.is_empty() { continue; }
                has_commits = true;
                
                let commit_summaries: Vec<String> = branch.commits
                    .iter()
                    .map(|c| {
                        format!("{} {}", c.id.chars().take(7).collect::<String>(), c.message.chars().take(30).collect::<String>())
                    })
                    .collect();
                
                branch_strings.push(format!("{}({})", branch.name, commit_summaries.join(", ")));
            }
            
            if has_commits {
                proj_output.push_str(&branch_strings.join(", "));
                proj_output.push_str(")\n");
                file.write_all(proj_output.as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn push_project(&mut self, force: bool) {
        if let Some(idx) = self.selected_project_idx {
            if idx < self.projects.len() {
                let path = &self.projects[idx].path;
                let mut cmd = std::process::Command::new("git");
                cmd.arg("push");
                if force {
                    cmd.arg("--force");
                }
                cmd.current_dir(path);
                cmd.env("GIT_TERMINAL_PROMPT", "0");
                
                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            self.flash_message = Some("Push successful!".to_string());
                        } else {
                            let err = String::from_utf8_lossy(&output.stderr).to_string();
                            self.flash_message = Some(format!("Push failed: {}", err));
                        }
                    }
                    Err(e) => {
                        self.flash_message = Some(format!("Failed to run git push: {}", e));
                    }
                }
            }
        }
    }
}

fn parse_date_input(input: &str) -> Option<(chrono::DateTime<Local>, chrono::DateTime<Local>)> {
    let now = Local::now();
    let input = input.trim().to_lowercase();
    
    let day_range = |d: NaiveDate| -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
        let start = d.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
        let end = d.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Local).unwrap();
        (start, end)
    };

    if input == "startofday" {
        return Some((day_range(now.date_naive()).0, now));
    } else if input == "endofday" {
        return Some((now, day_range(now.date_naive()).1));
    } else if input == "today" {
        return Some(day_range(now.date_naive()));
    } else if input == "startofmonth" {
        let d = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        return Some((day_range(d).0, now));
    } else if input == "endofmonth" {
        let next_month = if now.month() == 12 { 1 } else { now.month() + 1 };
        let next_year = if now.month() == 12 { now.year() + 1 } else { now.year() };
        let d = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap().pred_opt().unwrap();
        return Some((now, day_range(d).1));
    }
    
    if input.starts_with("month(") && input.ends_with(')') {
        let inner = &input[6..input.len()-1];
        let mut year = now.year();
        let mut month = now.month();
        let chars: String = inner.chars().filter(|c| c.is_alphabetic()).collect();
        let nums: String = inner.chars().filter(|c| c.is_numeric()).collect();
        
        if !nums.is_empty() {
            if let Ok(y) = nums.parse::<i32>() {
                if y < 100 { year = 2000 + y; } else { year = y; }
            }
        }
        
        let m_str = chars.as_str();
        if !m_str.is_empty() {
            match m_str {
                "jan" | "january" => month = 1,
                "feb" | "february" => month = 2,
                "mar" | "march" => month = 3,
                "apr" | "april" => month = 4,
                "may" => month = 5,
                "jun" | "june" => month = 6,
                "jul" | "july" => month = 7,
                "aug" | "august" => month = 8,
                "sep" | "september" => month = 9,
                "oct" | "october" => month = 10,
                "nov" | "november" => month = 11,
                "dec" | "december" => month = 12,
                _ => {}
            }
        } else if !nums.is_empty() {
            if let Ok(m) = nums.parse::<u32>() {
                if (1..=12).contains(&m) {
                    month = m;
                    year = now.year();
                }
            }
        }
        
        if let Some(start_d) = NaiveDate::from_ymd_opt(year, month, 1) {
            let next_m = if month == 12 { 1 } else { month + 1 };
            let next_y = if month == 12 { year + 1 } else { year };
            if let Some(end_d) = NaiveDate::from_ymd_opt(next_y, next_m, 1).unwrap().pred_opt() {
                return Some((day_range(start_d).0, day_range(end_d).1));
            }
        }
    }

    let parts: Vec<&str> = input.split("..").collect();
    if parts.len() == 2 {
        let d1 = chrono::NaiveDateTime::parse_from_str(parts[0].trim(), "%Y-%m-%d %H:%M:%S")
            .map(|d| d.and_local_timezone(Local).unwrap())
            .or_else(|_| NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d").map(|d| day_range(d).0));
            
        let d2 = chrono::NaiveDateTime::parse_from_str(parts[1].trim(), "%Y-%m-%d %H:%M:%S")
            .map(|d| d.and_local_timezone(Local).unwrap())
            .or_else(|_| NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d").map(|d| day_range(d).1));
            
        if let (Ok(start), Ok(end)) = (d1, d2) {
            return Some((start, end));
        }
    } else {
        let d1 = chrono::NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M:%S")
            .map(|d| d.and_local_timezone(Local).unwrap())
            .or_else(|_| NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d").map(|d| day_range(d).0));
        
        if let Ok(start) = d1 {
            let end = NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d").map(|d| day_range(d).1).unwrap_or(start);
            return Some((start, end));
        }
    }
    None
}


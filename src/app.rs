use crate::git_utils::DateGroup;
use chrono::{Local, NaiveDate, Datelike};
use ratatui::widgets::ListState;
use ratatui::layout::Rect;
use std::path::PathBuf;
use tui_input::Input;
use crate::config::{AppConfig, AppProfile};
use crate::file_explorer::FileExplorerState;

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Details,
    InputAuthor,
    InputDate,
    InputProfile,
    FileExplorer,
    ConfirmPush { force: bool },
    InputAddSource,
    InputExportPath,
    ExplorerJumpPath,
    InputBranch,
    CommitsView,
    Search(SearchTarget),
    Command,
    ConfirmQuit,
    Help,
    Dashboard,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SearchTarget {
    Projects,
    Commits,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TimeResolution {
    Day,
    Week,
    Month,
}

pub enum LoadingResult {
    ProjectScanned(ProjectData),
    ScanComplete(Vec<String>),
    ReloadComplete(Vec<ProjectData>),
    FetchComplete,
    ExportComplete(Result<String, String>),
}

#[derive(Clone)]
pub struct ProjectData {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub dates: Vec<DateGroup>,
    pub is_expanded: bool,
    pub last_commit_info: String,
}

#[derive(Clone)]
pub struct TimelineDate {
    pub date: String,
    pub is_expanded: bool,
    pub projects: Vec<TimelineProject>,
}

#[derive(Clone)]
pub struct TimelineProject {
    pub name: String,
    pub is_expanded: bool,
    pub authors: Vec<TimelineAuthor>,
}

#[derive(Clone)]
pub struct TimelineBranch {
    pub name: String,
    pub is_expanded: bool,
    pub commits: Vec<crate::git_utils::GitCommit>,
}

#[derive(Clone)]
pub struct TimelineAuthor {
    pub name: String,
    pub is_expanded: bool,
    pub branches: Vec<TimelineBranch>,
}

pub struct App {
    pub author_filter: String,
    pub date_start_filter: chrono::DateTime<Local>,
    pub date_end_filter: chrono::DateTime<Local>,
    pub branch_filter: String,
    pub previous_mode: Option<AppMode>,
    pub projects_area: Option<Rect>,
    pub commits_area: Option<Rect>,
    pub commit_list_map: Vec<(usize, Option<usize>, Option<usize>, Option<usize>, Option<usize>)>,
    
    pub dashboard_list_state: ratatui::widgets::TableState,
    pub dashboard_resolution: TimeResolution,
    
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
    pub hide_zero_commits: bool,
    pub diff_content: Option<ratatui::text::Text<'static>>,
    pub diff_scroll: u16,
    pub timeline: Vec<TimelineDate>,
    pub fuzzy_filter: String,
    
    pub project_search: String,
    pub visible_projects: Vec<usize>,
    pub last_search_typing: Option<std::time::Instant>,
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
            previous_mode: None,
            projects_area: None,
            commits_area: None,
            commit_list_map: Vec::new(),
            
            dashboard_list_state: ratatui::widgets::TableState::default(),
            dashboard_resolution: TimeResolution::Day,
            
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
            hide_zero_commits: false,
            diff_content: None,
            diff_scroll: 0,
            timeline: Vec::new(),
            fuzzy_filter: String::new(),
            project_search: String::new(),
            visible_projects: Vec::new(),
            last_search_typing: None,
        };
        app.current_profile = app.config.get_active_profile();
        app.sources = app.current_profile.sources.clone();
        
        app.author_filter = app.current_profile.author_filter.clone();
        app.branch_filter = app.current_profile.branch_filter.clone();
        app.hide_zero_commits = app.current_profile.hide_zero_commits;
        if let Some((start, end)) = crate::app::parse_date_input(&app.current_profile.date_filter) {
            app.date_start_filter = start;
            app.date_end_filter = end;
        }
        
        app.scan_sources();
        app
    }

    pub fn trigger_export(&mut self) {
        self.mode = AppMode::InputExportPath;
        let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
        let default_filename = format!("{}.md", date_str);
        let initial_value = if let Some(path) = &self.config.default_export_path {
            let path = std::path::PathBuf::from(path).join(&default_filename);
            path.to_string_lossy().into_owned()
        } else {
            default_filename
        };
        self.input = tui_input::Input::default().with_value(initial_value);
    }

    pub fn check_background_tasks(&mut self) -> bool {
        let mut updated = false;
        let mut needs_visibility_update = false;
        if let Some(rx) = &self.loading_rx {
            while let Ok(result) = rx.try_recv() {
                updated = true;
                match result {
                    LoadingResult::ProjectScanned(proj) => {
                        self.projects.push(proj);
                        needs_visibility_update = true;
                        if self.projects.len() == 1 && self.selected_project_idx.is_none() {
                            self.project_list_state.select(Some(0));
                            self.selected_project_idx = Some(0);
                        }
                    }
                    LoadingResult::ScanComplete(authors) => {
                        self.known_authors = authors;
                        self.projects.sort_by(|a, b| a.name.cmp(&b.name));
                        needs_visibility_update = true;
                        self.is_loading = false;
                        self.loading_rx = None;
                        self.build_timeline();
                        break;
                    }
                    LoadingResult::ReloadComplete(updated_projects) => {
                        self.projects = updated_projects;
                        needs_visibility_update = true;
                        self.is_loading = false;
                        self.loading_rx = None;
                        self.build_timeline();
                        break;
                    }
                    LoadingResult::FetchComplete => {
                        self.is_loading = false;
                        self.loading_rx = None;
                        self.flash_message = Some("Fetch complete!".to_string());
                        self.reload_data(); // reload after fetching
                        break;
                    }
                    LoadingResult::ExportComplete(res) => {
                        self.is_loading = false;
                        self.loading_rx = None;
                        match res {
                            Ok(msg) => self.flash_message = Some(msg),
                            Err(e) => self.flash_message = Some(format!("Export failed: {}", e)),
                        }
                        break;
                    }
                }
            }
        }
        if needs_visibility_update {
            self.update_visible_projects();
        }
        updated
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
    
    pub fn update_visible_projects(&mut self) {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let query = self.project_search.to_lowercase();
        let use_filter = !query.is_empty();
        
        let mut visible = Vec::new();
        for (idx, p) in self.projects.iter().enumerate() {
            if use_filter {
                use fuzzy_matcher::FuzzyMatcher;
                if matcher.fuzzy_match(&p.name, &query).is_none() {
                    continue;
                }
            }
            visible.push(idx);
        }
        self.visible_projects = visible;
        
        if let Some(selected) = self.selected_project_idx {
            if !self.visible_projects.contains(&selected) {
                self.selected_project_idx = self.visible_projects.first().copied();
                if let Some(s) = self.selected_project_idx {
                    if let Some(pos) = self.visible_projects.iter().position(|&x| x == s) {
                        self.project_list_state.select(Some(pos));
                    }
                } else {
                    self.project_list_state.select(None);
                }
            }
        }
    }
    
    pub fn apply_search_debounce(&mut self) {
        if let AppMode::Search(target) = self.mode {
            let val = self.input.value().to_string();
            match target {
                SearchTarget::Projects => {
                    if self.project_search != val {
                        self.project_search = val;
                        self.update_visible_projects();
                    }
                },
                SearchTarget::Commits => {
                    if self.fuzzy_filter != val {
                        self.fuzzy_filter = val;
                        self.build_timeline();
                    }
                }
            }
        }
    }

    pub fn build_timeline(&mut self) {
        use std::collections::{BTreeMap, HashMap};
        let mut agg: BTreeMap<String, BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<crate::git_utils::GitCommit>>>>> = BTreeMap::new();
        
        let mut old_date_expanded = HashMap::new();
        let mut old_proj_expanded = HashMap::new();
        let mut old_author_expanded = HashMap::new();
        let mut old_branch_expanded = HashMap::new();
        for d in &self.timeline {
            old_date_expanded.insert(d.date.clone(), d.is_expanded);
            for p in &d.projects {
                old_proj_expanded.insert(format!("{}|{}", d.date, p.name), p.is_expanded);
                for a in &p.authors {
                    old_author_expanded.insert(format!("{}|{}|{}", d.date, p.name, a.name), a.is_expanded);
                    for b in &a.branches {
                        old_branch_expanded.insert(format!("{}|{}|{}|{}", d.date, p.name, a.name, b.name), b.is_expanded);
                    }
                }
            }
        }

        let projects_to_show: Vec<_> = self.projects.iter().filter(|p| p.enabled).collect();
        let use_filter = !self.fuzzy_filter.is_empty();
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let query = self.fuzzy_filter.to_lowercase();

        for proj in projects_to_show {
            for date_group in &proj.dates {
                let mut proj_map_temp = BTreeMap::new();
                for branch in &date_group.branches {
                    for commit in &branch.commits {
                        if use_filter {
                            use fuzzy_matcher::FuzzyMatcher;
                            let target = format!("{} {} {} {}", proj.name, commit.author, commit.id, commit.message);
                            if matcher.fuzzy_match(&target, &query).is_none() {
                                continue;
                            }
                        }
                        let author_map = proj_map_temp.entry(commit.author.clone()).or_insert_with(BTreeMap::new);
                        let commit_list = author_map.entry(branch.name.clone()).or_insert_with(Vec::new);
                        commit_list.push(commit.clone());
                    }
                }
                if !proj_map_temp.is_empty() {
                    let proj_map = agg.entry(date_group.date.clone()).or_insert_with(BTreeMap::new);
                    let author_map = proj_map.entry(proj.name.clone()).or_insert_with(BTreeMap::new);
                    for (author, branches) in proj_map_temp {
                        let branch_map = author_map.entry(author).or_insert_with(BTreeMap::new);
                        for (branch, commits) in branches {
                            let commit_list = branch_map.entry(branch).or_insert_with(Vec::new);
                            commit_list.extend(commits);
                        }
                    }
                }
            }
        }
        
        let mut timeline = Vec::new();
        for (date, projects) in agg.into_iter().rev() {
            let mut tl_projects = Vec::new();
            for (proj_name, authors) in projects {
                let mut tl_authors = Vec::new();
                for (author_name, branches) in authors {
                    let mut tl_branches = Vec::new();
                    for (branch_name, mut commits) in branches {
                        commits.sort_by(|a, b| b.date.cmp(&a.date));
                        let key = format!("{}|{}|{}|{}", date, proj_name, author_name, branch_name);
                        let is_expanded = *old_branch_expanded.get(&key).unwrap_or(&true);
                        tl_branches.push(TimelineBranch {
                            name: branch_name,
                            is_expanded,
                            commits,
                        });
                    }
                    
                    let key = format!("{}|{}|{}", date, proj_name, author_name);
                    let is_expanded = *old_author_expanded.get(&key).unwrap_or(&true);
                    
                    tl_authors.push(TimelineAuthor {
                        name: author_name,
                        is_expanded,
                        branches: tl_branches,
                    });
                }
                
                let key = format!("{}|{}", date, proj_name);
                let is_expanded = *old_proj_expanded.get(&key).unwrap_or(&true);
                
                tl_projects.push(TimelineProject {
                    name: proj_name,
                    is_expanded,
                    authors: tl_authors,
                });
            }
            
            let is_expanded = *old_date_expanded.get(&date).unwrap_or(&true);
            timeline.push(TimelineDate {
                date,
                is_expanded,
                projects: tl_projects,
            });
        }
        
        self.timeline = timeline;
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

            let repos: Vec<_> = sources.iter()
                .filter(|p| p.exists())
                .flat_map(|p| crate::git_utils::find_git_repos(p))
                .filter(|p| !existing_paths.contains(p) && !removed_projects.contains(p))
                .collect();

            use rayon::prelude::*;
            let authors_results: Vec<Vec<String>> = repos.into_par_iter().map(|repo_path| {
                let name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let enabled = !disabled_projects.contains(&repo_path);
                
                let mut dates = Vec::new();
                if enabled {
                    if let Ok(b) = crate::git_utils::get_commits(&repo_path, &author_filter, &branch_filter, start_date, end_date) {
                        dates = b;
                    }
                }

                let last_commit_info = crate::git_utils::get_last_commit_info(&repo_path).unwrap_or_else(|| "(0 commits)".to_string());
                let _ = tx.send(LoadingResult::ProjectScanned(ProjectData {
                    name,
                    path: repo_path.clone(),
                    enabled,
                    dates,
                    is_expanded: true,
                    last_commit_info,
                }));
                
                crate::git_utils::get_recent_authors(&repo_path)
            }).collect();

            for set in authors_results {
                for author in set {
                    authors_set.insert(author);
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
            use rayon::prelude::*;
            projects.par_iter_mut().for_each(|proj| {
                proj.dates.clear();
                if proj.enabled {
                    if let Ok(dates) = crate::git_utils::get_commits(&proj.path, &author_filter, &branch_filter, start_date, end_date) {
                        proj.dates = dates;
                    }
                }
            });
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
        if let Some(i) = self.project_list_state.selected() {
            if i < self.projects.len() {
                self.projects[i].is_expanded = !self.projects[i].is_expanded;
            }
        }
    }

    pub fn toggle_expand_from_commits_view(&mut self) {
        if let Some(i) = self.commit_list_state.selected() {
            if i < self.commit_list_map.len() {
                let (date_idx, proj_idx, author_idx, branch_idx, commit_idx) = self.commit_list_map[i];
                if commit_idx.is_some() {
                    // Do nothing for commits
                } else if let Some(b_idx) = branch_idx {
                    if let Some(a_idx) = author_idx {
                        if let Some(p_idx) = proj_idx {
                            self.timeline[date_idx].projects[p_idx].authors[a_idx].branches[b_idx].is_expanded = !self.timeline[date_idx].projects[p_idx].authors[a_idx].branches[b_idx].is_expanded;
                        }
                    }
                } else if let Some(a_idx) = author_idx {
                    if let Some(p_idx) = proj_idx {
                        self.timeline[date_idx].projects[p_idx].authors[a_idx].is_expanded = !self.timeline[date_idx].projects[p_idx].authors[a_idx].is_expanded;
                    }
                } else if let Some(p_idx) = proj_idx {
                    self.timeline[date_idx].projects[p_idx].is_expanded = !self.timeline[date_idx].projects[p_idx].is_expanded;
                } else {
                    self.timeline[date_idx].is_expanded = !self.timeline[date_idx].is_expanded;
                }
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
        self.mode = AppMode::ConfirmQuit;
    }

    pub fn execute_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).unwrap_or(&"").trim();

        match command {
            "cp" => { for d in &mut self.timeline { for p in &mut d.projects { p.is_expanded = false; } } }
            "ep" => { for d in &mut self.timeline { for p in &mut d.projects { p.is_expanded = true; } } }
            "cd" => { for d in &mut self.timeline { d.is_expanded = false; } }
            "ed" => { for d in &mut self.timeline { d.is_expanded = true; } }
            "cb" | "ca" => {
                for d in &mut self.timeline {
                    for p in &mut d.projects {
                        for a in &mut p.authors { a.is_expanded = false; }
                    }
                }
            }
            "eb" | "ea" => {
                for d in &mut self.timeline {
                    for p in &mut d.projects {
                        for a in &mut p.authors { a.is_expanded = true; }
                    }
                }
            }
            "c" => {
                for d in &mut self.timeline {
                    d.is_expanded = false;
                    for p in &mut d.projects {
                        p.is_expanded = false;
                        for a in &mut p.authors { a.is_expanded = false; }
                    }
                }
            }
            "e" => {
                for d in &mut self.timeline {
                    d.is_expanded = true;
                    for p in &mut d.projects {
                        p.is_expanded = true;
                        for a in &mut p.authors { a.is_expanded = true; }
                    }
                }
            }
            "noprank" => {
                self.config.no_prank = true;
                self.config.save();
                self.flash_message = Some("Prank disabled. Theme reverted to normal.".to_string());
            }
            "prank" => {
                self.config.no_prank = false;
                self.config.save();
                self.flash_message = Some("Pride mode activated! 🌈".to_string());
            }
            "q!" => { self.should_quit = true; }
            "w" | "export" => {
                if arg.is_empty() {
                    self.flash_message = Some("Filename required for export".to_string());
                } else {
                    let p = std::path::Path::new(arg);
                    if let Some(parent) = p.parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() {
                            self.config.default_export_path = Some(parent_str);
                            self.config.save();
                        }
                    }
                    self.export_summary(arg);
                }
            }
            "sort" => {
                match arg {
                    "name" => {
                        self.projects.sort_by(|a, b| a.name.cmp(&b.name));
                        self.flash_message = Some("Sorted by name".to_string());
                    }
                    "activity" | "commits" => {
                        self.projects.sort_by(|a, b| {
                            let a_count: usize = a.dates.iter().flat_map(|d| d.branches.iter().map(|b| b.commits.len())).sum();
                            let b_count: usize = b.dates.iter().flat_map(|d| d.branches.iter().map(|b| b.commits.len())).sum();
                            b_count.cmp(&a_count)
                        });
                        self.flash_message = Some("Sorted by activity".to_string());
                    }
                    _ => self.flash_message = Some("Sort criteria must be 'name', 'activity', or 'commits'".to_string()),
                }
            }
            "since" | "until" => {
                if arg.is_empty() {
                    self.flash_message = Some(format!("Date required for {}", command));
                } else {
                    let date_str = if command == "since" {
                        format!("{}..today", arg)
                    } else {
                        format!("startofmonth..{}", arg)
                    };
                    
                    if let Some((start, end)) = parse_date_input(&date_str) {
                        self.date_start_filter = start;
                        self.date_end_filter = end;
                        self.current_profile.date_filter = date_str;
                        self.config.update_active_profile(self.current_profile.clone());
                        self.reload_data();
                        self.flash_message = Some(format!("Date filter updated: {}", self.current_profile.date_filter));
                    } else {
                        self.flash_message = Some("Invalid date format".to_string());
                    }
                }
            }
            "fetch" => {
                if arg == "all" {
                    let paths: Vec<PathBuf> = self.projects.iter().filter(|p| p.enabled).map(|p| p.path.clone()).collect();
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        use rayon::prelude::*;
                        paths.par_iter().for_each(|p| {
                            let _ = std::process::Command::new("git").arg("fetch").arg("--all").current_dir(p).output();
                        });
                        let _ = tx.send(crate::app::LoadingResult::FetchComplete);
                    });
                    self.is_loading = true;
                    self.loading_rx = Some(rx);
                    self.flash_message = Some("Fetching all enabled projects in background...".to_string());
                } else {
                    self.flash_message = Some("Unknown fetch command. Try ':fetch all'".to_string());
                }
            }
            "pull" => {
                if arg == "all" {
                    let paths: Vec<PathBuf> = self.projects.iter().filter(|p| p.enabled).map(|p| p.path.clone()).collect();
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        use rayon::prelude::*;
                        paths.par_iter().for_each(|p| {
                            let _ = std::process::Command::new("git").arg("pull").current_dir(p).output();
                        });
                        let _ = tx.send(crate::app::LoadingResult::FetchComplete);
                    });
                    self.is_loading = true;
                    self.loading_rx = Some(rx);
                    self.flash_message = Some("Pulling all enabled projects in background...".to_string());
                } else {
                    self.flash_message = Some("Unknown pull command. Try ':pull all'".to_string());
                }
            }
            _ => { self.flash_message = Some(format!("Unknown command: :{}", command)); }
        }
    }

    pub fn next_item(&mut self) {
        match self.mode {
            AppMode::Normal | AppMode::Search(SearchTarget::Projects) => {
                if !self.visible_projects.is_empty() {
                    let limit = self.visible_projects.len().saturating_sub(1);
                    let i = match self.project_list_state.selected() {
                        Some(i) => if i >= limit { 0 } else { i + 1 },
                        None => 0,
                    };
                    self.project_list_state.select(Some(i));
                    self.selected_project_idx = Some(self.visible_projects[i]);
                    self.commit_list_state.select(None);
                }
            }
            AppMode::CommitsView | AppMode::Details | AppMode::Search(SearchTarget::Commits) => {
                let limit = self.commit_list_map.len().saturating_sub(1);
                let i = match self.commit_list_state.selected() {
                    Some(i) => if i >= limit { 0 } else { i + 1 },
                    None => 0,
                };
                if limit > 0 {
                    self.commit_list_state.select(Some(i));
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
            AppMode::Normal | AppMode::Search(SearchTarget::Projects) => {
                if !self.visible_projects.is_empty() {
                    let limit = self.visible_projects.len().saturating_sub(1);
                    let i = match self.project_list_state.selected() {
                        Some(i) => if i == 0 { limit } else { i - 1 },
                        None => 0,
                    };
                    self.project_list_state.select(Some(i));
                    self.selected_project_idx = Some(self.visible_projects[i]);
                    self.commit_list_state.select(None);
                }
            }
            AppMode::Details | AppMode::CommitsView | AppMode::Search(SearchTarget::Commits) => {
                let limit = self.commit_list_map.len().saturating_sub(1);
                let i = match self.commit_list_state.selected() {
                    Some(i) => if i == 0 { limit } else { i - 1 },
                    None => 0,
                };
                if limit > 0 {
                    self.commit_list_state.select(Some(i));
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
                let mut total_commits = 0;
                for d in &self.projects[proj_idx].dates {
                    for b in &d.branches {
                        total_commits += b.commits.len();
                    }
                }
                if total_commits > 0 {
                    self.mode = AppMode::Details;
                    self.commit_list_state.select(Some(0));
                }
            }
        }
    }

    pub fn enter_commits_view(&mut self) {
        self.mode = AppMode::CommitsView;
        if self.commit_list_state.selected().is_none() {
            self.commit_list_state.select(Some(0));
        }
    }

    pub fn show_diff_for_selected(&mut self) {
        if let Some(idx) = self.commit_list_state.selected() {
            if idx < self.commit_list_map.len() {
                let (date_idx, proj_idx, author_idx, branch_idx, commit_idx) = self.commit_list_map[idx];
                if let (Some(p_idx), Some(a_idx), Some(b_idx), Some(c_idx)) = (proj_idx, author_idx, branch_idx, commit_idx) {
                    if date_idx < self.timeline.len() {
                        let date_group = &self.timeline[date_idx];
                        if p_idx < date_group.projects.len() {
                            let proj = &date_group.projects[p_idx];
                            if a_idx < proj.authors.len() {
                                let author = &proj.authors[a_idx];
                                if b_idx < author.branches.len() {
                                    let branch = &author.branches[b_idx];
                                    if c_idx < branch.commits.len() {
                                        let commit = &branch.commits[c_idx];
                                        let commit_id = &commit.id;
                                        
                                        let proj_name = &proj.name;
                                        let proj_path = self.projects.iter().find(|p| p.name == *proj_name).map(|p| p.path.clone()).unwrap_or_default();
                                        
                                        if let Ok(output) = std::process::Command::new("git")
                                            .arg("show")
                                            .arg("--color=always")
                                            .arg(commit_id)
                                            .current_dir(&proj_path)
                                            .output() {
                                            use ansi_to_tui::IntoText;
                                            if let Ok(text) = output.stdout.into_text() {
                                                self.diff_content = Some(text);
                                            } else {
                                                self.diff_content = Some(ratatui::text::Text::raw("Failed to parse diff"));
                                            }
                                            self.diff_scroll = 0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if commit_idx.is_none() {
                    self.diff_content = None;
                }
            }
        }
    }

    pub fn leave_details(&mut self) {
        self.mode = AppMode::Normal;
        self.commit_list_state.select(None);
    }

    pub fn enter_input_mode(&mut self, mode: AppMode) {
        if !matches!(self.mode, AppMode::InputAuthor | AppMode::InputDate | AppMode::InputProfile | AppMode::InputAddSource | AppMode::ExplorerJumpPath | AppMode::InputBranch | AppMode::Command | AppMode::Search(_)) {
            self.previous_mode = Some(self.mode.clone());
        }
        self.mode = mode.clone();
        match self.mode {
            AppMode::InputAuthor => {
                self.input = self.input.clone().with_value(if self.author_filter == "Any" { String::new() } else { self.author_filter.clone() });
                self.author_list_state.select(Some(0));
            },
            AppMode::InputDate => {
                self.input = self.input.clone().with_value(self.current_profile.date_filter.clone());
            },
            AppMode::InputBranch => {
                self.input = self.input.clone().with_value(self.branch_filter.clone());
            },
            AppMode::Search(target) => {
                let val = match target {
                    SearchTarget::Projects => self.project_search.clone(),
                    SearchTarget::Commits => self.fuzzy_filter.clone(),
                };
                self.input = self.input.clone().with_value(val);
                self.last_search_typing = None;
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
            AppMode::Command => {
                self.input.reset();
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
                self.current_profile.author_filter = self.author_filter.clone();
                self.config.update_active_profile(self.current_profile.clone());
                self.reload_data();
            }
            AppMode::InputDate => {
                if let Some((start, end)) = parse_date_input(self.input.value()) {
                    self.date_start_filter = start;
                    self.date_end_filter = end;
                    self.current_profile.date_filter = self.input.value().to_string();
                    self.config.update_active_profile(self.current_profile.clone());
                    self.reload_data();
                }
            }
            AppMode::InputBranch => {
                self.branch_filter = self.input.value().to_string();
                self.current_profile.branch_filter = self.branch_filter.clone();
                self.config.update_active_profile(self.current_profile.clone());
                self.reload_data();
            }

            AppMode::Command => {
                let cmd = self.input.value().to_string();
                self.execute_command(&cmd);
                if let Some(prev) = self.previous_mode.take() {
                    self.mode = prev;
                } else {
                    self.mode = AppMode::Normal;
                }
            }
            AppMode::InputProfile => {
                let name = self.input.value().to_string();
                if !name.is_empty() {
                    self.current_profile = self.config.switch_profile(&name);
                    self.sources = self.current_profile.sources.clone();
                    
                    self.author_filter = self.current_profile.author_filter.clone();
                    self.branch_filter = self.current_profile.branch_filter.clone();
                    self.hide_zero_commits = self.current_profile.hide_zero_commits;
                    if let Some((start, end)) = crate::app::parse_date_input(&self.current_profile.date_filter) {
                        self.date_start_filter = start;
                        self.date_end_filter = end;
                    }
                    
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
            AppMode::InputExportPath => {
                let export_path = self.input.value().trim().replace("\\", "/");
                if !export_path.is_empty() {
                    let p = std::path::Path::new(&export_path);
                    if let Some(parent) = p.parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() {
                            self.config.default_export_path = Some(parent_str);
                            self.config.save();
                        }
                    }
                    self.export_summary(&export_path);
                }
            }
            _ => {}
        }
        if let Some(prev) = self.previous_mode.take() {
            self.mode = prev;
        } else {
            self.mode = AppMode::Normal;
        }
    }

    pub fn cancel_input(&mut self) {
        if let Some(prev) = self.previous_mode.take() {
            self.mode = prev;
        } else {
            self.mode = AppMode::Normal;
        }
    }

    pub fn handle_mouse_event(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseEventKind, MouseButton};
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
            if let Some(pa) = self.projects_area {
                if pa.x <= event.column && event.column < pa.x + pa.width && pa.y <= event.row && event.row < pa.y + pa.height {
                    self.mode = AppMode::Normal;
                    let offset = self.project_list_state.offset();
                    let clicked_idx = offset.saturating_add((event.row.saturating_sub(pa.y).saturating_sub(1)) as usize);
                    if clicked_idx < self.projects.len() {
                        self.project_list_state.select(Some(clicked_idx));
                        self.selected_project_idx = Some(clicked_idx);
                        self.toggle_project();
                    }
                }
            }
            if let Some(ca) = self.commits_area {
                if ca.x <= event.column && event.column < ca.x + ca.width && ca.y <= event.row && event.row < ca.y + ca.height {
                    if matches!(self.mode, AppMode::Normal) {
                        self.mode = AppMode::CommitsView;
                    }
                }
            }
        } else if let MouseEventKind::ScrollDown = event.kind {
            if matches!(self.mode, AppMode::Normal) {
                self.next_item();
            } else if matches!(self.mode, AppMode::CommitsView | AppMode::Details) {
                let i = match self.commit_list_state.selected() {
                    Some(i) => i.saturating_add(1),
                    None => 0,
                };
                self.commit_list_state.select(Some(i));
            }
        } else if let MouseEventKind::ScrollUp = event.kind {
            if matches!(self.mode, AppMode::Normal) {
                self.previous_item();
            } else if matches!(self.mode, AppMode::CommitsView | AppMode::Details) {
                let i = match self.commit_list_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                self.commit_list_state.select(Some(i));
            }
        }
    }

    pub fn export_summary(&mut self, filename: &str) {
        use std::path::Path;

        let file_path = Path::new(filename);
        
        let path = match std::env::current_dir() {
            Ok(cwd) => cwd.join(file_path),
            Err(e) => {
                self.flash_message = Some(format!("Failed to get current dir: {}", e));
                return;
            }
        };

        let (tx, rx) = std::sync::mpsc::channel();
        self.is_loading = true;
        self.loading_rx = Some(rx);
        let projects = self.projects.clone();
        let timeline = self.timeline.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async move {
                use std::fs::File;
                use std::io::Write;
                
                let mut project_commits: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                for date_group in &timeline {
                    for proj in &date_group.projects {
                        for author_group in &proj.authors {
                            for b in &author_group.branches {
                                for c in &b.commits {
                                    project_commits.entry(proj.name.clone()).or_default().push(c.id.clone());
                                }
                            }
                        }
                    }
                }
                
                let mut join_set = tokio::task::JoinSet::new();
                let mut idx = 0;
                
                for proj in &projects {
                    if !proj.enabled || proj.dates.is_empty() { continue; }
                    
                    let commit_ids = project_commits.get(&proj.name).cloned().unwrap_or_default().clone();
                    if commit_ids.is_empty() { continue; }
                    
                    let proj_path = proj.path.clone();
                    let current_idx = idx;
                    join_set.spawn(async move {
                        let mut output = String::new();
                        for chunk in commit_ids.chunks(500) {
                            if let Ok(cmd_out) = tokio::process::Command::new("git")
                                .arg("show")
                                .arg("--name-only")
                                .arg("--format=COMMIT:%H")
                                .args(chunk)
                                .current_dir(&proj_path)
                                .output()
                                .await 
                            {
                                output.push_str(&String::from_utf8_lossy(&cmd_out.stdout));
                            }
                        }
                        
                        let mut commit_map = std::collections::HashMap::new();
                        let mut current_commit = String::new();
                        let mut current_files = Vec::new();
                        
                        for line in output.lines() {
                            if line.starts_with("COMMIT:") {
                                if !current_commit.is_empty() {
                                    commit_map.insert(current_commit.clone(), current_files.join("\n"));
                                }
                                current_commit = line.trim_start_matches("COMMIT:").trim().to_string();
                                current_files.clear();
                            } else if !line.trim().is_empty() {
                                current_files.push(line.trim().to_string());
                            }
                        }
                        if !current_commit.is_empty() {
                            commit_map.insert(current_commit, current_files.join("\n"));
                        }
                        (current_idx, commit_map)
                    });
                    idx += 1;
                }
                
                let mut all_diffs = std::collections::HashMap::new();
                while let Some(Ok((_, commit_map))) = join_set.join_next().await {
                    all_diffs.extend(commit_map);
                }
                
                // Divide and conquer: Generate markdown for each date in parallel
                use rayon::prelude::*;
                let date_strings: Vec<String> = timeline.par_iter().map(|date_group| {
                    let mut date_content = format!("## Date: {}\n\n", date_group.date);
                    
                    for proj in &date_group.projects {
                        let mut proj_has_commits = false;
                        let mut proj_content = format!("### Repository: {}\n\n", proj.name);
                        
                        for author_group in &proj.authors {
                            if author_group.branches.iter().all(|b| b.commits.is_empty()) { continue; }
                            proj_has_commits = true;
                            proj_content.push_str(&format!("#### Author: `{}`\n\n", author_group.name));
                            
                            for branch in &author_group.branches {
                                for c in &branch.commits {
                                    proj_content.push_str(&format!("- `{}` {}\n", c.id.chars().take(7).collect::<String>(), c.message));
                                    
                                    if let Some(files) = all_diffs.get(&c.id) {
                                        for line in files.lines() {
                                            proj_content.push_str(&format!("  - `{}`\n", line));
                                        }
                                    }
                                }
                            }
                            proj_content.push('\n');
                        }
                        if proj_has_commits {
                            date_content.push_str(&proj_content);
                        }
                    }
                    date_content
                }).collect();
                
                // Reunite
                let mut file = File::create(&path).map_err(|e| e.to_string())?;
                writeln!(file, "# Git Recap Summary\n").map_err(|e| e.to_string())?;
                for ds in date_strings {
                    file.write_all(ds.as_bytes()).map_err(|e| e.to_string())?;
                }
                
                Ok::<String, String>(format!("Exported to {}", path.display()))
            });
            
            let _ = tx.send(LoadingResult::ExportComplete(result));
        });
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
    // fuzzy search logic was removed in favor of live timeline and project filtering
}

fn parse_date_input(input: &str) -> Option<(chrono::DateTime<Local>, chrono::DateTime<Local>)> {
    let now = Local::now();
    let input = input.trim().to_lowercase();
    
    let day_range = |d: NaiveDate| -> (chrono::DateTime<Local>, chrono::DateTime<Local>) {
        let start = d.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
        let end = d.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Local).unwrap();
        (start, end)
    };

    let parse_single = |s: &str| -> Option<(chrono::DateTime<Local>, chrono::DateTime<Local>)> {
        let s = s.trim();
        if s == "today" {
            return Some(day_range(now.date_naive()));
        } else if s == "yesterday" {
            let yest = now.date_naive().pred_opt().unwrap();
            return Some(day_range(yest));
        } else if s == "startofday" {
            return Some((day_range(now.date_naive()).0, now));
        } else if s == "endofday" {
            return Some((now, day_range(now.date_naive()).1));
        } else if s.ends_with("d") || s.ends_with("days") || s.ends_with("day") {
            let num_str = s.trim_end_matches("days").trim_end_matches("day").trim_end_matches('d').trim();
            if let Ok(n) = num_str.parse::<i64>() {
                let start_date = now.date_naive() - chrono::Duration::days(n);
                let start = start_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Local).unwrap();
                return Some((start, end));
            }
        } else if s.ends_with("w") || s.ends_with("weeks") || s.ends_with("week") {
            let num_str = s.trim_end_matches("weeks").trim_end_matches("week").trim_end_matches('w').trim();
            if let Ok(n) = num_str.parse::<i64>() {
                let start_date = now.date_naive() - chrono::Duration::days(n * 7);
                let start = start_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Local).unwrap();
                return Some((start, end));
            }
        } else if s == "startofmonth" {
            let d = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
            return Some((day_range(d).0, now));
        } else if s == "endofmonth" {
            let next_month = if now.month() == 12 { 1 } else { now.month() + 1 };
            let next_year = if now.month() == 12 { now.year() + 1 } else { now.year() };
            let d = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap().pred_opt().unwrap();
            return Some((now, day_range(d).1));
        }

        // Try YYYY-MM
        if s.len() == 7 && s.chars().nth(4) == Some('-') {
            if let (Ok(y), Ok(m)) = (s[0..4].parse::<i32>(), s[5..7].parse::<u32>()) {
                if (1..=12).contains(&m) {
                    if let Some(start_d) = NaiveDate::from_ymd_opt(y, m, 1) {
                        let next_m = if m == 12 { 1 } else { m + 1 };
                        let next_y = if m == 12 { y + 1 } else { y };
                        if let Some(end_d) = NaiveDate::from_ymd_opt(next_y, next_m, 1).unwrap().pred_opt() {
                            return Some((day_range(start_d).0, day_range(end_d).1));
                        }
                    }
                }
            }
        }

        // Try month names or month(...)
        let inner = if s.starts_with("month(") && s.ends_with(')') {
            &s[6..s.len()-1]
        } else {
            s
        };
        
        let chars: String = inner.chars().filter(|c| c.is_alphabetic()).collect();
        let nums: String = inner.chars().filter(|c| c.is_numeric()).collect();
        
        let mut year = now.year();
        let mut month = 0;
        
        if !nums.is_empty() {
            if let Ok(y) = nums.parse::<i32>() {
                if y < 100 { year = 2000 + y; }
                else if y >= 1900 { year = y; }
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
        }
        
        if month > 0 {
            if let Some(start_d) = NaiveDate::from_ymd_opt(year, month, 1) {
                let next_m = if month == 12 { 1 } else { month + 1 };
                let next_y = if month == 12 { year + 1 } else { year };
                if let Some(end_d) = NaiveDate::from_ymd_opt(next_y, next_m, 1).unwrap().pred_opt() {
                    return Some((day_range(start_d).0, day_range(end_d).1));
                }
            }
        }

        let d1 = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .map(|d| d.and_local_timezone(Local).unwrap())
            .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d").map(|d| day_range(d).0));
            
        if let Ok(start) = d1 {
            let end = NaiveDate::parse_from_str(s, "%Y-%m-%d").map(|d| day_range(d).1).unwrap_or(start);
            return Some((start, end));
        }
        
        None
    };

    if input.starts_with("since ") {
        let after = input["since ".len()..].trim();
        if let Some((start, _)) = parse_single(after) {
            return Some((start, now));
        }
    }

    let parts: Vec<&str> = input.split("..").collect();
    if parts.len() == 2 {
        if let (Some((start, _)), Some((_, end))) = (parse_single(parts[0]), parse_single(parts[1])) {
            return Some((start, end));
        }
    } else {
        return parse_single(&input);
    }
    
    None
}


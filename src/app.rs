use crate::git_utils::{self, BranchCommits};
use chrono::{Local, NaiveDate};
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
}

pub struct ProjectData {
    pub name: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub branches: Vec<BranchCommits>,
}

pub struct App {
    pub author_filter: String,
    pub date_start_filter: NaiveDate,
    pub date_end_filter: NaiveDate,
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
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            author_filter: "Any".to_string(),
            date_start_filter: Local::now().date_naive(),
            date_end_filter: Local::now().date_naive(),
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
        };
        app.current_profile = app.config.get_active_profile();
        app.sources = app.current_profile.sources.clone();
        
        app.scan_sources();
        app.reload_data();
        app
    }

    pub fn add_source(&mut self, path: PathBuf) {
        if !self.sources.contains(&path) {
            self.sources.push(path.clone());
            self.current_profile.sources = self.sources.clone();
            self.config.update_active_profile(self.current_profile.clone());
            self.scan_sources();
            self.reload_data();
        }
    }

    pub fn scan_sources(&mut self) {
        let mut authors_set = HashSet::new();
        
        for base_path in &self.sources {
            if !base_path.exists() {
                continue;
            }
            let repos = git_utils::find_git_repos(base_path);
            for repo_path in repos {
                let already_exists = self.projects.iter().any(|p| p.path == repo_path);
                let is_removed = self.current_profile.removed_projects.contains(&repo_path);
                
                if !already_exists && !is_removed {
                    let name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let enabled = !self.current_profile.disabled_projects.contains(&repo_path);
                    
                    self.projects.push(ProjectData {
                        name,
                        path: repo_path.clone(),
                        enabled,
                        branches: Vec::new(),
                    });
                    
                    for author in git_utils::get_recent_authors(&repo_path) {
                        authors_set.insert(author);
                    }
                }
            }
        }
        
        let mut sorted_authors: Vec<String> = authors_set.into_iter().collect();
        sorted_authors.sort();
        self.known_authors = sorted_authors;
        
        if !self.projects.is_empty() && self.selected_project_idx.is_none() {
            self.project_list_state.select(Some(0));
            self.selected_project_idx = Some(0);
        }
    }

    pub fn reload_data(&mut self) {
        for proj in &mut self.projects {
            proj.branches.clear();
            if proj.enabled {
                if let Ok(mut branches) = git_utils::get_commits(&proj.path, &self.author_filter, self.date_start_filter, self.date_end_filter) {
                    for b in &mut branches {
                        b.commits.sort_by(|a, b| b.date.cmp(&a.date));
                    }
                    proj.branches = branches;
                }
            }
        }
    }

    pub fn toggle_project(&mut self) {
        if let Some(idx) = self.selected_project_idx {
            if idx < self.projects.len() {
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
    }

    pub fn remove_project(&mut self) {
        if let Some(idx) = self.selected_project_idx {
            if idx < self.projects.len() {
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
            AppMode::Details => {
                if let Some(proj_idx) = self.selected_project_idx {
                    let total_commits: usize = self.projects[proj_idx].branches.iter().map(|b| b.commits.len()).sum();
                    if total_commits > 0 {
                        let i = match self.commit_list_state.selected() {
                            Some(i) => if i >= total_commits.saturating_sub(1) { 0 } else { i + 1 },
                            None => 0,
                        };
                        self.commit_list_state.select(Some(i));
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
                    let total_commits: usize = self.projects[proj_idx].branches.iter().map(|b| b.commits.len()).sum();
                    if total_commits > 0 {
                        let i = match self.commit_list_state.selected() {
                            Some(i) => if i == 0 { total_commits.saturating_sub(1) } else { i - 1 },
                            None => 0,
                        };
                        self.commit_list_state.select(Some(i));
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
        if self.selected_project_idx.is_some() {
            if let Some(proj_idx) = self.selected_project_idx {
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
                let display = if self.date_start_filter == self.date_end_filter {
                    self.date_start_filter.format("%Y-%m-%d").to_string()
                } else {
                    format!("{}..{}", self.date_start_filter.format("%Y-%m-%d"), self.date_end_filter.format("%Y-%m-%d"))
                };
                self.input = self.input.clone().with_value(display);
            },
            AppMode::InputProfile => {
                self.input = self.input.clone().with_value(self.current_profile.name.clone());
            },
            AppMode::FileExplorer => {
                self.file_explorer.load_directory();
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
                let val = self.input.value().trim();
                let parts: Vec<&str> = val.split("..").collect();
                if parts.len() == 2 {
                    if let (Ok(d1), Ok(d2)) = (
                        NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d"),
                        NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d"),
                    ) {
                        self.date_start_filter = d1;
                        self.date_end_filter = d2;
                        self.reload_data();
                    }
                } else if let Ok(d) = NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                    self.date_start_filter = d;
                    self.date_end_filter = d;
                    self.reload_data();
                }
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
}

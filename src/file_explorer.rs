use ratatui::widgets::ListState;
use std::fs;
use std::path::PathBuf;
use tui_input::Input;

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_git_repo: bool,
}

pub struct FileExplorerState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub list_state: ListState,
    pub search_input: Input,
    pub is_searching: bool,
}

impl FileExplorerState {
    pub fn new(start_path: PathBuf) -> Self {
        let mut fe = FileExplorerState {
            current_path: start_path,
            entries: Vec::new(),
            list_state: ListState::default(),
            search_input: Input::default(),
            is_searching: false,
        };
        fe.load_directory();
        fe
    }

    pub fn load_directory(&mut self) {
        self.entries.clear();
        
        let search_term = self.search_input.value().to_lowercase();

        if let Some(parent) = self.current_path.parent() {
            if search_term.is_empty() {
                self.entries.push(FileEntry {
                    path: parent.to_path_buf(),
                    name: "..".to_string(),
                    is_dir: true,
                    is_git_repo: false,
                });
            }
        }

        if let Ok(read_dir) = fs::read_dir(&self.current_path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                
                if name.starts_with('.') {
                    continue;
                }
                
                if !search_term.is_empty() && !name.to_lowercase().contains(&search_term) {
                    continue;
                }

                let mut is_git_repo = false;
                if is_dir {
                    is_git_repo = path.join(".git").exists();
                }

                self.entries.push(FileEntry {
                    path,
                    name,
                    is_dir,
                    is_git_repo,
                });
            }
        }
        
        self.entries.sort_by(|a, b| {
            if a.name == ".." { return std::cmp::Ordering::Less; }
            if b.name == ".." { return std::cmp::Ordering::Greater; }
            
            b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name))
        });

        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i >= self.entries.len().saturating_sub(1) { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.entries.len().saturating_sub(1) } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    
    pub fn enter_directory(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if idx < self.entries.len() {
                let entry = &self.entries[idx];
                if entry.is_dir {
                    self.current_path = entry.path.clone();
                    self.search_input.reset();
                    self.is_searching = false;
                    self.load_directory();
                }
            }
        }
    }
    
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.search_input.reset();
            self.is_searching = false;
            self.load_directory();
        }
    }
    
    pub fn get_selected_path(&self) -> Option<PathBuf> {
        if let Some(idx) = self.list_state.selected() {
            if idx < self.entries.len() {
                return Some(self.entries[idx].path.clone());
            }
        }
        None
    }
}

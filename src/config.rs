use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppProfile {
    pub name: String,
    pub sources: Vec<PathBuf>,
    pub disabled_projects: Vec<PathBuf>,
    pub removed_projects: Vec<PathBuf>,
}

impl Default for AppProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            sources: vec![PathBuf::from(".")],
            disabled_projects: Vec::new(),
            removed_projects: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub active_profile: String,
    pub profiles: Vec<AppProfile>,
}

impl AppConfig {
    fn get_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = PathBuf::from(home).join(".config").join("git-recap");
        fs::create_dir_all(&config_dir).unwrap_or(());
        config_dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&data) {
                    return config;
                }
            }
        }
        
        Self {
            active_profile: "default".to_string(),
            profiles: vec![AppProfile::default()],
        }
    }

    pub fn save(&self) {
        let path = Self::get_config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }

    pub fn get_active_profile(&self) -> AppProfile {
        self.profiles.iter().find(|p| p.name == self.active_profile).cloned().unwrap_or_default()
    }

    pub fn update_active_profile(&mut self, profile: AppProfile) {
        if let Some(pos) = self.profiles.iter().position(|p| p.name == self.active_profile) {
            self.profiles[pos] = profile;
        } else {
            self.profiles.push(profile);
        }
        self.save();
    }
    
    pub fn switch_profile(&mut self, name: &str) -> AppProfile {
        self.active_profile = name.to_string();
        if !self.profiles.iter().any(|p| p.name == name) {
            let new_profile = AppProfile {
                name: name.to_string(),
                sources: Vec::new(),
                disabled_projects: Vec::new(),
                removed_projects: Vec::new(),
            };
            self.profiles.push(new_profile);
        }
        self.save();
        self.get_active_profile()
    }
}

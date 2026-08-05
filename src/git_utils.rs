use git2::Repository;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Local>,
    pub is_pushed: bool,
}

#[derive(Clone)]
pub struct DateGroup {
    pub date: String,
    pub is_expanded: bool,
    pub branches: Vec<BranchGroup>,
}

#[derive(Clone)]
pub struct BranchGroup {
    pub name: String,
    pub is_expanded: bool,
    pub commits: Vec<GitCommit>,
}

pub fn find_git_repos(base_path: &Path) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut it = WalkDir::new(base_path).max_depth(4).into_iter();
    
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => entry,
        };
        
        let name = entry.file_name().to_string_lossy();
        
        if name == ".git" && entry.file_type().is_dir() {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
            it.skip_current_dir();
        } else if entry.file_type().is_dir() {
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "vendor" || name == "build" || name == "dist" {
                it.skip_current_dir();
            }
        }
    }
    repos
}

pub fn get_commits(
    repo_path: &Path,
    author_name: &str,
    branch_filter: &str,
    start_date: chrono::DateTime<chrono::Local>,
    end_date: chrono::DateTime<chrono::Local>,
) -> Result<Vec<DateGroup>, git2::Error> {
    let repo = Repository::open(repo_path)?;
    let mut branch_commits = Vec::new();
    let start_ts = start_date.timestamp();
    let end_ts = end_date.timestamp();

    let mut pushed_commits = std::collections::HashSet::new();
    if let Ok(mut revwalk) = repo.revwalk() {
        let _ = revwalk.set_sorting(git2::Sort::TIME);
        if let Ok(branches) = repo.branches(Some(git2::BranchType::Remote)) {
            for branch in branches.filter_map(|b| b.ok()) {
                if let Some(target) = branch.0.get().target() {
                    let _ = revwalk.push(target);
                }
            }
        }
        for oid in revwalk.filter_map(|id| id.ok()) {
            if let Ok(commit) = repo.find_commit(oid) {
                if commit.time().seconds() < start_ts - 14 * 24 * 3600 {
                    break;
                }
                pushed_commits.insert(oid);
            }
        }
    }

    let filter_author = author_name.to_lowercase();
    let filter_branches: Vec<String> = branch_filter.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();

    if let Ok(local_branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch_res in local_branches.filter_map(|b| b.ok()) {
            let (branch, _) = branch_res;
            let raw_branch_name = branch.name().unwrap_or(Some("unknown")).unwrap_or("unknown").to_string();
            let branch_name = raw_branch_name.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>();
            
            if !filter_branches.is_empty() {
                let matches = filter_branches.iter().any(|fb| branch_name.to_lowercase().contains(fb));
                if !matches {
                    continue;
                }
            }
            
            let mut commits = Vec::new();
            if let Some(target) = branch.get().target()
                && let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push(target);
                    let _ = revwalk.set_sorting(git2::Sort::TIME);
                    
                    for oid in revwalk.filter_map(|id| id.ok()) {
                        if let Ok(commit) = repo.find_commit(oid) {
                            let committer_time = commit.time().seconds();
                            let author_time = commit.author().when().seconds();
                            
                            if committer_time < start_ts
                                && (start_ts - committer_time) > 7 * 24 * 3600 {
                                    break;
                                }
                            
                            if author_time >= start_ts && author_time <= end_ts {
                                let author = commit.author();
                                let author_str = author.name().unwrap_or("").to_lowercase();
                                
                                if filter_author.is_empty() || filter_author == "any" || author_str.contains(&filter_author) {
                                    let is_pushed = pushed_commits.contains(&oid);
                                    let msg_bytes = commit.message_bytes();
                                    let msg_str = String::from_utf8_lossy(msg_bytes);
                                    let summary = msg_str.lines().next().unwrap_or("").to_string();
                                    
                                    let sanitize_for_tui = |s: &str| -> String {
                                        s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect()
                                    };
                                    
                                    let local_time = chrono::DateTime::from_timestamp(author_time, 0).unwrap_or_default().into();
                                    
                                    commits.push(GitCommit {
                                        id: oid.to_string(),
                                        message: sanitize_for_tui(&summary),
                                        author: sanitize_for_tui(&author.name().unwrap_or("")),
                                        date: local_time,
                                        is_pushed,
                                    });
                                }
                            }
                        }
                    }
                }
            
            if !commits.is_empty() {
                branch_commits.push((branch_name, commits));
            }
        }
    }
    
    let mut date_map: std::collections::HashMap<String, std::collections::HashMap<String, Vec<GitCommit>>> = std::collections::HashMap::new();
    
    for (branch_name, commits) in branch_commits {
        for commit in commits {
            let d_str = commit.date.format("%Y-%m-%d").to_string();
            date_map.entry(d_str).or_default().entry(branch_name.clone()).or_default().push(commit);
        }
    }
    
    let mut date_groups = Vec::new();
    for (date, branch_map) in date_map {
        let mut branches = Vec::new();
        for (name, commits) in branch_map {
            let mut sorted_commits = commits;
            sorted_commits.sort_by(|a, b| b.date.cmp(&a.date));
            branches.push(BranchGroup {
                name,
                is_expanded: true,
                commits: sorted_commits,
            });
        }
        branches.sort_by(|a, b| a.name.cmp(&b.name));
        date_groups.push(DateGroup {
            date,
            is_expanded: true,
            branches,
        });
    }
    date_groups.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(date_groups)
}

pub fn get_recent_authors(repo_path: &Path) -> Vec<String> {
    let mut authors = Vec::new();
    if let Ok(repo) = Repository::open(repo_path)
        && let Ok(mut revwalk) = repo.revwalk() {
            let _ = revwalk.push_head();
            for oid in revwalk.filter_map(|id| id.ok()).take(50) {
                if let Ok(commit) = repo.find_commit(oid) {
                    let author_name = commit.author().name().unwrap_or("").to_string();
                    if !author_name.is_empty() {
                        authors.push(author_name);
                    }
                }
            }
        }
    authors
}

pub fn get_last_commit_info(repo_path: &Path) -> Option<String> {
    let repo = Repository::open(repo_path).ok()?;
    let head = repo.head().ok()?;
    let branch_name = head.shorthand().unwrap_or("unknown");
    
    let commit = head.peel_to_commit().ok()?;
    let commit_time = commit.author().when().seconds();
    let local_time = chrono::DateTime::from_timestamp(commit_time, 0).unwrap_or_default();
    let local_date = chrono::DateTime::<Local>::from(local_time).format("%Y-%m-%d").to_string();
    
    Some(format!("(last committed on {} on branch {})", local_date, branch_name))
}

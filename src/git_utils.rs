use git2::Repository;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Local>,
    pub is_pushed: bool,
}

pub struct BranchCommits {
    pub name: String,
    pub commits: Vec<GitCommit>,
}

pub fn find_git_repos(base_path: &Path) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let walker = WalkDir::new(base_path).into_iter().filter_entry(|e| {
        e.file_name() == ".git" || !e.file_name().to_string_lossy().starts_with('.')
    });

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_name() == ".git" && entry.file_type().is_dir()
            && let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
    }
    repos
}

pub fn get_commits(
    repo_path: &Path,
    author_name: &str,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<Vec<BranchCommits>, git2::Error> {
    let repo = Repository::open(repo_path)?;
    let mut branch_commits = Vec::new();

    if let Ok(local_branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch_res in local_branches.filter_map(|b| b.ok()) {
            let (branch, _) = branch_res;
            let branch_name = branch.name().unwrap_or(Some("unknown")).unwrap_or("unknown").to_string();
            
            let mut commits = Vec::new();
            if let Some(target) = branch.get().target()
                && let Ok(mut revwalk) = repo.revwalk() {
                    let _ = revwalk.push(target);
                    let _ = revwalk.set_sorting(git2::Sort::TIME);
                    
                    for oid in revwalk.filter_map(|id| id.ok()) {
                        if let Ok(commit) = repo.find_commit(oid) {
                            let time = commit.time();
                            if let Some(date_time) = chrono::DateTime::from_timestamp(time.seconds(), 0) {
                                let local_time: DateTime<Local> = date_time.into();
                                let naive_date = local_time.date_naive();
                                
                                if naive_date < start_date
                                    && (start_date - naive_date).num_days() > 7 {
                                        break;
                                    }
                                
                                if naive_date >= start_date && naive_date <= end_date {
                                    let author = commit.author();
                                    let author_str = author.name().unwrap_or("").to_lowercase();
                                    let filter_author = author_name.to_lowercase();
                                    
                                    if filter_author.is_empty() || filter_author == "any" || author_str.contains(&filter_author) {
                                        let is_pushed = is_commit_pushed(&repo, oid);
                                        let msg_bytes = commit.message_bytes();
                                        let msg_str = String::from_utf8_lossy(msg_bytes);
                                        let summary = msg_str.lines().next().unwrap_or("").to_string();
                                        
                                        commits.push(GitCommit {
                                            id: oid.to_string(),
                                            message: summary,
                                            author: author.name().unwrap_or("").to_string(),
                                            date: local_time,
                                            is_pushed,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            
            if !commits.is_empty() {
                branch_commits.push(BranchCommits { name: branch_name, commits });
            }
        }
    }

    Ok(branch_commits)
}

fn is_commit_pushed(repo: &Repository, commit_id: git2::Oid) -> bool {
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Remote)) {
        for branch in branches.filter_map(|b| b.ok()) {
            if let Some(target) = branch.0.get().target()
                && let Ok(base) = repo.merge_base(commit_id, target)
                    && base == commit_id {
                        return true;
                    }
        }
    }
    false
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

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorktreeInfo {
    pub is_worktree: bool,
    pub worktree_name: String,
    #[allow(dead_code)]
    pub parent_repo_path: String,
    pub parent_workspace: String,
    pub branch: Option<String>,
}

/// Detect if a given directory path is a Git worktree by inspecting the `.git` file pointer.
/// In Git worktrees, `.git` is a file containing: `gitdir: <path_to_parent>/.git/worktrees/<name>`.
pub fn detect_git_worktree(path: &Path) -> Option<WorktreeInfo> {
    let git_path = path.join(".git");
    if !git_path.is_file() {
        return None;
    }

    let content = fs::read_to_string(&git_path).ok()?;
    let trimmed = content.trim();
    if !trimmed.starts_with("gitdir:") {
        return None;
    }

    let raw_gitdir = trimmed.trim_start_matches("gitdir:").trim();
    let gitdir_path = if Path::new(raw_gitdir).is_relative() {
        path.join(raw_gitdir)
    } else {
        PathBuf::from(raw_gitdir)
    };

    let gitdir_canon = fs::canonicalize(&gitdir_path).unwrap_or(gitdir_path);

    // gitdir_canon typically ends in `.git/worktrees/<name>`
    let worktree_name = gitdir_canon
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "worktree".to_string())
        });

    // Parent repo root resolution:
    // gitdir_canon: `<parent_repo>/.git/worktrees/<name>`
    // parent 1: `<parent_repo>/.git/worktrees`
    // parent 2: `<parent_repo>/.git`
    // parent 3: `<parent_repo>`
    let parent_repo_path = gitdir_canon
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())?;

    let parent_workspace = parent_repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "default".to_string());

    // Extract current branch from `<gitdir>/HEAD`
    let head_file = gitdir_canon.join("HEAD");
    let branch = fs::read_to_string(head_file).ok().map(|h| {
        let h_trim = h.trim();
        if h_trim.starts_with("ref: refs/heads/") {
            h_trim.trim_start_matches("ref: refs/heads/").to_string()
        } else {
            "detached".to_string()
        }
    });

    Some(WorktreeInfo {
        is_worktree: true,
        worktree_name,
        parent_repo_path: parent_repo_path.to_string_lossy().to_string(),
        parent_workspace,
        branch,
    })
}

/// Enumerate all active git worktrees branched from a parent repository by inspecting `.git/worktrees`.
#[allow(dead_code)]
pub fn discover_parent_worktrees(parent_repo_path: &Path) -> Vec<WorktreeInfo> {
    let worktrees_dir = parent_repo_path.join(".git").join("worktrees");
    if !worktrees_dir.is_dir() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let entries = match fs::read_dir(&worktrees_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let parent_workspace = parent_repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "default".to_string());

    for entry in entries.filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let gitdir_file = entry_path.join("gitdir");
        if let Ok(content) = fs::read_to_string(&gitdir_file) {
            let target_git = PathBuf::from(content.trim());
            // target_git is `<worktree_root>/.git`
            if let Some(target_worktree_root) = target_git.parent() {
                if target_worktree_root.exists() {
                    let worktree_name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let head_file = entry_path.join("HEAD");
                    let branch = fs::read_to_string(head_file).ok().map(|h| {
                        let h_trim = h.trim();
                        if h_trim.starts_with("ref: refs/heads/") {
                            h_trim.trim_start_matches("ref: refs/heads/").to_string()
                        } else {
                            "detached".to_string()
                        }
                    });

                    results.push(WorktreeInfo {
                        is_worktree: true,
                        worktree_name,
                        parent_repo_path: parent_repo_path.to_string_lossy().to_string(),
                        parent_workspace: parent_workspace.clone(),
                        branch,
                    });
                }
            }
        }
    }

    results
}

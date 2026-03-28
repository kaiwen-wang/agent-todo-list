//! Git utilities for commit discovery and remote URL resolution.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Information about a single git commit.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub subject: String,
    pub author: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

const FIELD_SEP: &str = "\x1f"; // ASCII unit separator

/// Get all commits on `branch` that are not on `base` branch.
/// Returns newest-first.
pub fn commits_on_branch(repo_root: &Path, branch: &str, base: &str) -> Result<Vec<CommitInfo>> {
    let format = format!("%H{FIELD_SEP}%h{FIELD_SEP}%s{FIELD_SEP}%an{FIELD_SEP}%at");
    let range = format!("{base}..{branch}");

    let output = Command::new("git")
        .args(["log", &range, &format!("--format={format}"), "--no-merges"])
        .current_dir(repo_root)
        .output()
        .context("Failed to run git log")?;

    if !output.status.success() {
        // Branch may not exist yet or has no divergent commits
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_log_output(&stdout))
}

/// Hydrate a list of SHAs into CommitInfo structs.
/// SHAs that don't exist in the repo are silently skipped.
pub fn enrich_commits(repo_root: &Path, shas: &[String]) -> Vec<CommitInfo> {
    if shas.is_empty() {
        return vec![];
    }

    let format = format!("%H{FIELD_SEP}%h{FIELD_SEP}%s{FIELD_SEP}%an{FIELD_SEP}%at");
    // Use git show with --no-patch for each SHA
    let mut results = Vec::with_capacity(shas.len());
    for sha in shas {
        let output = Command::new("git")
            .args(["show", "--no-patch", &format!("--format={format}"), sha])
            .current_dir(repo_root)
            .output();

        if let Ok(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut infos = parse_log_output(stdout.trim());
            if let Some(info) = infos.pop() {
                results.push(info);
            }
        }
    }
    results
}

/// Get the default branch name (main or master).
pub fn get_default_branch(repo_root: &Path) -> Result<String> {
    // Try origin/HEAD first
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
        .current_dir(repo_root)
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Strip "origin/" prefix
        if let Some(name) = branch.strip_prefix("origin/") {
            return Ok(name.to_string());
        }
        return Ok(branch);
    }

    // Fall back to checking if main or master exists
    for name in &["main", "master"] {
        let output = Command::new("git")
            .args(["rev-parse", "--verify", name])
            .current_dir(repo_root)
            .output();
        if let Ok(output) = output
            && output.status.success()
        {
            return Ok(name.to_string());
        }
    }

    Ok("main".to_string())
}

/// Verify a commit SHA exists in the repo and return the full SHA.
/// Returns an error if the SHA doesn't resolve to a commit.
pub fn verify_commit(repo_root: &Path, sha: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("{sha}^{{commit}}")])
        .current_dir(repo_root)
        .output()
        .context("Failed to run git rev-parse")?;

    if !output.status.success() {
        anyhow::bail!("Commit {} not found in this repository", sha);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Resolve HEAD to a full SHA.
pub fn resolve_head(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output()
        .context("Failed to run git rev-parse HEAD")?;

    if !output.status.success() {
        anyhow::bail!("Failed to resolve HEAD");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the remote commit URL for a SHA, or None if no remote is configured.
///
/// Handles:
/// - `git@github.com:owner/repo.git`
/// - `https://github.com/owner/repo.git`
/// - GitLab, Bitbucket, etc. (same patterns)
pub fn remote_commit_url(repo_root: &Path, sha: &str) -> Option<String> {
    let base = remote_base_url(repo_root)?;
    Some(format!("{base}/commit/{sha}"))
}

/// Parse the origin remote URL into an HTTPS base URL.
pub fn remote_base_url(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_remote_url(&url)
}

/// Parse a git remote URL into an HTTPS base URL.
fn parse_remote_url(url: &str) -> Option<String> {
    let url = url.trim();

    // SSH format: git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;
        let path = path.trim_end_matches(".git");
        return Some(format!("https://{host}/{path}"));
    }

    // HTTPS format: https://github.com/owner/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let trimmed = url.trim_end_matches(".git");
        return Some(trimmed.to_string());
    }

    None
}

fn parse_log_output(stdout: &str) -> Vec<CommitInfo> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(FIELD_SEP).collect();
            if parts.len() < 5 {
                return None;
            }
            Some(CommitInfo {
                sha: parts[0].to_string(),
                short_sha: parts[1].to_string(),
                subject: parts[2].to_string(),
                author: parts[3].to_string(),
                timestamp: parts[4].parse().unwrap_or(0),
                url: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_remote_url_ssh() {
        assert_eq!(
            parse_remote_url("git@github.com:user/repo.git"),
            Some("https://github.com/user/repo".to_string())
        );
    }

    #[test]
    fn test_parse_remote_url_https() {
        assert_eq!(
            parse_remote_url("https://github.com/user/repo.git"),
            Some("https://github.com/user/repo".to_string())
        );
    }

    #[test]
    fn test_parse_remote_url_https_no_suffix() {
        assert_eq!(
            parse_remote_url("https://github.com/user/repo"),
            Some("https://github.com/user/repo".to_string())
        );
    }

    #[test]
    fn test_parse_remote_url_gitlab() {
        assert_eq!(
            parse_remote_url("git@gitlab.com:team/project.git"),
            Some("https://gitlab.com/team/project".to_string())
        );
    }
}

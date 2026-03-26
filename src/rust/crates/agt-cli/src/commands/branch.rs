//! `agt branch` — Create a git worktree + branch for a todo.
//!
//! Creates .worktrees/<branch-name>/ with a new branch.
//! Sets the branch name on the todo in the CRDT.

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use agt_lib::operations;
use agt_lib::queries;
use agt_lib::schema::Todo;

use super::{load_project, parse_ref, save_project};

/// Articles to strip from branch slugs.
const ARTICLES: &[&str] = &["a", "an", "the"];

/// Generate a branch name from a todo: `prefix-N-title-slug`.
pub fn make_branch_name(prefix: &str, num: u64, title: &str) -> String {
    let words: Vec<String> = title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !ARTICLES.contains(w))
        .take(5)
        .map(|s| s.to_string())
        .collect();

    let slug = words.join("-");
    format!("{}-{}-{}", prefix.to_lowercase(), num, slug)
}

/// Ensure .worktrees/ is listed in .gitignore.
fn ensure_gitignore_worktrees(root: &Path) -> Result<()> {
    let gitignore_path = root.join(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.contains(".worktrees") {
            let updated = format!(
                "{}\n\n# Git worktrees for todo branches\n.worktrees/\n",
                content.trim_end()
            );
            fs::write(&gitignore_path, updated)?;
        }
    } else {
        fs::write(
            &gitignore_path,
            "# Git worktrees for todo branches\n.worktrees/\n",
        )?;
    }
    Ok(())
}

/// Create a git worktree for a todo, returning the worktree path and branch name.
/// If the todo already has a branch, returns its existing worktree path.
///
/// This is the shared function used by both `agt branch` and `agt run`.
pub fn ensure_worktree(root: &Path, prefix: &str, todo: &Todo) -> Result<(String, PathBuf)> {
    // If already branched, return existing info
    if let Some(ref branch) = todo.branch {
        let worktree_path = root.join(".worktrees").join(branch);
        if worktree_path.exists() {
            return Ok((branch.clone(), worktree_path));
        }
        // Branch exists on todo but worktree dir is missing — recreate it
        ensure_gitignore_worktrees(root)?;
        let output = Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
            .current_dir(root)
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Failed to recreate worktree for existing branch {}: {}",
                branch,
                stderr.trim()
            );
        }
        return Ok((branch.clone(), worktree_path));
    }

    // Generate new branch name
    let branch_name = make_branch_name(prefix, todo.number, &todo.title);
    let worktrees_dir = root.join(".worktrees");
    let worktree_path = worktrees_dir.join(&branch_name);

    if worktree_path.exists() {
        // Worktree dir exists but todo doesn't have branch set — it's from a previous run
        return Ok((branch_name, worktree_path));
    }

    ensure_gitignore_worktrees(root)?;

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            &branch_name,
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(root)
        .output()
        .context("Failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create worktree: {}", stderr.trim());
    }

    Ok((branch_name, worktree_path))
}

/// `agt branch <ref>` — create worktree + branch, record on todo.
pub fn run(reference: String, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

    if let Some(ref branch) = todo.branch {
        let worktree_path = paths.root.join(".worktrees").join(branch);
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "reference": format!("{}-{}", prefix, num),
                    "branch": branch,
                    "worktree": worktree_path.display().to_string(),
                })
            );
        } else {
            println!("Todo {}-{} already has branch: {}", prefix, num, branch);
        }
        return Ok(());
    }

    let (branch_name, worktree_path) = ensure_worktree(&paths.root, &prefix, &todo)?;

    // Record branch on todo
    operations::set_branch(&mut doc, num, &branch_name, None)?;
    save_project(&paths, &mut doc)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "reference": format!("{}-{}", prefix, num),
                "branch": branch_name,
                "worktree": worktree_path.display().to_string(),
            })
        );
    } else {
        println!("Created branch: {}", branch_name);
        println!("  Worktree: {}", worktree_path.display());
    }
    Ok(())
}

//! `agt unbranch` — Remove a git worktree + branch for a todo.

use anyhow::Result;
use std::process::Command;

use agt_lib::operations;
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String, keep_branch: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

    let branch_name = match &todo.branch {
        Some(b) => b.clone(),
        None => {
            println!("Todo {}-{} has no branch.", prefix, num);
            return Ok(());
        }
    };

    let worktree_path = paths.root.join(".worktrees").join(&branch_name);

    // Remove worktree if it exists
    if worktree_path.exists() {
        let output = Command::new("git")
            .args(["worktree", "remove", worktree_path.to_str().unwrap()])
            .current_dir(&paths.root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: Failed to remove worktree: {}", stderr.trim());
        }
    }

    // Delete the git branch unless --keep-branch
    if !keep_branch {
        let output = Command::new("git")
            .args(["branch", "-d", &branch_name])
            .current_dir(&paths.root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: Could not delete branch: {}", stderr.trim());
        }
    }

    // Clear branch reference on the todo
    operations::clear_branch(&mut doc, num, None)?;
    save_project(&paths, &mut doc)?;

    println!("Removed branch: {}", branch_name);
    Ok(())
}

use anyhow::{Result, bail};
use std::process::Command;

use agt_lib::history;
use agt_lib::project::find_project;
use agt_lib::storage;

use std::env;

/// Build a commit message from new audit log entries.
///
/// Loads the last-committed version of data.automerge from git HEAD,
/// counts its audit entries, then diffs against the current doc to find
/// only the new entries.
fn build_commit_message(paths: &agt_lib::project::TodoPaths) -> Result<String> {
    let mut current_doc = match storage::load_doc(&paths.data_path)? {
        Some(doc) => doc,
        None => return Ok("todo: update".to_string()),
    };

    let current_count = history::get_audit_log_count(&mut current_doc);

    // Load the last-committed version from git
    let committed_count = {
        let output = Command::new("git")
            .args(["show", "HEAD:.todo/data.automerge"])
            .output();

        match output {
            Ok(out) if out.status.success() && !out.stdout.is_empty() => {
                match automerge::AutoCommit::load(&out.stdout) {
                    Ok(mut committed_doc) => history::get_audit_log_count(&mut committed_doc),
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    };

    let new_count = current_count.saturating_sub(committed_count);
    if new_count == 0 {
        return Ok("todo: update".to_string());
    }

    // Get the new entries (they're newest-first, so take the first new_count)
    let all_entries = history::get_audit_log(&mut current_doc, Some(new_count), None);

    summarize_entries(&all_entries)
}

fn summarize_entries(entries: &[agt_lib::schema::AuditEntry]) -> Result<String> {
    if entries.is_empty() {
        return Ok("todo: update".to_string());
    }

    let parts: Vec<String> = entries
        .iter()
        .map(|e| {
            let verb = match e.action.as_str() {
                "todo.created" => "created",
                "todo.updated" => "updated",
                "todo.deleted" => "deleted",
                "todo.unassigned" => "unassigned",
                "todo.commented" => "commented on",
                "todo.branched" => "branched",
                "todo.unbranched" => "unbranched",
                "member.added" => "added member",
                "member.removed" => "removed member",
                "member.updated" => "updated member",
                "project.updated" => "updated project",
                _ => &e.action,
            };
            format!("{} {}", verb, e.target)
        })
        .collect();

    const MAX_SHOWN: usize = 3;
    if parts.len() <= MAX_SHOWN {
        Ok(format!("todo: {}", parts.join(", ")))
    } else {
        let shown = parts[..MAX_SHOWN].join(", ");
        let remaining = parts.len() - MAX_SHOWN;
        Ok(format!("todo: {} and {} more", shown, remaining))
    }
}

pub fn run(push: bool, message: Option<String>) -> Result<()> {
    let cwd = env::current_dir()?;
    let paths = find_project(&cwd)
        .ok_or_else(|| anyhow::anyhow!("Not in an agt project. Run `agt init` first."))?;

    // Stage all .todo/ files
    let add_status = Command::new("git")
        .args(["add", &paths.todo_dir.to_string_lossy()])
        .current_dir(&paths.root)
        .status()?;

    if !add_status.success() {
        bail!("git add failed");
    }

    // Check if there's anything staged in .todo/
    let diff_status = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--quiet",
            "--",
            &paths.todo_dir.to_string_lossy(),
        ])
        .current_dir(&paths.root)
        .status()?;

    if diff_status.success() {
        println!("Nothing to commit — .todo/ is already up to date.");
        return Ok(());
    }

    let msg = match message {
        Some(m) => m,
        None => build_commit_message(&paths)?,
    };

    let commit_status = Command::new("git")
        .args([
            "commit",
            "-m",
            &msg,
            "--",
            paths.todo_dir.to_string_lossy().as_ref(),
        ])
        .current_dir(&paths.root)
        .status()?;

    if !commit_status.success() {
        bail!("git commit failed");
    }

    println!("Committed: {}", msg);

    if push {
        let push_status = Command::new("git")
            .args(["push"])
            .current_dir(&paths.root)
            .status()?;

        if !push_status.success() {
            bail!("git push failed");
        }

        println!("Pushed to remote.");
    }

    Ok(())
}

use anyhow::{Result, bail};
use std::process::Command;

use agt_lib::operations;
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

    // Generate branch name from todo
    let slug: String = todo
        .title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    // Collapse multiple dashes
    let slug = regex::Regex::new(r"-+")
        .unwrap()
        .replace_all(&slug, "-")
        .to_string();
    let branch_name = format!(
        "{}-{}/{}",
        prefix.to_lowercase(),
        num,
        &slug[..slug.len().min(50)]
    );

    // Create git worktree
    let output = Command::new("git")
        .args(["worktree", "add", "-b", &branch_name, &format!("../{branch_name}")])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create worktree: {}", stderr.trim());
    }

    // Record branch on todo
    operations::set_branch(&mut doc, num, &branch_name, None)?;
    save_project(&paths, &mut doc)?;

    println!("Created branch and worktree: {}", branch_name);
    Ok(())
}

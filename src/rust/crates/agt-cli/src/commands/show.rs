use anyhow::{Result, bail};

use agt_lib::git;
use agt_lib::queries;

use super::{load_project, parse_ref};
use crate::output;

pub fn run(reference: String, json: bool) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num);
    let Some(todo) = todo else {
        bail!("Todo {}-{} not found", prefix, num);
    };

    // Discover branch commits (live from git) and enrich linked commits
    let branch_commits = if let Some(ref branch) = todo.branch {
        let base = git::get_default_branch(&paths.root).unwrap_or_else(|_| "main".into());
        git::commits_on_branch(&paths.root, branch, &base).unwrap_or_default()
    } else {
        vec![]
    };

    let linked_commits = git::enrich_commits(&paths.root, &todo.commits);

    // Add remote URLs
    let remote_base = git::remote_base_url(&paths.root);
    let add_urls = |mut commits: Vec<git::CommitInfo>| -> Vec<git::CommitInfo> {
        if let Some(ref base) = remote_base {
            for c in &mut commits {
                c.url = Some(format!("{base}/commit/{}", c.sha));
            }
        }
        commits
    };
    let branch_commits = add_urls(branch_commits);
    let linked_commits = add_urls(linked_commits);

    if json {
        let mut val = serde_json::to_value(&todo)?;
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "branchCommits".into(),
                serde_json::to_value(&branch_commits)?,
            );
            obj.insert(
                "linkedCommits".into(),
                serde_json::to_value(&linked_commits)?,
            );
        }
        println!("{}", serde_json::to_string_pretty(&val)?);
    } else {
        let members = queries::read_all_members(&doc);
        output::print_todo_detail(&todo, &prefix, &members, &branch_commits, &linked_commits);

        // Show plan hint if it exists
        if let Some(plan_path) = &todo.plan_path {
            let full_path = paths.todo_dir.join(plan_path);
            if full_path.exists() {
                println!();
                println!(
                    "{}",
                    colored::Colorize::dimmed(
                        format!(
                            "Plan: {} (run `agt plan show {}` to view)",
                            plan_path, reference
                        )
                        .as_str(),
                    )
                );
            }
        }
    }

    Ok(())
}

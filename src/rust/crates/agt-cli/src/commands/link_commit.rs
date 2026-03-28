//! `agt link-commit <ref> [sha]` — Link a git commit to a todo.

use anyhow::Result;

use agt_lib::git;
use agt_lib::operations;
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String, sha: Option<String>, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    // Verify todo exists
    queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    // Resolve SHA: use provided or default to HEAD, then verify it exists
    let raw_sha = match sha {
        Some(s) => s,
        None => git::resolve_head(&paths.root)?,
    };
    let commit_sha = git::verify_commit(&paths.root, &raw_sha)?;

    operations::link_commit(&mut doc, num, &commit_sha, None)?;
    save_project(&paths, &mut doc)?;

    let short = if commit_sha.len() > 8 {
        &commit_sha[..8]
    } else {
        &commit_sha
    };

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "reference": todo_ref,
                "commit": commit_sha,
            })
        );
    } else {
        println!("Linked {} to {}", short, todo_ref);
    }

    Ok(())
}

//! `agt plan` — Manage plan/research files for todos.
//!
//! Plans live as markdown files in `.todo/plans/<REF>.md`.
//! The automerge doc stores a `planPath` pointer on the todo.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

/// Resolve the plan file path for a todo. Returns (relative_path, absolute_path).
fn plan_paths(todo_dir: &std::path::Path, prefix: &str, num: u64) -> (String, PathBuf) {
    let relative = format!("plans/{}-{}.md", prefix, num);
    let absolute = todo_dir.join(&relative);
    (relative, absolute)
}

/// `agt plan show <ref>`
pub fn show(reference: String) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let plan_file = if let Some(plan_path) = &todo.plan_path {
        paths.todo_dir.join(plan_path)
    } else {
        // Check if file exists even without pointer
        let (_, abs) = plan_paths(&paths.todo_dir, &prefix, num);
        abs
    };

    if !plan_file.exists() {
        println!("{}", format!("No plan file for {}", todo_ref).dimmed());
        return Ok(());
    }

    let content = fs::read_to_string(&plan_file)
        .with_context(|| format!("Failed to read {}", plan_file.display()))?;
    println!("{}", content);
    Ok(())
}

/// `agt plan init <ref>` — Create the plan file and set planPath on the todo.
pub fn init(reference: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let (relative, absolute) = plan_paths(&paths.todo_dir, &prefix, num);

    // Create plans directory
    let plans_dir = paths.todo_dir.join("plans");
    fs::create_dir_all(&plans_dir)?;

    if absolute.exists() {
        // File already exists, just ensure pointer is set
        if todo.plan_path.is_none() {
            operations::update_todo(
                &mut doc,
                num,
                UpdateTodoFields {
                    plan_path: Some(Some(&relative)),
                    ..Default::default()
                },
                None,
            )?;
            save_project(&paths, &mut doc)?;
            println!("Linked existing plan file to {}", todo_ref);
        } else {
            println!("Plan already exists for {}", todo_ref);
        }
        println!("  {}", absolute.display());
        return Ok(());
    }

    // Create the file with a header
    let content = format!("# {}: {}\n\n", todo_ref, todo.title);
    fs::write(&absolute, &content)?;

    // Set planPath on the todo
    operations::update_todo(
        &mut doc,
        num,
        UpdateTodoFields {
            plan_path: Some(Some(&relative)),
            ..Default::default()
        },
        None,
    )?;
    save_project(&paths, &mut doc)?;

    println!("Created plan for {}", todo_ref);
    println!("  {}", absolute.display());
    Ok(())
}

/// `agt plan answer <ref> <text>` — Append answers to the plan file.
pub fn answer(reference: String, text: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    // Find plan file
    let plan_file = if let Some(plan_path) = &todo.plan_path {
        paths.todo_dir.join(plan_path)
    } else {
        let (_, abs) = plan_paths(&paths.todo_dir, &prefix, num);
        abs
    };

    if !plan_file.exists() {
        bail!(
            "No plan file for {}. Run `agt plan init {}` first.",
            todo_ref,
            todo_ref
        );
    }

    let mut content = fs::read_to_string(&plan_file)?;

    // Find or create the Answers section
    if content.contains("\n## Answers\n") || content.contains("\n## Answers\r\n") {
        // Append to existing answers section
        content.push_str(&format!("\n> {}\n", text));
    } else {
        // Add new answers section
        content.push_str(&format!("\n## Answers\n\n> {}\n", text));
    }

    fs::write(&plan_file, &content)?;

    // Ensure planPath is set if it wasn't
    if todo.plan_path.is_none() {
        let (relative, _) = plan_paths(&paths.todo_dir, &prefix, num);
        operations::update_todo(
            &mut doc,
            num,
            UpdateTodoFields {
                plan_path: Some(Some(&relative)),
                ..Default::default()
            },
            None,
        )?;
        save_project(&paths, &mut doc)?;
    }

    println!("Answer added to {} plan", todo_ref);
    Ok(())
}

/// `agt plan path <ref>` — Print the plan file path (for scripting/agents).
pub fn path(reference: String) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

    if let Some(plan_path) = &todo.plan_path {
        let absolute = paths.todo_dir.join(plan_path);
        println!("{}", absolute.display());
    } else {
        let (_, absolute) = plan_paths(&paths.todo_dir, &prefix, num);
        println!("{}", absolute.display());
    }
    Ok(())
}

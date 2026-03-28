//! `agt queue <ref> [<ref>...]` — Set todo status to todo (ready for agent dispatch).
//!
//! Convenience command equivalent to `agt update <ref> --status todo`.

use anyhow::Result;

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;
use agt_lib::schema::Status;

use super::{load_project, parse_ref, save_project};

pub fn run(references: Vec<String>) -> Result<()> {
    if references.is_empty() {
        anyhow::bail!("No todo references provided. Usage: agt queue <ref> [<ref>...]");
    }

    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);

    for reference in &references {
        let num = parse_ref(reference, &prefix)?;
        let todo = queries::find_todo_by_number(&doc, num)
            .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

        if todo.status == Status::Todo {
            eprintln!("{}-{} is already queued", prefix, num);
            continue;
        }

        operations::update_todo(
            &mut doc,
            num,
            UpdateTodoFields {
                status: Some(Status::Todo),
                ..Default::default()
            },
            None,
        )?;
        println!("Queued {}-{}: \"{}\"", prefix, num, todo.title);
    }

    save_project(&paths, &mut doc)?;
    Ok(())
}

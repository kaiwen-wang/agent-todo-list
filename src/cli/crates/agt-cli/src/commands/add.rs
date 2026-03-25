use anyhow::Result;

use agt_lib::operations::{self, AddTodoOpts};
use agt_lib::queries;
use agt_lib::schema::*;

use super::{load_project, save_project};

pub fn run(
    title: String,
    priority: Option<String>,
    status: Option<String>,
    difficulty: Option<String>,
    assignee: Option<String>,
    description: Option<String>,
    labels: Option<String>,
    json: bool,
) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let priority: Option<Priority> = priority
        .map(|p| p.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let status: Option<Status> = status
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let difficulty: Option<Difficulty> = difficulty
        .map(|d| d.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let labels: Option<Vec<Label>> = labels
        .map(|l| {
            l.split(',')
                .map(|s| s.trim().parse::<Label>())
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // Resolve assignee
    let assignee_id = if let Some(name) = &assignee {
        let member = queries::find_member(&doc, name)
            .ok_or_else(|| anyhow::anyhow!("Member \"{}\" not found", name))?;
        Some(member.id)
    } else {
        None
    };

    let number = operations::add_todo(
        &mut doc,
        AddTodoOpts {
            title: &title,
            description: description.as_deref(),
            status,
            priority,
            difficulty,
            labels,
            assignee: assignee_id.as_deref(),
            created_by: None,
            platform: Some(Platform::Cli),
        },
    )?;

    save_project(&paths, &mut doc)?;

    let (_, prefix, _, _) = queries::read_project_meta(&doc);

    if json {
        let todo = queries::find_todo_by_number(&doc, number);
        println!("{}", serde_json::to_string_pretty(&todo)?);
    } else {
        println!("Created {}-{}: {}", prefix, number, title);
    }

    Ok(())
}

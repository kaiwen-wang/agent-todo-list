use anyhow::Result;

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;
use agt_lib::schema::*;

use super::{load_project, parse_ref, save_project};

pub fn run(
    reference: String,
    title: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    difficulty: Option<String>,
    description: Option<String>,
    labels: Option<String>,
    cycle: Option<String>,
    json: bool,
) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let status: Option<Status> = status
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let priority: Option<Priority> = priority
        .map(|p| p.parse())
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

    // Resolve cycle: "none" clears it, otherwise look up by name/id
    let cycle_id: Option<Option<String>> = if let Some(ref name) = cycle {
        if name.eq_ignore_ascii_case("none") {
            Some(None)
        } else {
            let c = queries::find_cycle(&doc, name)
                .ok_or_else(|| anyhow::anyhow!("Cycle \"{}\" not found", name))?;
            Some(Some(c.id))
        }
    } else {
        None
    };

    // Convert cycle_id to the right lifetime shape
    let cycle_id_ref: Option<Option<&str>> = cycle_id.as_ref().map(|opt| opt.as_deref());

    operations::update_todo(
        &mut doc,
        num,
        UpdateTodoFields {
            title: title.as_deref(),
            description: description.as_deref(),
            status,
            priority,
            difficulty,
            labels,
            cycle_id: cycle_id_ref,
            ..Default::default()
        },
        None,
    )?;

    save_project(&paths, &mut doc)?;

    if json {
        let todo = queries::find_todo_by_number(&doc, num);
        println!("{}", serde_json::to_string_pretty(&todo)?);
    } else {
        println!("Updated {}-{}", prefix, num);
    }

    Ok(())
}

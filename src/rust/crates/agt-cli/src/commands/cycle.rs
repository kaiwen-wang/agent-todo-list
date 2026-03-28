use anyhow::Result;

use agt_lib::operations::{self, AddCycleOpts, UpdateCycleFields};
use agt_lib::queries;
use agt_lib::schema::*;

use super::{load_project, save_project};

pub fn create(
    name: String,
    description: Option<String>,
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    json: bool,
) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let status: Option<CycleStatus> = status
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let id = operations::add_cycle(
        &mut doc,
        AddCycleOpts {
            name: &name,
            description: description.as_deref(),
            status,
            start_date: start_date.as_deref(),
            end_date: end_date.as_deref(),
            created_by: None,
        },
    )?;

    save_project(&paths, &mut doc)?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "id": id, "name": name })
        );
    } else {
        println!("Created sprint: {} ({})", name, id);
    }
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let (_paths, doc) = load_project()?;
    let cycles = queries::read_all_cycles(&doc);
    let todos = queries::read_all_todos(&doc);

    if json {
        println!("{}", serde_json::to_string_pretty(&cycles)?);
    } else if cycles.is_empty() {
        println!("No sprints found. Create one with `agt sprint create \"name\"`.");
    } else {
        for c in &cycles {
            let count = todos
                .iter()
                .filter(|t| t.cycle_id.as_deref() == Some(&c.id))
                .count();
            let dates = match (&c.start_date, &c.end_date) {
                (Some(s), Some(e)) => format!("  {s} → {e}"),
                (Some(s), None) => format!("  {s} →"),
                _ => String::new(),
            };
            println!(
                "  {:<20} [{:<10}]  {} todo(s){}",
                c.name,
                c.status.display_name(),
                count,
                dates,
            );
        }
    }
    Ok(())
}

pub fn show(name_or_id: String, json: bool) -> Result<()> {
    let (_paths, doc) = load_project()?;
    let cycle = queries::find_cycle(&doc, &name_or_id)
        .ok_or_else(|| anyhow::anyhow!("Sprint \"{}\" not found", name_or_id))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&cycle)?);
    } else {
        println!("Sprint: {}", cycle.name);
        println!("  ID:          {}", cycle.id);
        println!("  Status:      {}", cycle.status.display_name());
        println!(
            "  Description: {}",
            if cycle.description.is_empty() {
                "(none)"
            } else {
                &cycle.description
            }
        );
        if let Some(s) = &cycle.start_date {
            println!("  Start:       {}", s);
        }
        if let Some(e) = &cycle.end_date {
            println!("  End:         {}", e);
        }

        let todos = queries::read_all_todos(&doc);
        let cycle_todos: Vec<_> = todos
            .iter()
            .filter(|t| t.cycle_id.as_deref() == Some(&cycle.id))
            .collect();
        println!("  Todos:       {}", cycle_todos.len());
        let (_, prefix, _, _) = queries::read_project_meta(&doc);
        for t in &cycle_todos {
            println!(
                "    {}-{}: {} [{}]",
                prefix,
                t.number,
                t.title,
                t.status.display_name()
            );
        }
    }
    Ok(())
}

pub fn edit(
    name_or_id: String,
    name: Option<String>,
    description: Option<String>,
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<()> {
    if name.is_none()
        && description.is_none()
        && status.is_none()
        && start_date.is_none()
        && end_date.is_none()
    {
        anyhow::bail!(
            "No updates specified. Use --name, --description, --status, --start-date, or --end-date."
        );
    }

    let (paths, mut doc) = load_project()?;

    let cycle = queries::find_cycle(&doc, &name_or_id)
        .ok_or_else(|| anyhow::anyhow!("Sprint \"{}\" not found", name_or_id))?;

    let status: Option<CycleStatus> = status
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    operations::update_cycle(
        &mut doc,
        &cycle.id,
        UpdateCycleFields {
            name: name.as_deref(),
            description: description.as_deref(),
            status,
            start_date: start_date.as_deref().map(Some),
            end_date: end_date.as_deref().map(Some),
        },
        None,
    )?;

    save_project(&paths, &mut doc)?;
    println!("Updated sprint: {}", name.as_deref().unwrap_or(&cycle.name));
    Ok(())
}

pub fn delete(name_or_id: String, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let cycle = queries::find_cycle(&doc, &name_or_id)
        .ok_or_else(|| anyhow::anyhow!("Sprint \"{}\" not found", name_or_id))?;

    let cycle_name = cycle.name.clone();
    operations::delete_cycle(&mut doc, &cycle.id, None)?;

    save_project(&paths, &mut doc)?;

    if json {
        println!("{}", serde_json::json!({ "ok": true, "name": cycle_name }));
    } else {
        println!("Deleted sprint: {}", cycle_name);
    }
    Ok(())
}

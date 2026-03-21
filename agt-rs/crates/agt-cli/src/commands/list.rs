use anyhow::Result;

use agt_lib::queries::{self, TodoFilter};

use super::load_project;
use crate::output;

pub fn run(
    status: Option<String>,
    assignee: Option<String>,
    priority: Option<String>,
    search: Option<String>,
    json: bool,
) -> Result<()> {
    let (_paths, doc) = load_project()?;

    let filter = TodoFilter {
        status: status.map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect()),
        priority: priority.map(|p| p.split(',').filter_map(|v| v.trim().parse().ok()).collect()),
        assignee,
        search,
        ..Default::default()
    };

    let todos = queries::query_todos(&doc, &filter);

    if json {
        println!("{}", serde_json::to_string_pretty(&todos)?);
    } else {
        let (_, prefix, _, _) = queries::read_project_meta(&doc);
        let members = queries::read_all_members(&doc);
        output::print_todo_list(&todos, &prefix, &members);
    }

    Ok(())
}

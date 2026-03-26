use anyhow::Result;

use agt_lib::queries::{self, TodoFilter};
use agt_lib::schema::Status;

use super::load_project;
use crate::output;

pub fn run(
    status: Option<String>,
    assignee: Option<String>,
    priority: Option<String>,
    difficulty: Option<String>,
    search: Option<String>,
    all: bool,
    archived: bool,
    rank: bool,
    json: bool,
) -> Result<()> {
    let (_paths, doc) = load_project()?;

    let status_filter = if let Some(s) = status {
        Some(s.split(',').filter_map(|v| v.trim().parse().ok()).collect())
    } else if archived {
        Some(vec![Status::Archived])
    } else if all {
        Some(Status::ALL.to_vec())
    } else {
        None
    };

    let filter = TodoFilter {
        status: status_filter,
        priority: priority.map(|p| p.split(',').filter_map(|v| v.trim().parse().ok()).collect()),
        difficulty: difficulty.map(|d| d.split(',').filter_map(|v| v.trim().parse().ok()).collect()),
        assignee,
        search,
    };

    let mut todos = queries::query_todos(&doc, &filter);

    if rank {
        queries::rank_todos(&mut todos);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&todos)?);
    } else {
        let (_, prefix, _, _) = queries::read_project_meta(&doc);
        let members = queries::read_all_members(&doc);
        output::print_todo_list(&todos, &prefix, &members);
    }

    Ok(())
}

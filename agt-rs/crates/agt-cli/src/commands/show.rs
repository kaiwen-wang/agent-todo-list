use anyhow::{bail, Result};

use agt_lib::queries;

use super::{load_project, parse_ref};
use crate::output;

pub fn run(reference: String, json: bool) -> Result<()> {
    let (_paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num);
    let Some(todo) = todo else {
        bail!("Todo {}-{} not found", prefix, num);
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&todo)?);
    } else {
        let members = queries::read_all_members(&doc);
        output::print_todo_detail(&todo, &prefix, &members);
    }

    Ok(())
}

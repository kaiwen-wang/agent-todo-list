use anyhow::Result;

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String, member_name: String, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let member = queries::find_member(&doc, &member_name)
        .ok_or_else(|| anyhow::anyhow!("Member \"{}\" not found", member_name))?;

    let assignee_name = member.name.clone();

    operations::update_todo(
        &mut doc,
        num,
        UpdateTodoFields {
            assignee: Some(Some(member.id.as_str())),
            ..Default::default()
        },
        None,
    )?;

    save_project(&paths, &mut doc)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "reference": format!("{}-{}", prefix, num),
                "assignee": assignee_name,
            })
        );
    } else {
        println!("Assigned {}-{} to {}", prefix, num, assignee_name);
    }
    Ok(())
}

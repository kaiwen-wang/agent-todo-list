use anyhow::Result;

use agt_lib::operations;
use agt_lib::queries;
use agt_lib::schema::*;

use super::{load_project, save_project};
use crate::output;

pub fn add(
    name: String,
    role: String,
    email: Option<String>,
    provider: Option<String>,
    model: Option<String>,
) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let role: MemberRole = role.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    let provider: Option<AgentProvider> = provider
        .map(|p| p.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    operations::add_member(
        &mut doc,
        &name,
        role,
        email.as_deref(),
        None,
        provider,
        model.as_deref(),
    )?;

    save_project(&paths, &mut doc)?;
    println!("Added member: {} ({})", name, role.as_str());
    Ok(())
}

pub fn list(json: bool) -> Result<()> {
    let (_paths, doc) = load_project()?;
    let members = queries::read_all_members(&doc);

    if json {
        println!("{}", serde_json::to_string_pretty(&members)?);
    } else {
        output::print_members_list(&members);
    }

    Ok(())
}

pub fn remove(name: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let member = queries::find_member(&doc, &name)
        .ok_or_else(|| anyhow::anyhow!("Member \"{}\" not found", name))?;

    let member_name = member.name.clone();
    operations::remove_member(&mut doc, &member.id, None)?;

    save_project(&paths, &mut doc)?;
    println!("Removed member: {}", member_name);
    Ok(())
}

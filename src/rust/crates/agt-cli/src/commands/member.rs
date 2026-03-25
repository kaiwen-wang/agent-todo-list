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
    json: bool,
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

    if json {
        let member = queries::find_member(&doc, &name);
        let id = member.map(|m| m.id.clone()).unwrap_or_default();
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "name": name,
                "id": id,
            })
        );
    } else {
        println!("Added member: {} ({})", name, role.as_str());
    }
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

pub fn remove(name: String, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;

    let member = queries::find_member(&doc, &name)
        .ok_or_else(|| anyhow::anyhow!("Member \"{}\" not found", name))?;

    let member_name = member.name.clone();
    operations::remove_member(&mut doc, &member.id, None)?;

    save_project(&paths, &mut doc)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "name": member_name,
            })
        );
    } else {
        println!("Removed member: {}", member_name);
    }
    Ok(())
}

pub fn update(
    name: String,
    new_name: Option<String>,
    role: Option<String>,
    email: Option<String>,
    provider: Option<String>,
    model: Option<String>,
) -> Result<()> {
    if new_name.is_none() && role.is_none() && email.is_none() && provider.is_none() && model.is_none() {
        anyhow::bail!("No updates specified. Use --name, --role, --email, --provider, or --model.");
    }

    let (paths, mut doc) = load_project()?;

    let member = queries::find_member(&doc, &name)
        .ok_or_else(|| anyhow::anyhow!("Member \"{}\" not found", name))?;

    let role: Option<MemberRole> = role
        .map(|r| r.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    let provider: Option<AgentProvider> = provider
        .map(|p| p.parse())
        .transpose()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let member_name = member.name.clone();
    let email_update: Option<Option<&str>> = email.as_deref().map(Some);

    operations::update_member(
        &mut doc,
        &member.id,
        new_name.as_deref(),
        email_update,
        role,
        provider,
        model.as_deref(),
        None,
    )?;

    save_project(&paths, &mut doc)?;
    println!("Updated member: {}", new_name.as_deref().unwrap_or(&member_name));
    Ok(())
}

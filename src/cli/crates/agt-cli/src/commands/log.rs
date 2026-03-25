use anyhow::Result;
use colored::Colorize;

use agt_lib::history;

use super::load_project;

pub fn run(limit: Option<usize>, json: bool) -> Result<()> {
    let (_paths, mut doc) = load_project()?;
    let entries = history::get_audit_log(&mut doc, limit, None);

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    if entries.is_empty() {
        println!("{}", "No audit log entries.".dimmed());
        return Ok(());
    }

    for entry in &entries {
        let ts = chrono::DateTime::from_timestamp_millis(entry.timestamp)
            .map(|dt: chrono::DateTime<chrono::Utc>| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        println!(
            "{} {} {} {}",
            ts.dimmed(),
            entry.actor_name.bold(),
            entry.action,
            entry.target.cyan()
        );
    }

    Ok(())
}

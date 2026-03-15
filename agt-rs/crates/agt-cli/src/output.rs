//! Terminal formatting helpers.

use agt_lib::schema::*;
use colored::Colorize;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED, ContentArrangement};

/// Format a status with color.
pub fn colored_status(status: &Status) -> String {
    let s = status.display_name();
    match status {
        Status::None => s.dimmed().to_string(),
        Status::Todo => s.yellow().to_string(),
        Status::InProgress => s.blue().to_string(),
        Status::Completed => s.green().to_string(),
        Status::Archived => s.dimmed().to_string(),
        Status::WontDo => s.red().to_string(),
        Status::NeedsElaboration => s.magenta().to_string(),
    }
}

/// Format a priority with color.
pub fn colored_priority(priority: &Priority) -> String {
    let s = priority.display_name();
    match priority {
        Priority::None => s.dimmed().to_string(),
        Priority::Low => s.to_string(),
        Priority::Medium => s.yellow().to_string(),
        Priority::High => s.red().to_string(),
        Priority::Urgent => s.red().bold().to_string(),
    }
}

/// Print a table of todos.
pub fn print_todo_table(todos: &[Todo], prefix: &str, members: &[Member]) {
    if todos.is_empty() {
        println!("{}", "No todos found.".dimmed());
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Ref", "Title", "Status", "Priority", "Assignee"]);

    for todo in todos {
        let todo_ref = format!("{}-{}", prefix, todo.number);
        let assignee = todo
            .assignee
            .as_ref()
            .and_then(|a| members.iter().find(|m| m.id == *a))
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "—".dimmed().to_string());

        table.add_row(vec![
            todo_ref,
            todo.title.clone(),
            colored_status(&todo.status),
            colored_priority(&todo.priority),
            assignee,
        ]);
    }

    println!("{table}");
}

/// Print detailed view of a single todo.
pub fn print_todo_detail(todo: &Todo, prefix: &str, members: &[Member]) {
    let todo_ref = format!("{}-{}", prefix, todo.number);
    println!("{}", todo_ref.bold());
    println!("  {} {}", "Title:".dimmed(), todo.title);
    println!("  {} {}", "Status:".dimmed(), colored_status(&todo.status));
    println!("  {} {}", "Priority:".dimmed(), colored_priority(&todo.priority));
    println!("  {} {}", "Difficulty:".dimmed(), todo.difficulty.display_name());

    if !todo.labels.is_empty() {
        let labels: Vec<&str> = todo.labels.iter().map(|l| l.display_name()).collect();
        println!("  {} {}", "Labels:".dimmed(), labels.join(", "));
    }

    let assignee = todo
        .assignee
        .as_ref()
        .and_then(|a| members.iter().find(|m| m.id == *a))
        .map(|m| m.name.as_str())
        .unwrap_or("—");
    println!("  {} {}", "Assignee:".dimmed(), assignee);

    if let Some(branch) = &todo.branch {
        println!("  {} {}", "Branch:".dimmed(), branch);
    }

    if !todo.description.is_empty() {
        println!("\n  {}", "Description:".dimmed());
        for line in todo.description.lines() {
            println!("    {line}");
        }
    }

    if !todo.comments.is_empty() {
        println!("\n  {} ({})", "Comments:".dimmed(), todo.comments.len());
        for comment in &todo.comments {
            let ts = chrono::DateTime::from_timestamp_millis(comment.created_at)
                .map(|dt: chrono::DateTime<chrono::Utc>| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default();
            println!("    {} {} {}", comment.author_name.bold(), "at".dimmed(), ts.dimmed());
            println!("    {}", comment.text);
            println!();
        }
    }
}

/// Print members table.
pub fn print_members_table(members: &[Member]) {
    if members.is_empty() {
        println!("{}", "No members found.".dimmed());
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Role", "Email"]);

    for member in members {
        table.add_row(vec![
            member.name.clone(),
            member.role.as_str().to_string(),
            member.email.clone().unwrap_or_else(|| "—".to_string()),
        ]);
    }

    println!("{table}");
}

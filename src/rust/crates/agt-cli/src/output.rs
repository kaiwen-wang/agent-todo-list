//! Terminal formatting helpers.
//! Matches the original TypeScript CLI's compact line-per-todo format.

use agt_lib::git::CommitInfo;
use agt_lib::schema::*;
use colored::Colorize;

// ── Status ──────────────────────────────────────────────────────────

fn status_icon(status: &Status) -> &'static str {
    match status {
        Status::None => " ",
        Status::Todo => " ",
        Status::InProgress => "*",
        Status::Paused => "|",
        Status::Completed => "x",
        Status::Archived => "-",
        Status::WontDo => "~",
        Status::NeedsElaboration => "?",
    }
}

fn colored_title(title: &str, status: &Status) -> String {
    match status {
        Status::None => title.dimmed().to_string(),
        Status::Todo => title.white().to_string(),
        Status::InProgress => title.cyan().to_string(),
        Status::Paused => title.yellow().to_string(),
        Status::Completed => title.green().to_string(),
        Status::Archived => title.dimmed().to_string(),
        Status::WontDo => title.strikethrough().to_string(),
        Status::NeedsElaboration => title.magenta().to_string(),
    }
}

pub fn colored_status(status: &Status) -> String {
    let s = status.as_str().replace('_', " ");
    match status {
        Status::None => s.dimmed().to_string(),
        Status::Todo => s.white().to_string(),
        Status::InProgress => s.cyan().to_string(),
        Status::Paused => s.yellow().to_string(),
        Status::Completed => s.green().to_string(),
        Status::Archived => s.dimmed().to_string(),
        Status::WontDo => s.strikethrough().to_string(),
        Status::NeedsElaboration => s.magenta().to_string(),
    }
}

// ── Priority ────────────────────────────────────────────────────────

fn priority_label(priority: &Priority) -> &'static str {
    match priority {
        Priority::None => "",
        Priority::Low => "low",
        Priority::Medium => "med",
        Priority::High => "high",
        Priority::Urgent => "URGENT",
    }
}

fn colored_priority_short(priority: &Priority) -> String {
    let label = format!("P:{}", priority_label(priority).to_uppercase());
    match priority {
        Priority::None => String::new(),
        Priority::Low => label.dimmed().to_string(),
        Priority::Medium => label.white().to_string(),
        Priority::High => label.yellow().to_string(),
        Priority::Urgent => label.red().bold().to_string(),
    }
}

pub fn colored_priority(priority: &Priority) -> String {
    let label = priority_label(priority);
    match priority {
        Priority::None => label.dimmed().to_string(),
        Priority::Low => label.dimmed().to_string(),
        Priority::Medium => label.white().to_string(),
        Priority::High => label.yellow().to_string(),
        Priority::Urgent => label.red().bold().to_string(),
    }
}

// ── Difficulty ──────────────────────────────────────────────────────

fn difficulty_label(difficulty: &Difficulty) -> &'static str {
    match difficulty {
        Difficulty::None => "",
        Difficulty::Easy => "easy",
        Difficulty::Medium => "medium",
        Difficulty::Hard => "hard",
    }
}

fn colored_difficulty_short(difficulty: &Difficulty) -> String {
    let label = format!("D:{}", difficulty_label(difficulty).to_uppercase());
    match difficulty {
        Difficulty::None => String::new(),
        Difficulty::Easy => label.green().to_string(),
        Difficulty::Medium => label.yellow().to_string(),
        Difficulty::Hard => label.red().to_string(),
    }
}

// ── List output ─────────────────────────────────────────────────────

/// Print todos as compact one-line-per-item (matching the original TS CLI).
pub fn print_todo_list(todos: &[Todo], prefix: &str, members: &[Member]) {
    if todos.is_empty() {
        println!("{}", "No todos found.".dimmed());
        return;
    }

    for todo in todos {
        let icon = status_icon(&todo.status);
        let todo_ref = format!("{}-{}", prefix, todo.number).bold().to_string();
        let title = colored_title(&todo.title, &todo.status);

        let priority = if todo.priority != Priority::None {
            format!(" {}", colored_priority_short(&todo.priority))
        } else {
            String::new()
        };

        let difficulty = if todo.difficulty != Difficulty::None {
            format!(" {}", colored_difficulty_short(&todo.difficulty))
        } else {
            String::new()
        };

        let assignee = todo
            .assignee
            .as_ref()
            .and_then(|a| members.iter().find(|m| m.id == *a))
            .map(|m| format!(" {}", format!("@{}", m.name).dimmed()))
            .unwrap_or_default();

        println!("[{icon}] {todo_ref} {title}{priority}{difficulty}{assignee}");
    }
}

/// Print detailed view of a single todo.
pub fn print_todo_detail(
    todo: &Todo,
    prefix: &str,
    members: &[Member],
    branch_commits: &[CommitInfo],
    linked_commits: &[CommitInfo],
) {
    let todo_ref = format!("{}-{}", prefix, todo.number);
    println!("{}", format!("{}: {}", todo_ref, todo.title).bold());
    println!();
    println!(
        "  {}   {}",
        "Status:".dimmed(),
        colored_status(&todo.status)
    );
    println!(
        "  {} {}",
        "Priority:".dimmed(),
        colored_priority(&todo.priority)
    );

    if todo.difficulty != Difficulty::None {
        println!(
            "  {} {}",
            "Difficulty:".dimmed(),
            colored_difficulty_short(&todo.difficulty)
        );
    }

    if let Some(assignee_id) = &todo.assignee {
        let name = members
            .iter()
            .find(|m| m.id == *assignee_id)
            .map(|m| m.name.as_str())
            .unwrap_or(assignee_id);
        println!("  {} {}", "Assignee:".dimmed(), name);
    }

    if let Some(branch) = &todo.branch {
        println!("  {}   {}", "Branch:".dimmed(), branch.cyan());
    }

    if !todo.worktrees.is_empty() {
        for (i, wt) in todo.worktrees.iter().enumerate() {
            if i == 0 {
                println!("  {} {}", "Worktree:".dimmed(), wt.cyan());
            } else {
                println!("           {}", wt.cyan());
            }
        }
    }

    // Show commits: branch commits first, then manually linked
    let has_branch_commits = !branch_commits.is_empty();
    let has_linked_commits = !linked_commits.is_empty();

    if has_branch_commits || has_linked_commits {
        println!("  {}  ", "Commits:".dimmed());
        for c in branch_commits {
            let url_hint = c
                .url
                .as_deref()
                .map(|u| format!(" {}", u.dimmed()))
                .unwrap_or_default();
            println!(
                "           {} {}{}",
                c.short_sha.cyan(),
                c.subject,
                url_hint
            );
        }
        for c in linked_commits {
            let url_hint = c
                .url
                .as_deref()
                .map(|u| format!(" {}", u.dimmed()))
                .unwrap_or_default();
            println!(
                "           {} {} {}{}",
                c.short_sha.cyan(),
                c.subject,
                "(linked)".dimmed(),
                url_hint
            );
        }
    }

    if let Some(plan_path) = &todo.plan_path {
        println!("  {}     {}", "Plan:".dimmed(), plan_path.cyan());
    }

    if !todo.labels.is_empty() {
        let labels: Vec<&str> = todo.labels.iter().map(|l| l.display_name()).collect();
        println!("  {}   {}", "Labels:".dimmed(), labels.join(", "));
    }

    let created = format_ts(todo.created_at);
    println!("  {}  {}", "Created:".dimmed(), created);
    if todo.updated_at != todo.created_at {
        println!("  {}  {}", "Updated:".dimmed(), format_ts(todo.updated_at));
    }

    if !todo.description.is_empty() {
        println!();
        println!("{}", todo.description);
    }

    let comments = &todo.comments;
    if !comments.is_empty() {
        println!();
        println!(
            "{}",
            format!("--- Comments ({}) ---", comments.len()).dimmed()
        );
        for comment in comments {
            let ts = format_ts(comment.created_at);
            println!("  {} {}", comment.author_name.bold(), ts.dimmed());
            println!("  {}", comment.text);
            println!();
        }
    }
}

/// Print members list.
pub fn print_members_list(members: &[Member]) {
    if members.is_empty() {
        println!("{}", "No members found.".dimmed());
        return;
    }

    for member in members {
        let role = match member.role {
            MemberRole::Owner => "owner".yellow().to_string(),
            MemberRole::Member => "member".normal().to_string(),
            MemberRole::Agent => "agent".cyan().to_string(),
        };
        let email = member
            .email
            .as_deref()
            .map(|e| format!(" {}", e.dimmed()))
            .unwrap_or_default();
        println!("{} ({}){}", member.name.bold(), role, email);
    }
}

fn format_ts(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt: chrono::DateTime<chrono::Utc>| dt.format("%b %d, %Y %H:%M").to_string())
        .unwrap_or_default()
}

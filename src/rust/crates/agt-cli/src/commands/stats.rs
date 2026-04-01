use anyhow::Result;
use colored::Colorize;

use agt_lib::queries;
use agt_lib::stats::compute_stats;

use super::load_project;

pub fn run(json: bool) -> Result<()> {
    let (_paths, doc) = load_project()?;
    let stats = compute_stats(&doc);

    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    let (_, prefix, name, _) = queries::read_project_meta(&doc);

    println!("{} {}", name.bold(), format!("[{}]", prefix).dimmed());
    println!();

    // Summary
    println!("{}", "Summary".bold().underline());
    println!(
        "  Total: {}  Active: {}  In Progress: {}  Completed: {}  Unassigned: {}  Closed: {}%",
        stats.summary.total.to_string().bold(),
        stats.summary.active.to_string().bold(),
        stats.summary.in_progress.to_string().cyan().bold(),
        stats.summary.completed.to_string().green().bold(),
        stats.summary.unassigned.to_string().yellow().bold(),
        stats.summary.completion_rate.to_string().bold(),
    );
    println!();

    // Status breakdown
    if !stats.by_status.is_empty() {
        println!("{}", "By Status".bold().underline());
        for entry in &stats.by_status {
            let bar = bar_chart(entry.count, stats.summary.total, 20);
            println!("  {:<20} {} {}", entry.label, bar, entry.count);
        }
        println!();
    }

    // Priority breakdown
    if !stats.by_priority.is_empty() {
        println!("{}", "By Priority".bold().underline());
        let active_total = stats.summary.active;
        for entry in &stats.by_priority {
            let bar = bar_chart(entry.count, active_total, 20);
            println!("  {:<20} {} {}", entry.label, bar, entry.count);
        }
        println!();
    }

    // Difficulty breakdown
    if !stats.by_difficulty.is_empty() {
        println!("{}", "By Difficulty".bold().underline());
        let active_total = stats.summary.active;
        for entry in &stats.by_difficulty {
            let bar = bar_chart(entry.count, active_total, 20);
            println!("  {:<20} {} {}", entry.label, bar, entry.count);
        }
        println!();
    }

    // Labels
    if !stats.by_label.is_empty() {
        println!("{}", "Labels".bold().underline());
        let label_total: usize = stats.by_label.iter().map(|e| e.count).sum();
        for entry in &stats.by_label {
            let bar = bar_chart(entry.count, label_total, 20);
            println!("  {:<20} {} {}", entry.label, bar, entry.count);
        }
        println!();
    }

    // Member workload
    if !stats.members.is_empty() {
        println!("{}", "Member Workload".bold().underline());
        for m in &stats.members {
            println!(
                "  {:<20} {} active, {} done ({} total)",
                m.name,
                m.active.to_string().cyan(),
                m.completed.to_string().green(),
                m.total,
            );
        }
        println!();
    }

    // Sprint progress
    if !stats.cycles.is_empty() {
        println!("{}", "Sprint Progress".bold().underline());
        for c in &stats.cycles {
            let bar = bar_chart(c.completed, c.total, 20);
            let days = match (c.days_elapsed, c.days_total) {
                (Some(e), Some(t)) => format!(" (day {}/{})", e, t),
                _ => String::new(),
            };
            println!(
                "  {:<20} {} {}/{} ({}%){} [{}]",
                c.name, bar, c.completed, c.total, c.pct_done, days, c.status,
            );
        }
        println!();
    }

    Ok(())
}

fn bar_chart(value: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("[{}]", " ".repeat(width));
    }
    let filled = ((value as f64 / total as f64) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

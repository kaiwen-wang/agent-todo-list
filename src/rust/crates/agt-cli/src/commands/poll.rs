//! `agt poll` — Cron-compatible dispatcher.
//!
//! Runs once: checks for todo-status tasks, spawns `agt run` for each eligible one,
//! reaps dead runs, exits.
//!
//! Designed to be called from cron/launchd/systemd timer every 10-60 seconds.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::process::{Command, Stdio};

use agt_lib::queries;
use agt_lib::schema::Status;

use super::load_project;
use super::run::RunState;
use super::workflow;

/// Check if a process with the given PID is still alive.
fn is_pid_alive(pid: u32) -> bool {
    // kill(pid, 0) checks process existence without sending a signal
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

pub fn run(dry_run: bool) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);

    // Load workflow config for concurrency limit
    let workflow = workflow::load_workflow(&paths.todo_dir)?;
    let max_concurrent = workflow.config.max_concurrent;

    let runs_dir = paths.todo_dir.join("runs");

    // ── Step 1: Reap dead runs ──────────────────────────────────────
    let mut live_runs: HashSet<u64> = HashSet::new();
    let mut live_count: usize = 0;

    if runs_dir.exists() {
        for entry in fs::read_dir(&runs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => {
                    // Can't read — clean it up
                    let _ = fs::remove_file(&path);
                    continue;
                }
            };

            let state: RunState = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => {
                    // Invalid JSON — clean it up
                    let _ = fs::remove_file(&path);
                    continue;
                }
            };

            if is_pid_alive(state.pid) {
                live_runs.insert(state.todo_number);
                live_count += 1;
            } else {
                // Process is dead — remove stale run file
                eprintln!(
                    "Reaped dead run: {} (pid {} no longer alive)",
                    state.todo_ref, state.pid
                );
                let _ = fs::remove_file(&path);
            }
        }
    }

    // ── Step 2: Find eligible todos (status = todo) ─────────────────
    let filter = queries::TodoFilter {
        status: Some(vec![Status::Todo]),
        ..Default::default()
    };
    let mut candidates = queries::query_todos(&doc, &filter);

    // Filter out todos that already have a live run
    candidates.retain(|t| !live_runs.contains(&t.number));

    // Sort: priority ascending (urgent=4 > high=3 > med=2 > low=1), then oldest first
    candidates.sort_by(|a, b| {
        // Map priority to numeric value for sorting (higher = more urgent)
        fn priority_rank(p: &agt_lib::schema::Priority) -> u8 {
            match p {
                agt_lib::schema::Priority::Urgent => 0,
                agt_lib::schema::Priority::High => 1,
                agt_lib::schema::Priority::Medium => 2,
                agt_lib::schema::Priority::Low => 3,
                agt_lib::schema::Priority::None => 4,
            }
        }
        priority_rank(&a.priority)
            .cmp(&priority_rank(&b.priority))
            .then(a.created_at.cmp(&b.created_at))
    });

    // ── Step 3: Dispatch ────────────────────────────────────────────
    let available_slots = max_concurrent.saturating_sub(live_count);
    let to_dispatch = &candidates[..candidates.len().min(available_slots)];

    if dry_run {
        println!("=== Dry run ===");
        println!("Running: {}", live_count);
        println!("Max concurrent: {}", max_concurrent);
        println!("Available slots: {}", available_slots);
        println!("Todo candidates: {}", candidates.len());
        println!();
        for todo in to_dispatch {
            println!(
                "Would dispatch: {}-{} \"{}\" (priority: {}, created: {})",
                prefix,
                todo.number,
                todo.title,
                todo.priority.as_str(),
                chrono::DateTime::from_timestamp_millis(todo.created_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default(),
            );
        }
        return Ok(());
    }

    let mut dispatched = 0;

    for todo in to_dispatch {
        let todo_ref = format!("{}-{}", prefix, todo.number);

        // Spawn `agt run <ref>` as a detached background process
        let exe = std::env::current_exe().context("Failed to get current executable path")?;

        // Create log dir for stdout/stderr capture
        let logs_dir = paths.todo_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;

        let stdout_path = logs_dir.join(format!(
            "{}-poll-{}.stdout",
            todo_ref,
            chrono::Utc::now().timestamp()
        ));
        let stderr_path = logs_dir.join(format!(
            "{}-poll-{}.stderr",
            todo_ref,
            chrono::Utc::now().timestamp()
        ));

        let stdout_file = fs::File::create(&stdout_path)?;
        let stderr_file = fs::File::create(&stderr_path)?;

        match Command::new(&exe)
            .args(["run", &todo_ref])
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
        {
            Ok(child) => {
                eprintln!(
                    "Dispatched {}: \"{}\" (pid: {})",
                    todo_ref,
                    todo.title,
                    child.id()
                );
                dispatched += 1;
            }
            Err(e) => {
                eprintln!("Failed to dispatch {}: {}", todo_ref, e);
            }
        }
    }

    // ── Summary ─────────────────────────────────────────────────────
    let remaining = candidates.len() - dispatched;
    eprintln!(
        "Dispatched: {}, Running: {}, Remaining: {}",
        dispatched,
        live_count + dispatched,
        remaining,
    );

    Ok(())
}

//! `agt runs` — List active and recent agent runs.

use anyhow::Result;
use std::fs;

use super::load_project;
use super::run::RunState;

/// Check if a process with the given PID is still alive.
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

pub fn run(json: bool) -> Result<()> {
    let (paths, _doc) = load_project()?;
    let runs_dir = paths.todo_dir.join("runs");

    let mut live_runs: Vec<RunState> = Vec::new();
    let mut dead_runs: Vec<RunState> = Vec::new();

    if runs_dir.exists() {
        for entry in fs::read_dir(&runs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let state: RunState = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if is_pid_alive(state.pid) {
                live_runs.push(state);
            } else {
                dead_runs.push(state);
                // Clean up stale run file
                let _ = fs::remove_file(&path);
            }
        }
    }

    if json {
        let output = serde_json::json!({
            "running": live_runs,
            "stale_removed": dead_runs.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if live_runs.is_empty() && dead_runs.is_empty() {
        println!("No active or recent runs.");
        return Ok(());
    }

    if !live_runs.is_empty() {
        println!("RUNNING:");
        for r in &live_runs {
            let elapsed = elapsed_str(r.started_at);
            println!(
                "  {}  pid={}  {}  {}  {}",
                r.todo_ref, r.pid, elapsed, r.agent, r.workspace_path,
            );
        }
    }

    if !dead_runs.is_empty() {
        println!();
        println!("STALE (cleaned up):");
        for r in &dead_runs {
            println!("  {}  pid={} (dead)  {}", r.todo_ref, r.pid, r.agent,);
        }
    }

    Ok(())
}

fn elapsed_str(started_at_ms: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed_secs = (now - started_at_ms) / 1000;
    if elapsed_secs < 60 {
        format!("{}s ago", elapsed_secs)
    } else if elapsed_secs < 3600 {
        format!("{}m ago", elapsed_secs / 60)
    } else {
        format!("{}h ago", elapsed_secs / 3600)
    }
}

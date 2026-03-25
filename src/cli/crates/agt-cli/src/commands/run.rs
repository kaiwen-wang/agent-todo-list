//! `agt run <ref>` — Run a coding agent against a single todo.
//!
//! 1. Load todo context
//! 2. Ensure worktree exists
//! 3. Render prompt from workflow.md
//! 4. Set status to in_progress
//! 5. Spawn Claude CLI in the worktree
//! 6. Stream stdout, parse JSON events, log to .todo/logs/
//! 7. On exit: clean up run file

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;
use agt_lib::schema::Status;

use super::branch::ensure_worktree;
use super::workflow::{self, CommentContext, PromptContext};
use super::{load_project, parse_ref, save_project};

/// Run state file stored in .todo/runs/<REF>.json
#[derive(Debug, Serialize, Deserialize)]
pub struct RunState {
    pub todo_number: u64,
    pub todo_ref: String,
    pub pid: u32,
    pub workspace_path: String,
    pub started_at: i64,
    pub agent: String,
    pub attempt: u32,
}

/// Write a run state file.
fn write_run_state(todo_dir: &std::path::Path, state: &RunState) -> Result<()> {
    let runs_dir = todo_dir.join("runs");
    fs::create_dir_all(&runs_dir)?;
    let path = runs_dir.join(format!("{}.json", state.todo_ref));
    let json = serde_json::to_string_pretty(state)?;
    fs::write(&path, json)?;
    Ok(())
}

/// Remove a run state file.
fn remove_run_state(todo_dir: &std::path::Path, todo_ref: &str) {
    let path = todo_dir.join("runs").join(format!("{}.json", todo_ref));
    let _ = fs::remove_file(path);
}

/// Ensure .todo/logs/ exists and return a log file path.
fn open_log_file(todo_dir: &std::path::Path, todo_ref: &str) -> Result<(PathBuf, fs::File)> {
    let logs_dir = todo_dir.join("logs");
    fs::create_dir_all(&logs_dir)?;
    let timestamp = chrono::Utc::now().timestamp();
    let filename = format!("{}-{}.log", todo_ref, timestamp);
    let path = logs_dir.join(&filename);
    let file = fs::File::create(&path)
        .with_context(|| format!("Failed to create log file: {}", path.display()))?;
    Ok((path, file))
}

pub fn run(reference: String, budget: Option<f64>, dry_run: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, project_name, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    // Load workflow
    let workflow = workflow::load_workflow(&paths.todo_dir)?;
    let budget_usd = budget.unwrap_or(workflow.config.budget_usd);

    // Ensure worktree exists
    let (branch_name, worktree_path) = ensure_worktree(&paths.root, &prefix, &todo)?;

    // Build prompt context
    let members = queries::read_all_members(&doc);
    let assignee_name = todo
        .assignee
        .as_ref()
        .and_then(|a| members.iter().find(|m| m.id == *a).map(|m| m.name.as_str()));

    let label_strings: Vec<String> = todo.labels.iter().map(|l| l.as_str().to_string()).collect();
    let comment_contexts: Vec<CommentContext> = todo
        .comments
        .iter()
        .map(|c| CommentContext {
            author: &c.author_name,
            text: &c.text,
        })
        .collect();

    let ctx = PromptContext {
        project_name: &project_name,
        project_prefix: &prefix,
        todo_ref: todo_ref.clone(),
        todo_title: &todo.title,
        todo_description: &todo.description,
        todo_priority: todo.priority.as_str(),
        todo_difficulty: todo.difficulty.as_str(),
        todo_labels: &label_strings,
        todo_comments: comment_contexts,
        todo_assignee: assignee_name,
        attempt: None,
    };

    let prompt = workflow::render_prompt(&workflow.prompt_template, &ctx)?;

    if dry_run {
        println!("=== Rendered prompt ===");
        println!("{}", prompt);
        println!();
        println!("=== Would run ===");
        println!(
            "{} -p <prompt> --output-format stream-json --allowedTools \"{}\" --permission-mode bypassPermissions --max-budget-usd {}",
            workflow.config.agent_command,
            workflow.config.allowed_tools.join(" "),
            budget_usd,
        );
        println!("  cwd: {}", worktree_path.display());
        return Ok(());
    }

    // Record branch on todo if it wasn't set already
    if todo.branch.is_none() {
        operations::set_branch(&mut doc, num, &branch_name, None)?;
    }

    // Set status to in_progress
    operations::update_todo(
        &mut doc,
        num,
        UpdateTodoFields {
            status: Some(Status::InProgress),
            ..Default::default()
        },
        None,
    )?;
    save_project(&paths, &mut doc)?;

    // Write run state
    let run_state = RunState {
        todo_number: num,
        todo_ref: todo_ref.clone(),
        pid: std::process::id(),
        workspace_path: worktree_path.to_string_lossy().to_string(),
        started_at: chrono::Utc::now().timestamp_millis(),
        agent: workflow.config.agent_command.clone(),
        attempt: 1,
    };
    write_run_state(&paths.todo_dir, &run_state)?;

    // Open log file
    let (log_path, mut log_file) = open_log_file(&paths.todo_dir, &todo_ref)?;
    eprintln!("Log: {}", log_path.display());

    // Build the claude command
    let allowed_tools_str = workflow.config.allowed_tools.join(" ");

    let mut cmd = Command::new(&workflow.config.agent_command);
    cmd.arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--allowedTools")
        .arg(&allowed_tools_str)
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .arg("--max-budget-usd")
        .arg(format!("{:.2}", budget_usd))
        .current_dir(&worktree_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    eprintln!(
        "Running {} for {} in {}",
        workflow.config.agent_command,
        todo_ref,
        worktree_path.display()
    );

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn {}", workflow.config.agent_command))?;

    // Update run state with actual child PID
    let child_pid = child.id();
    let run_state = RunState {
        pid: child_pid,
        ..run_state
    };
    write_run_state(&paths.todo_dir, &run_state)?;

    // Stream stdout line by line
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut is_error = false;
    let mut result_text = String::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                writeln!(log_file, "[ERROR] Failed to read stdout: {}", e)?;
                break;
            }
        };

        // Write raw line to log
        writeln!(log_file, "{}", line)?;

        // Try to parse as JSON
        if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
            let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match event_type {
                "assistant" => {
                    // Extract text content for display
                    if let Some(content) = event
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            let block_type =
                                block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match block_type {
                                "text" => {
                                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                        // Only show first 200 chars of text blocks
                                        let preview = if text.len() > 200 {
                                            format!("{}...", &text[..200])
                                        } else {
                                            text.to_string()
                                        };
                                        eprintln!("  [agent] {}", preview);
                                    }
                                }
                                "tool_use" => {
                                    if let Some(input) = block.get("input") {
                                        if let Some(cmd_str) =
                                            input.get("command").and_then(|v| v.as_str())
                                        {
                                            eprintln!("  [tool]  $ {}", cmd_str);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "result" => {
                    is_error = event
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    result_text = event
                        .get("result")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
                _ => {
                    // Log but don't display other event types
                }
            }
        }
    }

    // Wait for process to finish
    let exit_status = child.wait()?;

    // Clean up run state
    remove_run_state(&paths.todo_dir, &todo_ref);

    // Reload doc (it may have been modified by the agent via agt commands)
    let (paths, mut doc) = load_project()?;

    if exit_status.success() && !is_error {
        eprintln!("Agent completed successfully for {}", todo_ref);

        // Add a comment summarizing the run
        let summary = if result_text.is_empty() {
            "Agent run completed.".to_string()
        } else {
            let truncated = if result_text.len() > 500 {
                format!("{}...", &result_text[..500])
            } else {
                result_text
            };
            format!("Agent run completed: {}", truncated)
        };
        operations::add_comment(&mut doc, num, &summary, None)?;
        save_project(&paths, &mut doc)?;
    } else {
        let code = exit_status.code().unwrap_or(-1);
        eprintln!("Agent failed for {} (exit code: {})", todo_ref, code);

        let error_msg = if result_text.is_empty() {
            format!("Agent run failed (exit code: {})", code)
        } else {
            let truncated = if result_text.len() > 500 {
                format!("{}...", &result_text[..500])
            } else {
                result_text
            };
            format!("Agent run failed (exit {}): {}", code, truncated)
        };
        operations::add_comment(&mut doc, num, &error_msg, None)?;
        save_project(&paths, &mut doc)?;
    }

    Ok(())
}

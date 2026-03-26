//! `agt plan` — Manage plan/research files for todos.
//!
//! Plans live as markdown files in `.todo/plans/<REF>.md`.
//! The automerge doc stores a `planPath` pointer on the todo.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::io::{BufRead, BufReader, IsTerminal};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use agt_lib::operations::{self, UpdateTodoFields};
use agt_lib::queries;
use agt_lib::storage;

use super::{load_project, parse_ref, save_project};

/// Resolve the plan file path for a todo. Returns (relative_path, absolute_path).
fn plan_paths(todo_dir: &std::path::Path, prefix: &str, num: u64) -> (String, PathBuf) {
    let relative = format!("plans/{}-{}.md", prefix, num);
    let absolute = todo_dir.join(&relative);
    (relative, absolute)
}

/// `agt plan show <ref>`
pub fn show(reference: String, answer: bool) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let plan_file = if let Some(plan_path) = &todo.plan_path {
        paths.todo_dir.join(plan_path)
    } else {
        // Check if file exists even without pointer
        let (_, abs) = plan_paths(&paths.todo_dir, &prefix, num);
        abs
    };

    if !plan_file.exists() {
        println!("{}", format!("No plan file for {}", todo_ref).dimmed());
        return Ok(());
    }

    let content = fs::read_to_string(&plan_file)
        .with_context(|| format!("Failed to read {}", plan_file.display()))?;

    // If stdout is a terminal, render markdown and show in a pager
    if std::io::stdout().is_terminal() {
        // Get terminal width for word wrapping
        let term_width = terminal_size::terminal_size()
            .map(|(w, _)| w.0 as u16)
            .unwrap_or(80);

        // Render markdown with termimad (colors, wrapping, formatting)
        let mut skin = termimad::MadSkin::default();

        // Code blocks: bold cyan text, no background highlight
        use termimad::crossterm::style::{Attribute, Color};
        let orange = Color::Rgb { r: 255, g: 170, b: 60 };
        skin.code_block.compound_style.set_fg(orange);
        skin.code_block.compound_style.add_attr(Attribute::Bold);
        skin.code_block.compound_style.set_bg(Color::Reset);
        skin.code_block.left_margin = 2;

        // Inline code: bold orange, no background
        skin.inline_code.set_fg(orange);
        skin.inline_code.add_attr(Attribute::Bold);
        skin.inline_code.set_bg(Color::Reset);

        let text = termimad::FmtText::from(&skin, &content, Some(term_width as usize));
        let rendered_str = format!("{}", text);

        let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
        let mut child = Command::new(&pager)
            .arg("-R") // interpret ANSI color codes
            .stdin(Stdio::piped())
            .spawn();

        match &mut child {
            Ok(proc) => {
                use std::io::Write;
                if let Some(stdin) = proc.stdin.as_mut() {
                    let _ = stdin.write_all(rendered_str.as_bytes());
                }
                drop(proc.stdin.take());
                let _ = proc.wait();
            }
            Err(_) => {
                // Pager not available, fall back to printing
                println!("{}", rendered_str);
            }
        }
        // After pager closes, if -a flag and there's a Questions section, prompt for an answer
        if answer {
        if let Some(questions_start) = content.find("\n## Questions") {
            let questions_section = &content[questions_start..];
            // Find where the next section starts (or end of file)
            let section_end = questions_section[1..]
                .find("\n## ")
                .map(|i| i + 1)
                .unwrap_or(questions_section.len());
            let questions_text = &questions_section[..section_end].trim();

            println!();
            println!("{}", colored::Colorize::bold("Questions from the plan:"));
            // Print just the bullet points, skip the heading
            for line in questions_text.lines().skip(1) {
                if !line.trim().is_empty() {
                    println!("  {}", line.trim());
                }
            }
            println!();
            eprint!(
                "{}",
                colored::Colorize::dimmed("Add an answer (enter to skip): ")
            );

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let input = input.trim();
                if !input.is_empty() {
                    append_answer_to_file(&plan_file, input)?;
                    println!(
                        "{}",
                        "Answer added to plan.".green()
                    );
                }
            }
        }
        }
    } else {
        // Piped output — just print raw markdown
        println!("{}", content);
    }
    Ok(())
}

/// Append an answer directly to a plan file's Answers section.
fn append_answer_to_file(plan_file: &std::path::Path, text: &str) -> Result<()> {
    let mut content = fs::read_to_string(plan_file)?;

    if content.contains("\n## Answers\n") || content.contains("\n## Answers\r\n") {
        content.push_str(&format!("\n> {}\n", text));
    } else {
        content.push_str(&format!("\n## Answers\n\n> {}\n", text));
    }

    fs::write(plan_file, &content)?;
    storage::git_stage(plan_file);
    Ok(())
}

/// `agt plan init <ref>` — Create the plan file and set planPath on the todo.
pub fn init(reference: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let (relative, absolute) = plan_paths(&paths.todo_dir, &prefix, num);

    // Create plans directory
    let plans_dir = paths.todo_dir.join("plans");
    fs::create_dir_all(&plans_dir)?;

    if absolute.exists() {
        // File already exists, just ensure pointer is set
        if todo.plan_path.is_none() {
            operations::update_todo(
                &mut doc,
                num,
                UpdateTodoFields {
                    plan_path: Some(Some(&relative)),
                    ..Default::default()
                },
                None,
            )?;
            save_project(&paths, &mut doc)?;
            println!("Linked existing plan file to {}", todo_ref);
        } else {
            println!("Plan already exists for {}", todo_ref);
        }
        println!("  {}", absolute.display());
        return Ok(());
    }

    // Create the file with a header
    let content = format!("# {}: {}\n\n", todo_ref, todo.title);
    fs::write(&absolute, &content)?;
    storage::git_stage(&absolute);

    // Set planPath on the todo
    operations::update_todo(
        &mut doc,
        num,
        UpdateTodoFields {
            plan_path: Some(Some(&relative)),
            ..Default::default()
        },
        None,
    )?;
    save_project(&paths, &mut doc)?;

    println!("Created plan for {}", todo_ref);
    println!("  {}", absolute.display());
    Ok(())
}

/// `agt plan answer <ref> <text>` — Append answers to the plan file.
pub fn answer(reference: String, text: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    // Find plan file
    let plan_file = if let Some(plan_path) = &todo.plan_path {
        paths.todo_dir.join(plan_path)
    } else {
        let (_, abs) = plan_paths(&paths.todo_dir, &prefix, num);
        abs
    };

    if !plan_file.exists() {
        bail!(
            "No plan file for {}. Run `agt plan init {}` first.",
            todo_ref,
            todo_ref
        );
    }

    let mut content = fs::read_to_string(&plan_file)?;

    // Find or create the Answers section
    if content.contains("\n## Answers\n") || content.contains("\n## Answers\r\n") {
        // Append to existing answers section
        content.push_str(&format!("\n> {}\n", text));
    } else {
        // Add new answers section
        content.push_str(&format!("\n## Answers\n\n> {}\n", text));
    }

    fs::write(&plan_file, &content)?;
    storage::git_stage(&plan_file);

    // Ensure planPath is set if it wasn't
    if todo.plan_path.is_none() {
        let (relative, _) = plan_paths(&paths.todo_dir, &prefix, num);
        operations::update_todo(
            &mut doc,
            num,
            UpdateTodoFields {
                plan_path: Some(Some(&relative)),
                ..Default::default()
            },
            None,
        )?;
        save_project(&paths, &mut doc)?;
    }

    println!("Answer added to {} plan", todo_ref);
    Ok(())
}

/// `agt plan path <ref>` — Print the plan file path (for scripting/agents).
pub fn path(reference: String) -> Result<()> {
    let (paths, doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {}-{} not found", prefix, num))?;

    if let Some(plan_path) = &todo.plan_path {
        let absolute = paths.todo_dir.join(plan_path);
        println!("{}", absolute.display());
    } else {
        let (_, absolute) = plan_paths(&paths.todo_dir, &prefix, num);
        println!("{}", absolute.display());
    }
    Ok(())
}

/// `agt plan trash <ref>` — Move plan file to macOS Trash and clear planPath.
pub fn trash(reference: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let plan_file = if let Some(plan_path) = &todo.plan_path {
        paths.todo_dir.join(plan_path)
    } else {
        let (_, abs) = plan_paths(&paths.todo_dir, &prefix, num);
        abs
    };

    if !plan_file.exists() {
        bail!("No plan file for {}", todo_ref);
    }

    // Move to macOS Trash via /usr/bin/trash
    let status = Command::new("/usr/bin/trash")
        .arg(&plan_file)
        .status()
        .context("Failed to run /usr/bin/trash")?;

    if !status.success() {
        bail!("trash command failed with exit code {}", status.code().unwrap_or(-1));
    }

    // Unstage from git if tracked
    let _ = Command::new("git")
        .arg("rm")
        .arg("--cached")
        .arg("--quiet")
        .arg(&plan_file)
        .current_dir(&paths.todo_dir)
        .status();

    // Clear planPath on the todo
    if todo.plan_path.is_some() {
        operations::update_todo(
            &mut doc,
            num,
            UpdateTodoFields {
                plan_path: Some(None),
                ..Default::default()
            },
            None,
        )?;
        save_project(&paths, &mut doc)?;
    }

    println!("Trashed plan for {}", todo_ref);
    Ok(())
}

/// `agt plan research <ref>` — Spawn an agent to research and flesh out the plan.
pub fn research(reference: String, dry_run: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, project_name, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;
    let todo_ref = format!("{}-{}", prefix, num);

    let todo = queries::find_todo_by_number(&doc, num)
        .ok_or_else(|| anyhow::anyhow!("Todo {} not found", todo_ref))?;

    let (relative, absolute) = plan_paths(&paths.todo_dir, &prefix, num);

    // Ensure plans dir exists
    let plans_dir = paths.todo_dir.join("plans");
    fs::create_dir_all(&plans_dir)?;

    // If plan file already exists, warn and prompt for confirmation
    if absolute.exists() && !dry_run {
        let existing = fs::read_to_string(&absolute)?;
        let line_count = existing.trim().lines().count();
        if line_count > 1 {
            if std::io::stdin().is_terminal() {
                eprintln!(
                    "{} Plan file already exists ({} lines): {}",
                    "⚠".yellow().bold(),
                    line_count,
                    absolute.display()
                );
                eprint!("Overwrite with new research? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            } else {
                bail!(
                    "Plan file already exists for {} ({} lines). Use --dry-run to preview, or delete the file first.",
                    todo_ref,
                    line_count
                );
            }
        }
    }

    if !absolute.exists() {
        let content = format!("# {}: {}\n\n", todo_ref, todo.title);
        fs::write(&absolute, &content)?;
        storage::git_stage(&absolute);
    }

    // Set planPath if not already set
    if todo.plan_path.is_none() {
        operations::update_todo(
            &mut doc,
            num,
            UpdateTodoFields {
                plan_path: Some(Some(&relative)),
                ..Default::default()
            },
            None,
        )?;
        save_project(&paths, &mut doc)?;
    }

    // Build prompt (matches server's handle_research_plan)
    let description_section = if todo.description.is_empty() {
        String::new()
    } else {
        format!("\n## Description\n{}\n", todo.description)
    };

    let comments_section = if todo.comments.is_empty() {
        String::new()
    } else {
        let comments: Vec<String> = todo
            .comments
            .iter()
            .map(|c| format!("**{}**: {}", c.author_name, c.text))
            .collect();
        format!("\n## Comments\n{}\n", comments.join("\n"))
    };

    // Read existing plan content to include as context
    let existing_content = fs::read_to_string(&absolute).unwrap_or_default();
    let existing_section = if existing_content.trim().lines().count() > 1 {
        format!(
            "\n## Existing Plan Content\n```markdown\n{}\n```\n\nBuild on this existing content. Keep what's useful, replace what needs updating.\n",
            existing_content.trim()
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are researching a task for the project "{project_name}".

## Task: {todo_ref} — {title}
Priority: {priority}
Difficulty: {difficulty}
{description_section}{comments_section}{existing_section}
Research this task by reading relevant code and files in the project.

Output the complete plan as markdown, starting with `# {todo_ref}: {title}`.

Structure it with these sections:
- **## Research** — What you found, relevant context, prior art
- **## Approach Options** — Numbered options with pros/cons
- **## Recommendation** — Your suggested approach
- **## Questions** — Bullet-pointed questions for the user (if any)
- **## Answers** — Include this empty section if you have questions, so the user can fill it in

Output ONLY the markdown plan content. Do not use any write tools."#,
        project_name = project_name,
        todo_ref = todo_ref,
        title = todo.title,
        priority = todo.priority.as_str(),
        difficulty = todo.difficulty.as_str(),
        description_section = description_section,
        comments_section = comments_section,
        existing_section = existing_section,
    );

    if dry_run {
        println!("{}", "Prompt:".bold());
        println!("{}", prompt);
        return Ok(());
    }

    eprintln!(
        "{} Researching {} — {}",
        "▶".green().bold(),
        todo_ref.bold(),
        todo.title
    );
    eprintln!("  Plan: {}", absolute.display().to_string().dimmed());
    eprintln!();

    let mut cmd = Command::new("claude");
    cmd.arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--allowedTools")
        .arg("Read Glob Grep")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .current_dir(&paths.todo_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("Failed to spawn `claude`. Is Claude Code installed?")?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Drain stderr in a background thread so it doesn't block
    let stderr_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut buf = String::new();
        for line in reader.lines() {
            if let Ok(l) = line {
                buf.push_str(&l);
                buf.push('\n');
            }
        }
        buf
    });

    let reader = BufReader::new(stdout);
    let mut result_text = String::new();
    let mut is_error = false;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
            let event_type = event
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match event_type {
                "assistant" => {
                    if let Some(content) = event
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            let block_type =
                                block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if block_type == "text" {
                                if let Some(text) =
                                    block.get("text").and_then(|v| v.as_str())
                                {
                                    eprintln!("  {}", text.dimmed());
                                }
                            } else if block_type == "tool_use" {
                                let tool_name = block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                eprintln!("  {} {}", "→".dimmed(), tool_name.dimmed());
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
                _ => {}
            }
        }
    }

    let status = child.wait()?;
    let stderr_output = stderr_handle.join().unwrap_or_default();

    if status.success() && !is_error {
        if result_text.is_empty() {
            bail!("Research agent returned empty result");
        }
        // Write the agent's output to the plan file
        fs::write(&absolute, &result_text)
            .with_context(|| format!("Failed to write plan to {}", absolute.display()))?;
        storage::git_stage(&absolute);
        eprintln!();
        eprintln!("{} Plan written to {}", "✓".green().bold(), absolute.display());
        println!("{}", result_text);
        Ok(())
    } else {
        let detail = if !result_text.is_empty() {
            result_text
        } else if !stderr_output.is_empty() {
            stderr_output.trim().to_string()
        } else {
            format!("exit code {}", status.code().unwrap_or(-1))
        };
        bail!("Research agent failed: {}", detail);
    }
}

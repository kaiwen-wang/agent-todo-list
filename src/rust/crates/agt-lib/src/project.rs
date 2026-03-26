//! .todo/ directory management — finding, creating, and reading project config.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::schema::ProjectConfig;

const TODO_DIR: &str = ".todo";
const CONFIG_FILE: &str = "config.toml";
const DATA_FILE: &str = "data.automerge";
const INBOX_FILE: &str = "TODO.md";
const PROCESSED_FILE: &str = "TODO-PROCESSED.md";

#[derive(Debug, Clone)]
pub struct TodoPaths {
    /// Root of the git repo (or wherever .todo/ lives)
    pub root: PathBuf,
    /// Path to .todo/ directory
    pub todo_dir: PathBuf,
    /// Path to .todo/config.toml
    pub config_path: PathBuf,
    /// Path to .todo/data.automerge
    pub data_path: PathBuf,
    /// Path to .todo/TODO.md (inbox)
    pub inbox_path: PathBuf,
    /// Path to .todo/TODO-PROCESSED.md (processed inbox archive)
    pub processed_path: PathBuf,
}

/// Walk up from `start_dir` looking for a .todo/ directory.
pub fn find_project(start_dir: &Path) -> Option<TodoPaths> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(TODO_DIR);
        if candidate.is_dir() {
            return Some(TodoPaths {
                root: dir.clone(),
                config_path: candidate.join(CONFIG_FILE),
                data_path: candidate.join(DATA_FILE),
                inbox_path: candidate.join(INBOX_FILE),
                processed_path: candidate.join(PROCESSED_FILE),
                todo_dir: candidate,
            });
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Check if the given directory is a git repo.
pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

/// Create the .todo/ directory and config file for a new project.
pub fn init_project(dir: &Path, config: &ProjectConfig) -> Result<TodoPaths> {
    let todo_dir = dir.join(TODO_DIR);
    std::fs::create_dir_all(&todo_dir)?;

    let paths = TodoPaths {
        root: dir.to_path_buf(),
        config_path: todo_dir.join(CONFIG_FILE),
        data_path: todo_dir.join(DATA_FILE),
        inbox_path: todo_dir.join(INBOX_FILE),
        processed_path: todo_dir.join(PROCESSED_FILE),
        todo_dir,
    };

    // Write config.toml
    let toml = format!(
        "id = \"{}\"\nprefix = \"{}\"\nname = \"{}\"\n",
        config.id, config.prefix, config.name
    );
    std::fs::write(&paths.config_path, &toml)?;

    // Set up .gitattributes for the Automerge merge driver
    let gitattrs_path = dir.join(".gitattributes");
    let merge_driver_line = ".todo/data.automerge merge=automerge-crdt";

    if gitattrs_path.exists() {
        let content = std::fs::read_to_string(&gitattrs_path)?;
        if !content.contains("merge=automerge-crdt") {
            let new_content = format!(
                "{}\n\n# CRDT merge driver for Automerge binary data\n{}\n",
                content.trim_end(),
                merge_driver_line
            );
            std::fs::write(&gitattrs_path, new_content)?;
        }
    } else {
        std::fs::write(
            &gitattrs_path,
            format!("# CRDT merge driver for Automerge binary data\n{merge_driver_line}\n"),
        )?;
    }

    // Create inbox files
    if !paths.inbox_path.exists() {
        std::fs::write(&paths.inbox_path, "")?;
    }
    if !paths.processed_path.exists() {
        std::fs::write(&paths.processed_path, "")?;
    }

    // Configure the git merge driver
    if is_git_repo(dir) {
        // The merge driver will call our own binary
        let _ = Command::new("git")
            .args([
                "config",
                "merge.automerge-crdt.name",
                "Automerge CRDT merge driver",
            ])
            .current_dir(dir)
            .output();
        let _ = Command::new("git")
            .args([
                "config",
                "merge.automerge-crdt.driver",
                "agt merge-driver %O %A %B",
            ])
            .current_dir(dir)
            .output();
    }

    Ok(paths)
}

/// Write config.toml to keep it in sync with the Automerge doc.
pub fn sync_config(config_path: &Path, prefix: &str, name: &str) -> Result<()> {
    let existing = read_config(config_path)?;
    let toml = format!(
        "id = \"{}\"\nprefix = \"{}\"\nname = \"{}\"\n",
        existing.id, prefix, name
    );
    std::fs::write(config_path, toml)?;
    Ok(())
}

/// Read the config.toml file.
pub fn read_config(config_path: &Path) -> Result<ProjectConfig> {
    let text = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    let mut id = None;
    let mut prefix = None;
    let mut name = None;

    let re = regex::Regex::new(r#"^(\w+)\s*=\s*"([^"]*)""#).unwrap();
    for line in text.lines() {
        if let Some(caps) = re.captures(line) {
            match &caps[1] {
                "id" => id = Some(caps[2].to_string()),
                "prefix" => prefix = Some(caps[2].to_string()),
                "name" => name = Some(caps[2].to_string()),
                _ => {}
            }
        }
    }

    match (id, prefix, name) {
        (Some(id), Some(prefix), Some(name)) => Ok(ProjectConfig { id, prefix, name }),
        _ => bail!(
            "invalid config at {}: missing required fields",
            config_path.display()
        ),
    }
}

/// Auto-detect a project name from the environment.
pub fn detect_project_name(dir: &Path) -> String {
    // 1. package.json name
    let pkg_path = dir.join("package.json");
    if pkg_path.exists()
        && let Ok(content) = std::fs::read_to_string(&pkg_path)
        && let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(name) = pkg.get("name").and_then(|n| n.as_str())
    {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return prettify_name(trimmed);
        }
    }

    // 2. git remote (origin) repo name
    if let Ok(output) = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(dir)
        .output()
        && output.status.success()
    {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Some(repo_name) = Path::new(&url)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.trim_end_matches(".git"))
            && !repo_name.is_empty()
        {
            return prettify_name(repo_name);
        }
    }

    // 3. Directory name
    dir.file_name()
        .and_then(|n| n.to_str())
        .map(prettify_name)
        .unwrap_or_else(|| "Project".to_string())
}

/// Turn a slug like "agent-todo-list" into "Agent Todo List".
fn prettify_name(slug: &str) -> String {
    // Strip npm scope
    let slug = if slug.starts_with('@') {
        slug.split('/').nth(1).unwrap_or(slug)
    } else {
        slug
    };

    slug.split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Derive a short prefix from a project name.
pub fn derive_prefix(name: &str) -> String {
    let slug = if name.starts_with('@') {
        name.split('/').nth(1).unwrap_or(name)
    } else {
        name
    };

    let words: Vec<&str> = slug
        .split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .collect();

    if words.len() == 1 {
        words[0].chars().take(3).collect::<String>().to_uppercase()
    } else {
        words
            .iter()
            .take(4)
            .filter_map(|w| w.chars().next())
            .collect::<String>()
            .to_uppercase()
    }
}

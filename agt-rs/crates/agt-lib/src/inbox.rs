//! Inbox file utilities — read/write .todo/TODO.md and .todo/TODO-PROCESSED.md.

use anyhow::Result;
use std::path::Path;

pub struct ProcessedEntry {
    /// Original text from the inbox
    pub original: String,
    /// The created task reference (e.g. "ISD-5")
    pub reference: String,
    /// The created task title
    pub title: String,
}

/// Read the inbox (TODO.md). Returns empty string if file doesn't exist.
pub fn read_inbox(todo_dir: &Path) -> Result<String> {
    let path = todo_dir.join("TODO.md");
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(path)?)
}

/// Overwrite the inbox (TODO.md) with new content.
pub fn write_inbox(todo_dir: &Path, text: &str) -> Result<()> {
    let path = todo_dir.join("TODO.md");
    Ok(std::fs::write(path, text)?)
}

/// Read the processed archive (TODO-PROCESSED.md).
pub fn read_processed(todo_dir: &Path) -> Result<String> {
    let path = todo_dir.join("TODO-PROCESSED.md");
    if !path.exists() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(path)?)
}

/// Append processed entries to TODO-PROCESSED.md with a timestamp header.
pub fn append_processed(todo_dir: &Path, entries: &[ProcessedEntry]) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let existing = read_processed(todo_dir)?;
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let lines: Vec<String> = entries
        .iter()
        .map(|e| format!("- [{}] {} -> \"{}\"", e.reference, e.original, e.title))
        .collect();

    let block = format!("\n## {}\n\n{}\n", timestamp, lines.join("\n"));
    let content = if existing.is_empty() {
        block.trim_start().to_string()
    } else {
        format!("{}\n{}", existing.trim_end(), block)
    };

    let path = todo_dir.join("TODO-PROCESSED.md");
    Ok(std::fs::write(path, content)?)
}

/// Create empty inbox files if they don't exist yet.
pub fn ensure_inbox_files(todo_dir: &Path) -> Result<()> {
    let inbox = todo_dir.join("TODO.md");
    if !inbox.exists() {
        std::fs::write(&inbox, "")?;
    }
    let processed = todo_dir.join("TODO-PROCESSED.md");
    if !processed.exists() {
        std::fs::write(&processed, "")?;
    }
    Ok(())
}

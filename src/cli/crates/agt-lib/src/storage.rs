//! Load and save Automerge documents to disk.

use anyhow::{Context, Result};
use automerge::AutoCommit;
use std::path::Path;

/// Save an Automerge document to a file, then git-add it.
pub fn save_doc(path: &Path, doc: &mut AutoCommit) -> Result<()> {
    let binary = doc.save();
    std::fs::write(path, binary)
        .with_context(|| format!("failed to write {}", path.display()))?;

    // Auto-stage so the file is included in the next commit
    let _ = std::process::Command::new("git")
        .args(["add", &path.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    Ok(())
}

/// Load an Automerge document from a file. Returns None if file doesn't exist.
pub fn load_doc(path: &Path) -> Result<Option<AutoCommit>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let doc = AutoCommit::load(&bytes)
        .with_context(|| format!("failed to load automerge doc from {}", path.display()))?;
    Ok(Some(doc))
}

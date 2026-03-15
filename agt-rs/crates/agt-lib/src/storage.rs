//! Load and save Automerge documents to disk.

use anyhow::{Context, Result};
use automerge::AutoCommit;
use std::path::Path;

/// Save an Automerge document to a file.
pub fn save_doc(path: &Path, doc: &mut AutoCommit) -> Result<()> {
    let binary = doc.save();
    std::fs::write(path, binary)
        .with_context(|| format!("failed to write {}", path.display()))?;
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

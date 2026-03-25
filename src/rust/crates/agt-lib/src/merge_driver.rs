//! Git merge driver for Automerge .automerge files.
//!
//! When git encounters a merge conflict on a .automerge file, it calls
//! this instead of its default binary conflict handler.

use anyhow::{Context, Result};
use automerge::AutoCommit;
use std::path::Path;

/// Perform a three-way merge of Automerge documents.
/// Writes the merged result to `ours_path` (git convention).
pub fn merge_files(
    _base_path: &Path,
    ours_path: &Path,
    theirs_path: &Path,
) -> Result<()> {
    let ours_bytes = std::fs::read(ours_path)
        .with_context(|| format!("failed to read {}", ours_path.display()))?;
    let theirs_bytes = std::fs::read(theirs_path)
        .with_context(|| format!("failed to read {}", theirs_path.display()))?;

    let mut ours = AutoCommit::load(&ours_bytes)
        .with_context(|| "failed to load ours")?;
    let mut theirs = AutoCommit::load(&theirs_bytes)
        .with_context(|| "failed to load theirs")?;

    // Automerge merge is conflict-free — concurrent changes combine cleanly
    ours.merge(&mut theirs)
        .with_context(|| "automerge merge failed")?;

    let binary = ours.save();
    std::fs::write(ours_path, binary)
        .with_context(|| format!("failed to write merged result to {}", ours_path.display()))?;

    Ok(())
}

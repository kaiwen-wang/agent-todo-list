pub mod init;
pub mod add;
pub mod list;
pub mod show;
pub mod update;
pub mod delete;
pub mod assign;
pub mod unassign;
pub mod comment;
pub mod branch;
pub mod member;
pub mod config;
pub mod serve;
pub mod inbox;
pub mod log;
pub mod merge_driver;

use agt_lib::project::{TodoPaths, find_project};
use agt_lib::storage;
use agt_lib::migrate;
use anyhow::{Context, Result};
use std::env;

/// Load the project: find .todo/, load the automerge doc, run migrations if needed.
fn load_project() -> Result<(TodoPaths, automerge::AutoCommit)> {
    let cwd = env::current_dir()?;
    let paths = find_project(&cwd)
        .context("No .todo/ directory found. Run `agt init` first.")?;
    let mut doc = storage::load_doc(&paths.data_path)?
        .context("Failed to load data.automerge")?;

    if migrate::needs_migration(&doc) {
        migrate::migrate_doc(&mut doc)?;
        storage::save_doc(&paths.data_path, &mut doc)?;
    }

    Ok((paths, doc))
}

/// Save the document back to disk.
fn save_project(paths: &TodoPaths, doc: &mut automerge::AutoCommit) -> Result<()> {
    storage::save_doc(&paths.data_path, doc)
}

/// Parse a todo reference, returning the number.
fn parse_ref(reference: &str, prefix: &str) -> Result<u64> {
    agt_lib::queries::parse_todo_ref(reference, prefix)
        .context(format!("Invalid todo reference: {reference}"))
}

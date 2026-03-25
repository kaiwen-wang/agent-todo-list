pub mod add;
pub mod assign;
pub mod branch;
pub mod comment;
pub mod commit;
pub mod config;
pub mod delete;
pub mod inbox;
pub mod init;
pub mod list;
pub mod log;
pub mod member;
pub mod merge_driver;
pub mod plan;
pub mod poll;
pub mod queue;
pub mod run;
pub mod runs;
pub mod serve;
pub mod show;
pub mod unassign;
pub mod unbranch;
pub mod update;
pub mod workflow;

use agt_lib::migrate;
use agt_lib::project::{find_project, TodoPaths};
use agt_lib::storage;
use anyhow::{Context, Result};
use std::env;

/// Load the project: find .todo/, load the automerge doc, run migrations if needed.
fn load_project() -> Result<(TodoPaths, automerge::AutoCommit)> {
    let cwd = env::current_dir()?;
    let paths = find_project(&cwd).context("No .todo/ directory found. Run `agt init` first.")?;
    let mut doc = storage::load_doc(&paths.data_path)?.context("Failed to load data.automerge")?;

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

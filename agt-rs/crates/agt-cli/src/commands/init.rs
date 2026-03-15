use anyhow::{Result, bail};
use std::env;
use uuid::Uuid;

use agt_lib::git_identity::get_git_identity;
use agt_lib::operations;
use agt_lib::project::{self, is_git_repo};
use agt_lib::schema::ProjectConfig;
use agt_lib::storage;

pub fn run(name: Option<String>, prefix: Option<String>) -> Result<()> {
    let cwd = env::current_dir()?;

    // Check if already initialized
    if project::find_project(&cwd).is_some() {
        bail!("Project already initialized. Found existing .todo/ directory.");
    }

    // Detect name and prefix
    let name = name.unwrap_or_else(|| project::detect_project_name(&cwd));
    let prefix = prefix.unwrap_or_else(|| project::derive_prefix(&name));

    // Get git identity for owner
    let git = get_git_identity();
    let owner_name = git.name.unwrap_or_else(|| "Owner".to_string());
    let owner_email = git.email;

    // Create config
    let config = ProjectConfig {
        id: Uuid::new_v4().to_string(),
        prefix: prefix.clone(),
        name: name.clone(),
    };

    // Initialize project directory
    let paths = project::init_project(&cwd, &config)?;

    // Create automerge document
    let mut doc = operations::create_project(
        &prefix,
        &name,
        &owner_name,
        owner_email.as_deref(),
    )?;

    // Save
    storage::save_doc(&paths.data_path, &mut doc)?;

    println!("Initialized project \"{}\" with prefix {}", name, prefix);
    if is_git_repo(&cwd) {
        println!("Git merge driver configured for .automerge files.");
    }

    Ok(())
}

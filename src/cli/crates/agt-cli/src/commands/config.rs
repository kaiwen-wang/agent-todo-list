use anyhow::Result;

use agt_lib::operations;
use agt_lib::project;
use agt_lib::queries;

use super::{load_project, save_project};

pub fn run(name: Option<String>, prefix: Option<String>) -> Result<()> {
    let has_updates = name.is_some() || prefix.is_some();

    if has_updates {
        let (paths, mut doc) = load_project()?;

        operations::update_project(
            &mut doc,
            name.as_deref(),
            None,
            prefix.as_deref(),
            None,
        )?;

        save_project(&paths, &mut doc)?;

        // Keep config.toml in sync with the Automerge doc
        let (_, new_prefix, new_name, _) = queries::read_project_meta(&doc);
        project::sync_config(&paths.config_path, &new_prefix, &new_name)?;

        println!("Updated project settings");
    } else {
        let (paths, doc) = load_project()?;
        let (id, prefix, name, description) = queries::read_project_meta(&doc);
        let members = queries::read_all_members(&doc);
        let todos = queries::read_all_todos(&doc);

        println!("Project: {}", name);
        println!("  ID:     {}", id);
        println!("  Prefix: {}", prefix);
        if !description.is_empty() {
            println!("  Desc:   {}", description);
        }
        println!("  Root:   {}", paths.root.display());
        println!("  Todos:  {}", todos.len());
        println!("  Members: {}", members.len());
    }

    Ok(())
}

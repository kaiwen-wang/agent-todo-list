use anyhow::Result;

use agt_lib::queries;

use super::load_project;

pub fn run() -> Result<()> {
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

    Ok(())
}

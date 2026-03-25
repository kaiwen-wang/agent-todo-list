use anyhow::Result;

use agt_lib::inbox;

use super::load_project;

pub fn run(action: Option<String>, text: Option<String>) -> Result<()> {
    let (paths, _doc) = load_project()?;

    match action.as_deref().unwrap_or("show") {
        "show" => {
            let content = inbox::read_inbox(&paths.todo_dir)?;
            if content.trim().is_empty() {
                println!("Inbox is empty.");
            } else {
                println!("{}", content);
            }
        }
        "append" => {
            let text = text.ok_or_else(|| anyhow::anyhow!("Text required for append"))?;
            let mut current = inbox::read_inbox(&paths.todo_dir)?;
            if !current.is_empty() && !current.ends_with('\n') {
                current.push('\n');
            }
            current.push_str(&text);
            current.push('\n');
            inbox::write_inbox(&paths.todo_dir, &current)?;
            println!("Appended to inbox.");
        }
        "clear" => {
            inbox::write_inbox(&paths.todo_dir, "")?;
            println!("Inbox cleared.");
        }
        other => {
            anyhow::bail!("Unknown inbox action: {other}. Use show, append, or clear.");
        }
    }

    Ok(())
}

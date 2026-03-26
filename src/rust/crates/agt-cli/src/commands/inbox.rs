use anyhow::Result;

use agt_lib::inbox;
use agt_lib::operations::{self, AddTodoOpts};
use agt_lib::queries;
use agt_lib::schema::Platform;

use super::{load_project, save_project};

pub fn run(action: Option<String>, text: Option<String>) -> Result<()> {
    let (paths, mut doc) = load_project()?;

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
        "process" => {
            let content = inbox::read_inbox(&paths.todo_dir)?;
            let items = inbox::parse_inbox_items(&content);

            if items.is_empty() {
                println!("Inbox is empty — nothing to process.");
                return Ok(());
            }

            let (_, prefix, _, _) = queries::read_project_meta(&doc);
            let mut entries = Vec::new();

            for item in &items {
                let number = operations::add_todo(
                    &mut doc,
                    AddTodoOpts {
                        title: item,
                        description: None,
                        status: None,
                        priority: None,
                        difficulty: None,
                        labels: None,
                        assignee: None,
                        created_by: None,
                        platform: Some(Platform::Cli),
                    },
                )?;

                let reference = format!("{}-{}", prefix, number);
                println!("Created {}: {}", reference, item);

                entries.push(inbox::ProcessedEntry {
                    original: item.clone(),
                    reference,
                    title: item.clone(),
                });
            }

            save_project(&paths, &mut doc)?;
            inbox::append_processed(&paths.todo_dir, &entries)?;
            inbox::write_inbox(&paths.todo_dir, "")?;

            println!(
                "\nProcessed {} item{} from inbox.",
                entries.len(),
                if entries.len() == 1 { "" } else { "s" }
            );
        }
        other => {
            anyhow::bail!("Unknown inbox action: {other}. Use show, append, clear, or process.");
        }
    }

    Ok(())
}

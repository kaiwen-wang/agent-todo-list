use anyhow::Result;

use agt_lib::operations;
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String, text: String, reply_to: Option<String>, json: bool) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    operations::add_comment(&mut doc, num, &text, None, reply_to.as_deref())?;
    save_project(&paths, &mut doc)?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "reference": format!("{}-{}", prefix, num),
            })
        );
    } else {
        let suffix = if reply_to.is_some() { " (reply)" } else { "" };
        println!("Added comment{suffix} to {prefix}-{num}");
    }
    Ok(())
}

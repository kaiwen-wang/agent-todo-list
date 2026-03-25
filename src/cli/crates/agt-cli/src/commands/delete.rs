use anyhow::Result;

use agt_lib::operations;
use agt_lib::queries;

use super::{load_project, parse_ref, save_project};

pub fn run(reference: String) -> Result<()> {
    let (paths, mut doc) = load_project()?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let num = parse_ref(&reference, &prefix)?;

    operations::delete_todo(&mut doc, num, None)?;
    save_project(&paths, &mut doc)?;

    println!("Deleted {}-{}", prefix, num);
    Ok(())
}

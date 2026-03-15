use anyhow::Result;
use std::path::Path;

pub fn run(base: String, ours: String, theirs: String) -> Result<()> {
    agt_lib::merge_driver::merge_files(
        Path::new(&base),
        Path::new(&ours),
        Path::new(&theirs),
    )?;
    Ok(())
}

use anyhow::Result;

use super::load_project;

pub fn run(port: u16) -> Result<()> {
    let (paths, _doc) = load_project()?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(agt_server::start_server(&paths.root, port))?;

    Ok(())
}

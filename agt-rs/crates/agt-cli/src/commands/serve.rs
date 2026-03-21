use anyhow::Result;

use super::load_project;

pub fn run(port: u16, open: bool) -> Result<()> {
    let (paths, _doc) = load_project()?;

    if open {
        // Open browser after a short delay to let the server start
        let url = format!("http://localhost:{}", port);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = std::process::Command::new("open").arg(&url).status();
        });
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(agt_server::start_server(&paths.root, port))?;

    Ok(())
}

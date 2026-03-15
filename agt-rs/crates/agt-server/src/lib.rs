//! agt web dashboard server.
//!
//! API endpoints:
//!   GET  /api/project — returns full project as JSON
//!   POST /api/change  — applies a single mutation
//!   WS   /ws          — push refresh notifications to browsers
//!
//! Serves the Vue dist/ as static files.

mod routes;
mod state;

use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use state::AppState;

/// Locate the web dist directory.
fn resolve_dist_dir() -> PathBuf {
    // Installed location
    let home = dirs_home();
    let installed = home.join(".local/share/agt/web");
    if installed.exists() {
        return installed;
    }
    // Dev fallback: check for src/web/dist relative to the crate
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../src/web/dist");
    if dev.exists() {
        return dev;
    }
    // Last resort: current directory
    PathBuf::from("dist")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

/// Start the server and block until shutdown.
pub async fn start_server(project_path: &Path, port: u16) -> Result<()> {
    let todo_dir = project_path.join(".todo");
    let data_path = todo_dir.join("data.automerge");
    let config_path = todo_dir.join("config.toml");
    let dist_dir = resolve_dist_dir();

    let state = AppState::new(data_path.clone(), config_path, todo_dir)?;

    // Start file watcher in background
    let watcher_state = state.clone();
    let watch_path = data_path.clone();
    tokio::spawn(async move {
        if let Err(e) = state::watch_file(watch_path, watcher_state).await {
            tracing::error!("File watcher error: {e}");
        }
    });

    // Build router
    let api = Router::new()
        .route("/api/project", axum::routing::get(routes::get_project))
        .route("/api/change", axum::routing::post(routes::post_change))
        .route("/ws", axum::routing::get(routes::ws_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Serve static files as fallback (Vue SPA)
    let app = api.fallback_service(
        ServeDir::new(&dist_dir).fallback(
            tower_http::services::ServeFile::new(dist_dir.join("index.html")),
        ),
    );

    // Try ports
    let max_attempts = 10;
    for attempt in 0..max_attempts {
        let addr = SocketAddr::from(([127, 0, 0, 1], port + attempt));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Dashboard: http://localhost:{}", port + attempt);
                axum::serve(listener, app).await?;
                return Ok(());
            }
            Err(e) if attempt < max_attempts - 1 => {
                tracing::debug!("Port {} busy: {e}, trying next", port + attempt);
                continue;
            }
            Err(e) => {
                return Err(e).context(format!(
                    "Could not bind to any port in range {}-{}",
                    port,
                    port + max_attempts - 1
                ));
            }
        }
    }

    unreachable!()
}

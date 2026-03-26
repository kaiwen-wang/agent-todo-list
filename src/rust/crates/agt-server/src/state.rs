//! Shared application state: Automerge doc, WebSocket broadcast, file watcher.

use anyhow::{Context, Result};
use automerge::AutoCommit;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use agt_lib::migrate;
use agt_lib::storage;

/// Shared state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub data_path: PathBuf,
    pub config_path: PathBuf,
    pub todo_dir: PathBuf,
    pub doc: Arc<Mutex<AutoCommit>>,
    /// Broadcast channel for WebSocket refresh notifications.
    pub tx: broadcast::Sender<String>,
    /// Flag to ignore our own file writes.
    pub saving: Arc<std::sync::atomic::AtomicBool>,
}

impl AppState {
    pub fn new(data_path: PathBuf, config_path: PathBuf, todo_dir: PathBuf) -> Result<Self> {
        let mut doc = storage::load_doc(&data_path)?.context("Failed to load data.automerge")?;

        if migrate::needs_migration(&doc) {
            migrate::migrate_doc(&mut doc)?;
            storage::save_doc(&data_path, &mut doc)?;
        }

        let (tx, _) = broadcast::channel(64);

        Ok(Self {
            data_path,
            config_path,
            todo_dir,
            doc: Arc::new(Mutex::new(doc)),
            tx,
            saving: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    /// Reload the document from disk.
    pub async fn reload(&self) -> Result<()> {
        let mut doc =
            storage::load_doc(&self.data_path)?.context("Failed to reload data.automerge")?;

        if migrate::needs_migration(&doc) {
            migrate::migrate_doc(&mut doc)?;
        }

        let mut guard = self.doc.lock().await;
        *guard = doc;
        Ok(())
    }

    /// Save the document to disk and broadcast refresh.
    pub async fn save(&self) -> Result<()> {
        self.saving.store(true, std::sync::atomic::Ordering::SeqCst);

        {
            let mut doc = self.doc.lock().await;
            storage::save_doc(&self.data_path, &mut doc)?;
        }

        // Broadcast refresh to all WebSocket clients
        let _ = self.tx.send(r#"{"type":"refresh"}"#.to_string());

        // Allow file watcher to resume after a small delay
        let saving = self.saving.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            saving.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        Ok(())
    }
}

/// Watch the data.automerge file for external changes (CLI writes).
pub async fn watch_file(path: PathBuf, state: AppState) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res
                && matches!(event.kind, EventKind::Modify(_))
            {
                let _ = tx.blocking_send(());
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    // Keep watcher alive by holding it in this task
    loop {
        if rx.recv().await.is_none() {
            break;
        }

        // Skip if we're the ones writing
        if state.saving.load(std::sync::atomic::Ordering::SeqCst) {
            continue;
        }

        // Reload and broadcast
        if let Err(e) = state.reload().await {
            tracing::error!("Failed to reload after file change: {e}");
        } else {
            let _ = state.tx.send(r#"{"type":"refresh"}"#.to_string());
        }
    }

    // prevent watcher from being dropped
    drop(watcher);
    Ok(())
}

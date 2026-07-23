//! Articuelate application entry point.
//!
//! This module only assembles the execution engine, UI application, and their
//! thread boundary. Floem-specific setup lives in `app.rs`.

mod app;
mod exec;
mod model;
mod theme;
mod ui;

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::app::App;
use crate::exec::ExecutionEngine;
use crate::model::WorkspaceState;

fn main() {
    // Temporarily use programmatic samples.
    // TODO: remove after save state is implemented.
    let workspace = Arc::new(ArcSwap::from_pointee(WorkspaceState::sample()));

    let (exec_engine, exec_state_rx, events_tx) = ExecutionEngine::init(workspace.clone());
    let (app, state_forwarder, theme_reload_tx) = App::init(workspace, exec_state_rx, events_tx);

    let (tokio_tx, tokio_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build shared tokio runtime");

        rt.block_on(async move {
            let handle = tokio::runtime::Handle::current();
            let _ = tokio_tx.send(handle);

            exec_engine.run().await;
        });
    });

    let tokio_handle = tokio_rx.recv().expect("Tokio runtime failed to start");
    tokio_handle.spawn(state_forwarder);

    // Wire up the theme file watcher on the tokio runtime so edits to
    // themes/*.toml trigger live reloads in the UI.
    let theme_watcher = crate::theme::watch_theme_async(theme_reload_tx);
    tokio_handle.spawn(theme_watcher);

    app.run();
}

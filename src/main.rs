//! Articuelate application entry point.
//!
//! This module only assembles the execution engine, UI application, and their
//! thread boundary. Floem-specific setup lives in `app.rs`.

mod app;
mod audio;
mod exec;
mod model;
mod style;
mod ui;

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::app::App;
use crate::audio::AudioEngine;
use crate::exec::ExecutionEngine;
use crate::model::WorkspaceState;

fn main() {
    // Temporarily use programmatic samples.
    // TODO: remove after save state is implemented.
    let workspace = Arc::new(ArcSwap::from_pointee(WorkspaceState::sample()));

    let (_audio_engine, dsp_tx, audio_events_rx, telemetry) = AudioEngine::init();
    let (exec_engine, exec_state_rx, events_tx) =
        ExecutionEngine::init(workspace.clone(), dsp_tx, audio_events_rx);
    let (app, exec_state_forward, theme_reload_tx) =
        App::init(workspace, exec_state_rx, events_tx, telemetry);

    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build shared tokio runtime");

    let tokio_handle = rt.handle().clone();

    std::thread::spawn(move || {
        rt.block_on(async {
            let _ = shutdown_rx.await;
        });
    });

    tokio_handle.spawn(exec_engine.run());
    tokio_handle.spawn(exec_state_forward);
    tokio_handle.spawn(crate::style::watch_theme_async(theme_reload_tx));

    app.run();
}

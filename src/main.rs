//! Articuelate application entry point.

mod app;
mod audio;
mod exec;
mod model;
mod style;
mod ui;

use std::sync::Arc;

use arc_swap::ArcSwap;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::app::App;
use crate::audio::AudioEngine;
use crate::exec::ExecutionEngine;
use crate::model::WorkspaceState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Articuelate starting");

    // ── Workspace (source of truth shared with the execution engine) ──
    let cuelist = crate::model::Cuelist::new(crate::model::sample_cues());
    let workspace = Arc::new(ArcSwap::from_pointee(WorkspaceState {
        cuelist: Arc::new(cuelist),
    }));

    // ── Audio engine ──
    let mut audio_engine = Arc::new(AudioEngine::new());

    // ── Execution engine ──
    let execution = ExecutionEngine::new(
        workspace.clone(),
        audio_engine.command_sender(),
        audio_engine.take_audio_events(),
    );

    let (app, exec_state_forward, theme_reload_tx) = App::new(
        workspace,
        execution.state_receiver(),
        execution.handle(),
        audio_engine,
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build shared tokio runtime");
    let tokio_handle = rt.handle().clone();

    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    std::thread::spawn(move || {
        rt.block_on(async {
            let _ = shutdown_rx.await;
        });
    });

    tokio_handle.spawn(execution.run());
    tokio_handle.spawn(exec_state_forward);
    tokio_handle.spawn(crate::style::watch_theme_async(theme_reload_tx));

    app.run();

    info!("Articuelate shutting down");
}

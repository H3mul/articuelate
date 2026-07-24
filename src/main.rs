//! Articuelate application entry point.

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
    let workspace = Arc::new(ArcSwap::from_pointee(WorkspaceState::sample()));

    let mut audio_engine = AudioEngine::new();
    let audio_events = audio_engine.take_audio_events();
    let telemetry = audio_engine.take_telemetry();
    let audio_engine = Arc::new(audio_engine);

    let mut execution = ExecutionEngine::new();
    execution.set_workspace_state(workspace.clone());
    execution.set_audio_engine(&audio_engine, audio_events);

    let (app, exec_state_forward, theme_reload_tx) = App::new(
        workspace,
        execution.state_receiver(),
        execution.handle(),
        telemetry,
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
}

//! The Execution Thread: a Tokio event loop (event bus) that receives UI
//! intents and publishes execution state back to the UI.
//!
//! Threading model (mirrors `docs/pdd.md` §3.2): the Execution Engine runs
//! entirely off the Floem UI thread on its own Tokio runtime, so async work
//! never blocks window interaction.
//!
//! Communication pipelines:
//!
//! ```text
//!   UI Thread                Execution Thread (Tokio)
//!   ─────────                ───────────────────────
//!   UiEvent  ──mpsc──▶       event loop  (wakes on recv)
//!   (Go, ...)                │
//!            ◀─watch──       ExecutionState (pub/sub broadcast)
//! ```
//!
//! The UI pushes discrete intents through the `mpsc` event bus; each one wakes
//! Tokio and is dispatched. State changes are published back through the
//! `watch` channel, which the UI ingests into a Floem `RwSignal`.

use std::sync::Arc;
use std::thread;

use tokio::sync::{mpsc, watch};

use crate::model::{ExecutionState, WorkspaceState};

/// Discrete intents pushed from the UI onto the event bus.
///
/// Only `Go` is wired for now; this enum is the single extension point for
/// future UI events (Pause, Panic, Scrub, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiEvent {
    /// The operator pressed GO.
    Go,
}

/// UI-side handles used to wire the Execution Thread.
pub struct ExecutorHandle {
    /// Send UI intents to the Execution Thread. Cheaply cloneable; use
    /// `try_send` from synchronous UI callbacks.
    pub events: mpsc::Sender<UiEvent>,
    /// Subscribe to Execution Thread state for UI ingestion.
    pub state: watch::Receiver<ExecutionState>,
}

/// Boot the Execution Thread, returning the UI-side handles.
///
/// Spawns a dedicated OS thread running its own single-threaded Tokio runtime,
/// so the async orchestrator is fully decoupled from the Floem UI thread.
pub fn spawn(workspace: Arc<WorkspaceState>) -> ExecutorHandle {
    let (events_tx, events_rx) = mpsc::channel::<UiEvent>(64);
    let (state_tx, state_rx) = watch::channel(ExecutionState::default());

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("failed to build execution runtime");
        rt.block_on(run(workspace, events_rx, state_tx));
    });

    ExecutorHandle {
        events: events_tx,
        state: state_rx,
    }
}

/// The Tokio event loop. Awaits UI events; each one wakes the runtime and is
/// dispatched to its handler. State changes are published to the UI via the
/// `watch` channel.
async fn run(
    workspace: Arc<WorkspaceState>,
    mut events: mpsc::Receiver<UiEvent>,
    state: watch::Sender<ExecutionState>,
) {
    let mut exec = ExecutionState::default();

    while let Some(event) = events.recv().await {
        match event {
            UiEvent::Go => {
                // Dynamic read of the workspace: advance the linear playhead to
                // the next cue. (Strict playhead skipping of With/After cues
                // and actor spawning arrive in a later phase.)
                let last = workspace.cues.len().saturating_sub(1);
                exec.playhead = (exec.playhead + 1).min(last);
                exec.running = true;

                // Broadcast the new state to the UI. `send` only notifies when
                // the value actually changes, so the UI is not woken spuriously.
                let _ = state.send(exec);
            }
        }
    }
}

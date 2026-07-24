//! The Execution Engine: a Tokio event loop (event bus) that receives UI
//! intents and publishes execution state back to the UI.
//!
//! Threading model (mirrors `docs/pdd.md` §3.2): the Execution Engine runs
//! entirely off the Floem UI thread, on a *shared* Tokio runtime owned by
//! `main`. It knows nothing about the UI thread — it only communicates through
//! the `mpsc` intent bus and the `watch` state channel.
//!
//! Communication pipelines:
//!
//! ```text
//!   UI Thread                Execution Engine (Tokio, shared domain)
//!   ─────────                ───────────────────────────────────────
//!   UiEvent  ──mpsc──▶       event loop  (wakes on recv)
//!   (Go, ...)                │
//!            ◀─watch──       ExecutionState (pub/sub broadcast)
//! ```
//!
//! The UI pushes discrete intents through the `mpsc` event bus; each one wakes
//! Tokio and is dispatched. State changes are published back through the
//! `watch` channel, which the UI ingests into a Floem `RwSignal`.

use arc_swap::ArcSwap;
use std::sync::Arc;

use tokio::sync::{mpsc, watch};

use crate::audio::{AudioEvent, DSPCommand};
use crate::model::{CueKind, ExecutionState, Playhead, WorkspaceState};

/// Discrete intents pushed from the UI onto the event bus.
///
/// Only `Go` is wired for now; this enum is the single extension point for
/// future UI events (Pause, Panic, Scrub, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiEvent {
    /// The operator pressed GO.
    Go,
}

pub struct ExecutionEngine {
    rx_events: mpsc::Receiver<UiEvent>,
    rx_audio: mpsc::Receiver<AudioEvent>,
    tx_dsp: mpsc::Sender<DSPCommand>,
    tx_state: watch::Sender<Arc<ExecutionState>>,
    workspace_state: Arc<ArcSwap<WorkspaceState>>,
    state: Arc<ExecutionState>,
}

impl ExecutionEngine {
    pub fn init(
        workspace_state: Arc<ArcSwap<WorkspaceState>>,
        tx_dsp: mpsc::Sender<DSPCommand>,
        rx_audio: mpsc::Receiver<AudioEvent>,
    ) -> (
        Self,
        watch::Receiver<Arc<ExecutionState>>,
        mpsc::Sender<UiEvent>,
    ) {
        let state = Arc::new(ExecutionState::default());
        let (events_tx, events_rx) = mpsc::channel::<UiEvent>(64);
        let (state_tx, state_rx) = watch::channel(state.clone());
        (
            Self {
                rx_events: events_rx,
                rx_audio: rx_audio,
                tx_dsp: tx_dsp,
                tx_state: state_tx,
                workspace_state,
                state,
            },
            state_rx,
            events_tx,
        )
    }

    /// Helper: Publish the engine-owned execution state through the `watch`
    /// channel.
    ///
    /// Hands the current `Arc` to subscribers via a cheap refcount bump — the
    /// owned `Arc` stays put (ground truth). `send` only notifies when the
    /// value changed, so the UI is not woken spuriously.
    pub fn commit_exec_state(&self) {
        let _ = self.tx_state.send(self.state.clone());
    }

    /// Lock-free read of the workspace: `load_full` returns an
    /// `Arc<WorkspaceState>` (O(1) clone of the shared pointer), and we only
    /// borrow through it — the workspace value is never copied. The engine
    /// always resolves the successor against the latest cue ordering the UI
    /// published via the `ArcSwap`.
    fn workspace_state(&self) -> Arc<WorkspaceState> {
        self.workspace_state.load_full()
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.rx_events.recv() => {
                    let Some(UiEvent::Go) = event else { break };
                    let cuelist = self.workspace_state().cuelist.clone();
                    let next = match self.state.playhead {
                        Playhead::Stopped => cuelist.iter().next().cloned(),
                        Playhead::Playing(active) => cuelist.iter_after(active).and_then(|mut it| it.next().cloned()),
                    };
                    if let Some(cue) = next {
                        let CueKind::Media { file_path, volume_db, looping, .. } = &cue.kind;
                        {
                            let _ = self.tx_dsp.send(DSPCommand::Play {
                                cue_id: cue.id,
                                file_path: file_path.clone(),
                                volume_db: *volume_db,
                                looping: *looping,
                            }).await;
                        }
                        Arc::make_mut(&mut self.state).playhead = Playhead::Playing(cue.id);
                        self.commit_exec_state();
                    }
                }
                event = self.rx_audio.recv() => {
                    let Some(AudioEvent::PlaybackFinished { cue_id }) = event else { break };
                    if self.state.playhead == Playhead::Playing(cue_id) {
                        Arc::make_mut(&mut self.state).playhead = Playhead::Stopped;
                        self.commit_exec_state();
                    }
                }
            }
        }
    }
}

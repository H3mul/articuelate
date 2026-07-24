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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiEvent {
    /// The operator pressed GO.
    Go,
    SetAudioDevice(String),
}

#[derive(Clone)]
pub struct ExecutionHandle {
    tx_events: mpsc::Sender<UiEvent>,
}

impl ExecutionHandle {
    pub fn send_user_intent(
        &self,
        event: UiEvent,
    ) -> Result<(), mpsc::error::TrySendError<UiEvent>> {
        self.tx_events.try_send(event)
    }
}

pub struct ExecutionEngine {
    rx_events: mpsc::Receiver<UiEvent>,
    rx_audio: mpsc::Receiver<AudioEvent>,
    tx_dsp: mpsc::Sender<DSPCommand>,
    tx_state: watch::Sender<Arc<ExecutionState>>,
    tx_events: mpsc::Sender<UiEvent>,
    workspace_state: Arc<ArcSwap<WorkspaceState>>,
    state: Arc<ExecutionState>,
}

impl ExecutionEngine {
    pub fn new(
        workspace_state: Arc<ArcSwap<WorkspaceState>>,
        tx_dsp: mpsc::Sender<DSPCommand>,
        rx_audio: mpsc::Receiver<AudioEvent>,
    ) -> Self {
        let (state_tx, _state_rx) = watch::channel(Arc::new(ExecutionState::default()));
        let (events_tx, events_rx) = mpsc::channel::<UiEvent>(64);
        Self {
            rx_events: events_rx,
            tx_events: events_tx,
            rx_audio,
            tx_dsp,
            tx_state: state_tx,
            workspace_state,
            state: Arc::new(ExecutionState::default()),
        }
    }

    pub fn state_receiver(&self) -> watch::Receiver<Arc<ExecutionState>> {
        self.tx_state.subscribe()
    }

    pub fn handle(&self) -> ExecutionHandle {
        ExecutionHandle {
            tx_events: self.tx_events.clone(),
        }
    }

    pub async fn run(self) {
        let Self {
            mut rx_events,
            mut rx_audio,
            tx_dsp,
            tx_state,
            tx_events: _,
            workspace_state,
            mut state,
        } = self;
        loop {
            tokio::select! {
                event = rx_events.recv() => {
                    let Some(event) = event else { break };
                    if let UiEvent::SetAudioDevice(device_name) = event {
                        let _ = tx_dsp.send(DSPCommand::SetAudioDevice { device_name }).await;
                        continue;
                    }
                    let cuelist = workspace_state.load_full().cuelist.clone();
                    let next = match state.playhead {
                        Playhead::Stopped => cuelist.iter().next().cloned(),
                        Playhead::Playing(active) => cuelist.iter_after(active).and_then(|mut it| it.next().cloned()),
                    };
                    if let Some(cue) = next {
                        let CueKind::Media { file_path, volume_db, looping, .. } = &cue.kind;
                        {
                            let _ = tx_dsp.send(DSPCommand::Play {
                                cue_id: cue.id,
                                file_path: file_path.clone(),
                                volume_db: *volume_db,
                                looping: *looping,
                            }).await;
                        }
                        Arc::make_mut(&mut state).playhead = Playhead::Playing(cue.id);
                        let _ = tx_state.send(state.clone());
                    }
                }
                event = rx_audio.recv() => {
                    let Some(event) = event else { break };
                    match event {
                        AudioEvent::PlaybackFinished { cue_id } => {
                            if state.playhead == Playhead::Playing(cue_id) {
                                Arc::make_mut(&mut state).playhead = Playhead::Stopped;
                                let _ = tx_state.send(state.clone());
                            }
                        }
                        AudioEvent::DeviceLost { device_name, .. } => {
                            // Keep the execution loop alive while the audio
                            // router tears down the failed stream. If the
                            // event identifies the lost device, retrying it is
                            // harmless and lets the router perform a clean
                            // context handoff; the UI can select another
                            // device through UiEvent::SetAudioDevice.
                            if let Some(device_name) = device_name {
                                let _ = tx_dsp.send(DSPCommand::SetAudioDevice { device_name }).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

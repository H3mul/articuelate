//! Autonomous audio subsystem boundary.
//!
//! The public surface is deliberately small: the execution engine sends high-level
//! commands, while the UI receives a lock-free telemetry consumer. Runtime, decoder
//! tasks, and the CPAL-owned thread remain implementation details here.

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use tokio::sync::{Notify, mpsc};

use crate::model::CueId;

pub const SAMPLE_BUFFER_SIZE: usize = 48_000;
pub const TELEMETRY_BUFFER_SIZE: usize = 256;

#[derive(Clone, Debug)]
pub enum DSPCommand {
    Play {
        cue_id: CueId,
        file_path: PathBuf,
        volume_db: f32,
        looping: bool,
    },
    Seek {
        cue_id: CueId,
        position_sec: f32,
    },
    Pause {
        cue_id: CueId,
    },
    Stop {
        cue_id: CueId,
    },
}

#[derive(Clone, Debug)]
pub enum AudioEvent {
    PlaybackFinished { cue_id: CueId },
}

#[derive(Clone, Debug)]
pub enum AudioTelemetry {
    PlaybackState {
        cue_id: CueId,
        current_time_sec: f32,
        total_duration_sec: f32,
        left_peak: f32,
        right_peak: f32,
    },
}

#[derive(Clone, Debug)]
pub enum DecoderControl {
    SeekTo(f32),
    Stop,
}

struct DecoderBuffers {
    samples: HeapCons<f32>,
    control: HeapProd<DecoderControl>,
}

struct CpalCommand {
    cue_id: CueId,
    buffers: DecoderBuffers,
}

/// Owner of every resource belonging to the audio domain.
pub struct AudioEngine {
    runtime: Option<tokio::runtime::Runtime>,
    cpal_thread: Option<JoinHandle<()>>,
    cpal_tx: Option<std::sync::mpsc::Sender<CpalCommand>>,
}

impl AudioEngine {
    /// Starts the private async runtime, router, event forwarder, and CPAL thread.
    pub fn init() -> (
        Self,
        mpsc::Sender<DSPCommand>,
        mpsc::Receiver<AudioEvent>,
        HeapCons<AudioTelemetry>,
    ) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build audio tokio runtime");

        let (command_tx, command_rx) = mpsc::channel(64);
        let (audio_event_tx, audio_event_rx) = mpsc::channel(64);
        let (event_prod, event_cons) = HeapRb::<AudioEvent>::new(256).split();
        let (telemetry_prod, telemetry_cons) =
            HeapRb::<AudioTelemetry>::new(TELEMETRY_BUFFER_SIZE).split();
        let notify = Arc::new(Notify::new());
        let (cpal_tx, cpal_rx) = std::sync::mpsc::channel();

        let cpal_thread = spawn_cpal_thread(cpal_rx, event_prod, telemetry_prod, notify.clone());
        runtime.spawn(command_router(command_rx, cpal_tx.clone()));
        runtime.spawn(event_forwarder(event_cons, notify, audio_event_tx));

        (
            Self {
                runtime: Some(runtime),
                cpal_thread: Some(cpal_thread),
                cpal_tx: Some(cpal_tx),
            },
            command_tx,
            audio_event_rx,
            telemetry_cons,
        )
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        // Dropping the runtime stops the router/forwarder before joining the
        // CPAL thread, so shutdown cannot wait on a task that owns its sender.
        self.runtime.take();
        self.cpal_tx.take();
        if let Some(thread) = self.cpal_thread.take() {
            let _ = thread.join();
        }
    }
}

async fn command_router(
    mut rx: mpsc::Receiver<DSPCommand>,
    cpal_tx: std::sync::mpsc::Sender<CpalCommand>,
) {
    while let Some(command) = rx.recv().await {
        match command {
            DSPCommand::Play {
                cue_id, file_path, ..
            } => {
                let (sample_prod, sample_cons) = HeapRb::<f32>::new(SAMPLE_BUFFER_SIZE).split();
                let (control_prod, control_cons) = HeapRb::<DecoderControl>::new(32).split();
                let _ = cpal_tx.send(CpalCommand {
                    cue_id,
                    buffers: DecoderBuffers {
                        samples: sample_cons,
                        control: control_prod,
                    },
                });
                tokio::task::spawn_blocking(move || {
                    decode_file(file_path, sample_prod, control_cons)
                });
            }
            DSPCommand::Stop { .. } => {
                // Seek/stop are consumed by the CPAL node in the full mixer.
                // Their typed boundary is established now so callers do not
                // need to know how decoder controls are transported.
            }
            _ => {}
        }
    }
}

fn decode_file(
    path: PathBuf,
    mut samples: HeapProd<f32>,
    mut controls: ringbuf::HeapCons<DecoderControl>,
) {
    let Ok(file) = std::fs::File::open(path) else {
        return;
    };
    let Ok(decoder) = rodio::Decoder::new(BufReader::new(file)) else {
        return;
    };
    for sample in decoder {
        while let Some(control) = controls.try_pop() {
            if matches!(control, DecoderControl::Stop) {
                return;
            }
        }
        if samples.try_push(sample as f32 / i16::MAX as f32).is_err() {
            std::thread::yield_now();
        }
    }
}

async fn event_forwarder(
    mut events: HeapCons<AudioEvent>,
    notify: Arc<Notify>,
    tx: mpsc::Sender<AudioEvent>,
) {
    loop {
        notify.notified().await;
        while let Some(event) = events.try_pop() {
            if tx.send(event).await.is_err() {
                return;
            }
        }
    }
}

fn spawn_cpal_thread(
    rx: std::sync::mpsc::Receiver<CpalCommand>,
    mut events: HeapProd<AudioEvent>,
    mut telemetry: HeapProd<AudioTelemetry>,
    notify: Arc<Notify>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("articuelate-cpal".into())
        .spawn(move || {
            // Touch the host on the owning thread. Stream construction belongs here;
            // failure to have an output device must not prevent the UI from starting.
            let _host = cpal::default_host();
            let mut active = Vec::new();
            loop {
                let command = match rx.recv_timeout(Duration::from_millis(16)) {
                    Ok(command) => command,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                };
                active.push((command.cue_id, command.buffers));
                for (cue_id, buffers) in &mut active {
                    let _ = buffers.samples.try_pop();
                    let _ = telemetry.try_push(AudioTelemetry::PlaybackState {
                        cue_id: *cue_id,
                        current_time_sec: 0.0,
                        total_duration_sec: 0.0,
                        left_peak: 0.0,
                        right_peak: 0.0,
                    });
                }
            }
            for (cue_id, _) in active {
                if events
                    .try_push(AudioEvent::PlaybackFinished { cue_id })
                    .is_ok()
                {
                    notify.notify_one();
                }
            }
        })
        .expect("failed to spawn CPAL thread")
}

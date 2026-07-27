//! Driver-independent audio orchestration.

mod cpal_driver;
mod driver;
mod rodio_driver;

use arc_swap::ArcSwap;
use cpal::traits::HostTrait;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tokio::sync::mpsc;

use crate::model::CueId;
use tracing::{debug, error, info};

pub use cpal_driver::CpalDriver;
pub use driver::{AudioDriver, AudioDriverError, DriverCapabilities};
pub use rodio_driver::RodioDriver;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DSPCommand {
    SetAudioDevice {
        device_name: String,
    },
    Play {
        cue_id: CueId,
        file_path: std::path::PathBuf,
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
#[allow(dead_code)]
pub enum AudioEvent {
    PlaybackFinished {
        cue_id: CueId,
    },
    DeviceLost {
        device_name: Option<String>,
        error_message: String,
    },
}

#[derive(Debug)]
pub struct AtomicCueMetrics {
    current_time_bits: AtomicU64,
    total_duration_bits: AtomicU64,
    left_peak_bits: AtomicU32,
    right_peak_bits: AtomicU32,
}

impl AtomicCueMetrics {
    pub fn new() -> Self {
        Self {
            current_time_bits: AtomicU64::new(0),
            total_duration_bits: AtomicU64::new(0),
            left_peak_bits: AtomicU32::new(0),
            right_peak_bits: AtomicU32::new(0),
        }
    }

    pub fn update(
        &self,
        current_time_sec: f64,
        total_duration_sec: f64,
        left_peak: f32,
        right_peak: f32,
    ) {
        self.current_time_bits
            .store(current_time_sec.to_bits(), Ordering::Relaxed);
        self.total_duration_bits
            .store(total_duration_sec.to_bits(), Ordering::Relaxed);
        self.left_peak_bits
            .store(left_peak.to_bits(), Ordering::Relaxed);
        self.right_peak_bits
            .store(right_peak.to_bits(), Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn current_time_sec(&self) -> f64 {
        f64::from_bits(self.current_time_bits.load(Ordering::Relaxed))
    }
    #[allow(dead_code)]
    pub fn total_duration_sec(&self) -> f64 {
        f64::from_bits(self.total_duration_bits.load(Ordering::Relaxed))
    }
    #[allow(dead_code)]
    pub fn left_peak(&self) -> f32 {
        f32::from_bits(self.left_peak_bits.load(Ordering::Relaxed))
    }
    #[allow(dead_code)]
    pub fn right_peak(&self) -> f32 {
        f32::from_bits(self.right_peak_bits.load(Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct AudioTelemetry {
    pub cues: ArcSwap<HashMap<CueId, Arc<AtomicCueMetrics>>>,
}

impl AudioTelemetry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            cues: ArcSwap::from_pointee(HashMap::new()),
        })
    }

    pub fn metrics_for(&self, cue_id: CueId) -> Arc<AtomicCueMetrics> {
        if let Some(metrics) = self.cues.load().get(&cue_id) {
            return metrics.clone();
        }
        let metrics = Arc::new(AtomicCueMetrics::new());
        self.cues.rcu(|current| {
            let mut next = current.as_ref().clone();
            next.entry(cue_id).or_insert_with(|| metrics.clone());
            Arc::new(next)
        });
        self.cues
            .load()
            .get(&cue_id)
            .expect("telemetry metric inserted")
            .clone()
    }

    #[allow(dead_code)]
    pub fn remove(&self, cue_id: CueId) {
        let mut next = self.cues.load_full().as_ref().clone();
        next.remove(&cue_id);
        self.cues.store(Arc::new(next));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DriverType {
    Rodio,
    CpalCustom,
}

pub struct AudioEngine {
    command_tx: Option<mpsc::Sender<DSPCommand>>,
    audio_event_rx: Option<mpsc::Receiver<AudioEvent>>,
    telemetry: Arc<AudioTelemetry>,
    runtime_thread: Option<std::thread::JoinHandle<()>>,
    device_names: Vec<String>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl AudioEngine {
    /// Constructs the MVP Rodio-backed engine.
    pub fn new() -> Self {
        Self::with_driver(DriverType::Rodio)
    }

    pub fn with_driver(driver_type: DriverType) -> Self {
        let telemetry = AudioTelemetry::new();
        let (command_tx, command_rx) = mpsc::channel(64);
        let (audio_event_tx, audio_event_rx) = mpsc::channel(64);
        let device_names: Vec<String> = cpal::default_host()
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| cpal::traits::DeviceTrait::name(&device).ok())
            .collect();

        info!(
            driver = ?driver_type,
            device_count = device_names.len(),
            "Audio engine initializing"
        );

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build audio runtime");

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let telemetry_for_driver = telemetry.clone();
        let runtime_thread = std::thread::Builder::new()
            .name("articuelate-audio-router".into())
            .spawn(move || {
                let driver: Box<dyn AudioDriver> = match driver_type {
                    DriverType::Rodio => Box::new(RodioDriver::new(
                        telemetry_for_driver.clone(),
                        audio_event_tx.clone(),
                    )),
                    DriverType::CpalCustom => {
                        Box::new(CpalDriver::new(telemetry_for_driver, audio_event_tx))
                    }
                };
                runtime.block_on(driver_router(command_rx, driver, &mut shutdown_rx))
            })
            .expect("failed to spawn audio runtime thread");

        Self {
            command_tx: Some(command_tx),
            audio_event_rx: Some(audio_event_rx),
            telemetry,
            runtime_thread: Some(runtime_thread),
            device_names,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn command_sender(&self) -> mpsc::Sender<DSPCommand> {
        self.command_tx
            .as_ref()
            .expect("audio command channel initialized")
            .clone()
    }

    #[allow(dead_code)]
    pub async fn send_command(
        &self,
        command: DSPCommand,
    ) -> Result<(), mpsc::error::SendError<DSPCommand>> {
        self.command_tx
            .as_ref()
            .expect("audio command channel initialized")
            .send(command)
            .await
    }

    pub fn take_audio_events(self: &mut Arc<Self>) -> mpsc::Receiver<AudioEvent> {
        Arc::get_mut(self)
            .expect("audio engine has multiple owners when taking audio events")
            .audio_event_rx
            .take()
            .expect("audio events already taken")
    }

    pub fn telemetry(&self) -> Arc<AudioTelemetry> {
        self.telemetry.clone()
    }

    pub fn output_devices(&self) -> Vec<String> {
        self.device_names.clone()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        debug!("Audio engine shutting down");
        // Drop the command sender so no new commands arrive.
        self.command_tx.take();
        // Signal the router thread to shut down (the command channel may still
        // have a live clone in the spawned ExecutionEngine task).
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(thread) = self.runtime_thread.take() {
            let _ = thread.join();
        }
        info!("Audio engine stopped");
    }
}

async fn driver_router(
    mut rx: mpsc::Receiver<DSPCommand>,
    mut driver: Box<dyn AudioDriver>,
    shutdown: &mut tokio::sync::oneshot::Receiver<()>,
) {
    info!("Audio driver router started");
    loop {
        tokio::select! {
            command = rx.recv() => {
                let Some(command) = command else {
                    info!("Audio driver router stopped (command channel closed)");
                    return;
                };
                let result = match command {
                    DSPCommand::SetAudioDevice { device_name } => driver.set_device(device_name).await,
                    DSPCommand::Play {
                        cue_id,
                        file_path,
                        volume_db,
                        looping,
                    } => {
                        driver
                            .play_cue(cue_id, file_path, volume_db, looping, None)
                            .await
                    }
                    DSPCommand::Seek {
                        cue_id,
                        position_sec,
                    } => driver.seek_cue(cue_id, position_sec).await,
                    DSPCommand::Pause { cue_id } | DSPCommand::Stop { cue_id } => {
                        driver.stop_cue(cue_id).await
                    }
                };
                if let Err(error) = result {
                    error!(%error, "Audio driver command failed");
                }
            }
            _ = &mut *shutdown => {
                info!("Audio driver router stopped (shutdown signal)");
                return;
            }
        }
    }
}
